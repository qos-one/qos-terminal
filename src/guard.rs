//! Inbound checks for the Terminal server.
//!
//! The server proxies browser calls to the engine and may attach the local token.
//! It listens on loopback unless `QOS_TERMINAL_BIND` says otherwise, and it answers
//! a request only when the `Host` is loopback or listed in `QOS_TRUSTED_ORIGINS` /
//! `QOS_TERMINAL_HOST`. `POST`, `PUT`, `PATCH`, and `DELETE` must carry this process's
//! `qos_csrf` cookie and a matching origin (`http` on loopback, `https` on a public host).
//! The desktop client sends no `Origin`; a loopback request with only the cookie is accepted.

use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::OnceLock;

use dioxus::fullstack::FullstackContext;
use dioxus::server::axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use dioxus::server::http::{header, HeaderMap, HeaderValue, Method, StatusCode};

pub const CSRF_COOKIE: &str = "qos_csrf";

/// Address Dioxus should bind. Loopback by default. `IP=0.0.0.0` is not followed;
/// set `QOS_TERMINAL_BIND` to listen on another address. `PORT` defaults to 8080.
pub fn bind_addr() -> SocketAddr {
    let port = std::env::var("PORT").ok().and_then(|s| s.trim().parse().ok()).unwrap_or(8080);
    if let Some(ip) = env_ip("QOS_TERMINAL_BIND") {
        return SocketAddr::new(ip, port);
    }
    if let Some(ip) = env_ip("IP") {
        if ip.is_loopback() {
            return SocketAddr::new(ip, port);
        }
    }
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

fn env_ip(key: &str) -> Option<IpAddr> {
    std::env::var(key).ok().and_then(|s| s.trim().parse().ok())
}

/// Hostnames this process will answer. Always includes loopback names.
pub fn trusted_hosts() -> HashSet<String> {
    let mut hosts = HashSet::from([
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ]);
    for key in ["QOS_TRUSTED_ORIGINS", "QOS_TERMINAL_HOST"] {
        let Ok(raw) = std::env::var(key) else { continue };
        for part in raw.split([',', ' ']) {
            let part = part.trim();
            if part.is_empty() { continue; }
            let host = parse_origin(part).map(|o| o.host).or_else(|| host_key(part).map(|(h, _)| h));
            if let Some(host) = host { hosts.insert(host); }
        }
    }
    hosts
}

/// `host` header → (lowercase name, explicit port).
fn host_key(value: &str) -> Option<(String, Option<u16>)> {
    let value = value.trim();
    if value.is_empty() { return None; }
    if let Some(rest) = value.strip_prefix('[') {
        let (name, port) = if let Some((name, port)) = rest.rsplit_once("]:") {
            (name, port.parse().ok())
        } else {
            (rest.trim_end_matches(']'), None)
        };
        if name.is_empty() { return None; }
        return Some((name.to_ascii_lowercase(), port));
    }
    if let Some((name, port)) = value.rsplit_once(':') {
        if !name.is_empty() && !name.contains(':') {
            if let Ok(port) = port.parse::<u16>() {
                return Some((name.to_ascii_lowercase(), Some(port)));
            }
        }
    }
    Some((value.to_ascii_lowercase(), None))
}

struct ParsedOrigin { scheme: String, host: String, port: u16 }

/// Origin or absolute URL. `http` defaults to port 80, `https` to 443.
fn parse_origin(value: &str) -> Option<ParsedOrigin> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("null") { return None; }
    let (scheme, rest) = value.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    let default_port = match scheme.as_str() {
        "http" => 80u16,
        "https" => 443u16,
        _ => return None,
    };
    let rest = rest.split(['/', '?', '#']).next().unwrap_or("");
    let rest = rest.rsplit('@').next().unwrap_or("");
    let (host, port) = host_key(rest)?;
    Some(ParsedOrigin { scheme, host, port: port.unwrap_or(default_port) })
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

/// Host and Origin/Referer name the same site. Loopback is `http`; a configured public host is `https`.
fn same_origin(host_header: &str, origin: &str) -> bool {
    let Some((host, host_port)) = host_key(host_header) else { return false };
    let Some(origin) = parse_origin(origin) else { return false };
    if host != origin.host { return false; }
    let scheme_ok = if is_loopback_host(&host) { origin.scheme == "http" } else { origin.scheme == "https" };
    if !scheme_ok { return false; }
    match host_port {
        Some(port) => port == origin.port,
        None => origin.port == if origin.scheme == "https" { 443 } else { 80 },
    }
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok()).map(str::trim).filter(|s| !s.is_empty())
}

fn safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn tokens_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() { return false; }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) { diff |= x ^ y; }
    diff == 0
}

fn fetch_site_cross(headers: &HeaderMap) -> bool {
    header_str(headers, "sec-fetch-site").is_some_and(|s| s.eq_ignore_ascii_case("cross-site"))
}

/// Whether this inbound request may be proxied to the engine.
pub fn decide(method: &Method, headers: &HeaderMap, csrf: &str, hosts: &HashSet<String>) -> Result<(), ()> {
    let host = header_str(headers, "host").ok_or(())?;
    let (name, _) = host_key(host).ok_or(())?;
    if !hosts.contains(&name) || fetch_site_cross(headers) { return Err(()); }
    if safe_method(method) {
        if let Some(origin) = header_str(headers, "origin") {
            if !same_origin(host, origin) { return Err(()); }
        }
        return Ok(());
    }
    let origin = header_str(headers, "origin");
    let referer = header_str(headers, "referer");
    let site = header_str(headers, "sec-fetch-site");
    // The desktop app calls this server with reqwest: a loopback Host, the cookie jar, and none of
    // Origin, Referer, or Sec-Fetch-Site. A browser request sends at least one of those.
    if origin.is_none() && referer.is_none() && site.is_none() {
        if !is_loopback_host(&name) { return Err(()); }
    } else {
        if let Some(site) = site {
            if !site.eq_ignore_ascii_case("same-origin") && !site.eq_ignore_ascii_case("none") {
                return Err(());
            }
        }
        match origin {
            Some(origin) => { if !same_origin(host, origin) { return Err(()); } }
            None => {
                let referer = referer.ok_or(())?;
                if !same_origin(host, referer) { return Err(()); }
            }
        }
    }
    let cookie = header_str(headers, "cookie").unwrap_or("");
    let presented = crate::calc::cookie_value(cookie, CSRF_COOKIE).unwrap_or_default();
    if !tokens_eq(&presented, csrf) { return Err(()); }
    Ok(())
}

pub fn csrf_token() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        let mut buf = [0u8; 32];
        let mut f = std::fs::File::open("/dev/urandom").expect("entropy");
        std::io::Read::read_exact(&mut f, &mut buf).expect("entropy");
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(64);
        for b in buf {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0xf) as usize] as char);
        }
        out
    })
}

fn csrf_cookie(token: &str, secure: bool) -> String {
    let mut cookie = format!("{CSRF_COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict");
    if secure { cookie.push_str("; Secure"); }
    cookie
}

fn https_request(headers: &HeaderMap) -> bool {
    header_str(headers, "x-forwarded-proto")
        .and_then(|v| v.split(',').next())
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("https"))
}

/// Server-function body: refuse before the engine client is built.
/// No request context (in-process tests) is allowed through.
pub fn allow_current() -> Result<(), ()> {
    let Some(ctx) = FullstackContext::current() else { return Ok(()) };
    let parts = ctx.parts_mut();
    decide(&parts.method, &parts.headers, csrf_token(), &trusted_hosts())
}

async fn inbound(req: Request, next: Next) -> Response {
    let headers = req.headers().clone();
    let method = req.method().clone();
    if decide(&method, &headers, csrf_token(), &trusted_hosts()).is_err() {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(dioxus::server::axum::body::Body::from("forbidden"))
            .unwrap();
    }
    let secure = https_request(&headers);
    let mut res = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&csrf_cookie(csrf_token(), secure)) {
        res.headers_mut().append(header::SET_COOKIE, value);
    }
    res
}

/// Bind loopback (see [`bind_addr`]) and serve the Terminal with the inbound check.
pub fn serve() -> ! {
    let addr = bind_addr();
    // SAFETY: process start, on the main thread, before the server runtime reads IP/PORT.
    unsafe {
        std::env::set_var("IP", addr.ip().to_string());
        std::env::set_var("PORT", addr.port().to_string());
    }
    let _ = csrf_token();
    dioxus::server::serve(|| async {
        let router = dioxus::server::router(crate::app::App)
            .layer(dioxus::server::axum::middleware::from_fn(inbound));
        Ok(router)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    const KEYS: [&str; 5] = ["IP", "PORT", "QOS_TERMINAL_BIND", "QOS_TRUSTED_ORIGINS", "QOS_TERMINAL_HOST"];

    struct Env {
        saved: Vec<(&'static str, Option<String>)>,
        _lock: MutexGuard<'static, ()>,
    }
    impl Env {
        fn acquire() -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let saved = KEYS.iter().map(|k| (*k, std::env::var(k).ok())).collect();
            Self { saved, _lock: lock }
        }
        fn set(&self, key: &str, val: &str) { unsafe { std::env::set_var(key, val) } }
        fn unset(&self, key: &str) { unsafe { std::env::remove_var(key) } }
    }
    impl Drop for Env {
        fn drop(&mut self) {
            for (k, v) in &self.saved {
                match v {
                    Some(val) => unsafe { std::env::set_var(k, val) },
                    None => unsafe { std::env::remove_var(k) },
                }
            }
        }
    }

    fn clear(env: &Env) {
        for k in KEYS { env.unset(k); }
    }

    #[test]
    fn bind_addr_is_loopback_unless_explicit() {
        let env = Env::acquire();
        clear(&env);
        assert_eq!(bind_addr(), SocketAddr::from((Ipv4Addr::LOCALHOST, 8080)));
        env.set("PORT", "9090");
        assert_eq!(bind_addr(), SocketAddr::from((Ipv4Addr::LOCALHOST, 9090)));
        env.set("IP", "0.0.0.0");
        assert_eq!(bind_addr().ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        env.set("IP", "192.168.1.9");
        assert_eq!(bind_addr().ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        env.set("IP", "127.0.0.1");
        assert_eq!(bind_addr().ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        env.set("IP", "::1");
        assert!(bind_addr().ip().is_loopback());
        env.set("QOS_TERMINAL_BIND", "0.0.0.0");
        assert_eq!(bind_addr().ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(bind_addr().port(), 9090);
    }

    #[test]
    fn trusted_hosts_include_configured_site() {
        let env = Env::acquire();
        clear(&env);
        let hosts = trusted_hosts();
        assert!(hosts.contains("127.0.0.1") && hosts.contains("localhost") && hosts.contains("::1"));
        env.set("QOS_TRUSTED_ORIGINS", "https://terminal.example.com, https://desk.example");
        env.set("QOS_TERMINAL_HOST", "extra.example");
        let hosts = trusted_hosts();
        assert!(hosts.contains("terminal.example.com"));
        assert!(hosts.contains("desk.example"));
        assert!(hosts.contains("extra.example"));
    }

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(dioxus::server::http::HeaderName::from_bytes(k.as_bytes()).unwrap(), HeaderValue::from_str(v).unwrap());
        }
        h
    }

    fn loopback() -> HashSet<String> {
        HashSet::from(["localhost".into(), "127.0.0.1".into(), "::1".into()])
    }

    #[test]
    fn same_origin_requests_pass_and_cross_origin_requests_do_not() {
        let hosts = loopback();
        let csrf = "unit-csrf-token";
        let get = headers(&[("host", "127.0.0.1:8080")]);
        assert!(decide(&Method::GET, &get, csrf, &hosts).is_ok());

        let foreign_get = headers(&[("host", "127.0.0.1:8080"), ("origin", "https://evil.example")]);
        assert!(decide(&Method::GET, &foreign_get, csrf, &hosts).is_err());

        let rebound = headers(&[("host", "evil.example"), ("origin", "http://evil.example")]);
        assert!(decide(&Method::GET, &rebound, csrf, &hosts).is_err());
        assert!(decide(&Method::POST, &rebound, csrf, &hosts).is_err());

        let cross_post = headers(&[
            ("host", "127.0.0.1:8080"),
            ("origin", "https://evil.example"),
            ("cookie", "qos_csrf=unit-csrf-token"),
            ("sec-fetch-site", "cross-site"),
        ]);
        for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
            assert!(decide(&method, &cross_post, csrf, &hosts).is_err());
        }

        let no_cookie = headers(&[
            ("host", "127.0.0.1:8080"),
            ("origin", "http://127.0.0.1:8080"),
            ("sec-fetch-site", "same-origin"),
        ]);
        assert!(decide(&Method::POST, &no_cookie, csrf, &hosts).is_err());

        let bad_cookie = headers(&[
            ("host", "127.0.0.1:8080"),
            ("origin", "http://127.0.0.1:8080"),
            ("cookie", "qos_csrf=other"),
            ("sec-fetch-site", "same-origin"),
        ]);
        assert!(decide(&Method::POST, &bad_cookie, csrf, &hosts).is_err());

        let ok = headers(&[
            ("host", "127.0.0.1:8080"),
            ("origin", "http://127.0.0.1:8080"),
            ("cookie", "other=1; qos_csrf=unit-csrf-token"),
            ("sec-fetch-site", "same-origin"),
        ]);
        for method in [Method::POST, Method::PUT, Method::DELETE] {
            assert!(decide(&method, &ok, csrf, &hosts).is_ok());
        }

        let referer_only = headers(&[
            ("host", "localhost:8080"),
            ("referer", "http://localhost:8080/"),
            ("cookie", "qos_csrf=unit-csrf-token"),
        ]);
        assert!(decide(&Method::POST, &referer_only, csrf, &hosts).is_ok());

        let null_origin = headers(&[
            ("host", "127.0.0.1:8080"),
            ("origin", "null"),
            ("cookie", "qos_csrf=unit-csrf-token"),
        ]);
        assert!(decide(&Method::POST, &null_origin, csrf, &hosts).is_err());

        // Desktop / dx serve: reqwest sends the cookie and a loopback Host, and no browser headers.
        let desktop = headers(&[("host", "127.0.0.1:8080"), ("cookie", "qos_csrf=unit-csrf-token")]);
        for method in [Method::POST, Method::PUT, Method::DELETE] {
            assert!(decide(&method, &desktop, csrf, &hosts).is_ok());
        }
        let desktop_bad_cookie = headers(&[("host", "127.0.0.1:8080"), ("cookie", "qos_csrf=other")]);
        assert!(decide(&Method::POST, &desktop_bad_cookie, csrf, &hosts).is_err());
        let desktop_no_cookie = headers(&[("host", "localhost:8080")]);
        assert!(decide(&Method::POST, &desktop_no_cookie, csrf, &hosts).is_err());
        // A browser header is present, so the desktop exception does not apply.
        let site_only = headers(&[
            ("host", "127.0.0.1:8080"),
            ("cookie", "qos_csrf=unit-csrf-token"),
            ("sec-fetch-site", "same-origin"),
        ]);
        assert!(decide(&Method::POST, &site_only, csrf, &hosts).is_err());
        let cross_site_only = headers(&[
            ("host", "127.0.0.1:8080"),
            ("cookie", "qos_csrf=unit-csrf-token"),
            ("sec-fetch-site", "cross-site"),
        ]);
        assert!(decide(&Method::POST, &cross_site_only, csrf, &hosts).is_err());
    }

    #[test]
    fn public_host_matches_configured_origin() {
        let mut hosts = loopback();
        hosts.insert("terminal.example.com".into());
        let csrf = "unit-csrf-token";
        let ok = headers(&[
            ("host", "terminal.example.com"),
            ("origin", "https://terminal.example.com"),
            ("cookie", "qos_csrf=unit-csrf-token"),
            ("sec-fetch-site", "same-origin"),
        ]);
        assert!(decide(&Method::POST, &ok, csrf, &hosts).is_ok());
        let other = headers(&[
            ("host", "terminal.example.com"),
            ("origin", "https://other.example"),
            ("cookie", "qos_csrf=unit-csrf-token"),
        ]);
        assert!(decide(&Method::POST, &other, csrf, &hosts).is_err());
        // No port on Host: scheme has to match (http on loopback, https on a public host).
        let http_public = headers(&[
            ("host", "terminal.example.com"),
            ("origin", "http://terminal.example.com"),
            ("cookie", "qos_csrf=unit-csrf-token"),
        ]);
        assert!(decide(&Method::POST, &http_public, csrf, &hosts).is_err());
        let https_loopback = headers(&[
            ("host", "127.0.0.1"),
            ("origin", "https://127.0.0.1"),
            ("cookie", "qos_csrf=unit-csrf-token"),
        ]);
        assert!(decide(&Method::GET, &https_loopback, csrf, &hosts).is_err());
        assert!(decide(&Method::POST, &https_loopback, csrf, &hosts).is_err());
        let https_loopback_port = headers(&[
            ("host", "127.0.0.1:8080"),
            ("origin", "https://127.0.0.1:8080"),
            ("cookie", "qos_csrf=unit-csrf-token"),
        ]);
        assert!(decide(&Method::POST, &https_loopback_port, csrf, &hosts).is_err());
        // No browser headers on a public host is not the desktop client.
        let bare_public = headers(&[
            ("host", "terminal.example.com"),
            ("cookie", "qos_csrf=unit-csrf-token"),
        ]);
        assert!(decide(&Method::POST, &bare_public, csrf, &hosts).is_err());
    }

    #[test]
    fn allow_current_rejects_a_cross_origin_post() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("runtime");
        rt.block_on(async {
            let (parts, _) = dioxus::server::http::Request::builder()
                .method("POST")
                .uri("/api/order")
                .header("host", "127.0.0.1:8080")
                .header("origin", "https://evil.example")
                .body(())
                .unwrap()
                .into_parts();
            FullstackContext::new(parts).scope(async {
                assert!(allow_current().is_err());
            }).await;

            let (parts, _) = dioxus::server::http::Request::builder()
                .method("GET")
                .uri("/api/snapshot")
                .header("host", "127.0.0.1:8080")
                .body(())
                .unwrap()
                .into_parts();
            FullstackContext::new(parts).scope(async {
                assert!(allow_current().is_ok());
            }).await;
        });
    }
}
