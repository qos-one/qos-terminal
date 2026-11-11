//! Data types used by the UI, and the server functions that call the bot FastAPI
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::calc::null_def;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Account {
    #[serde(default, deserialize_with = "null_def")] pub account_id: String,
    #[serde(default, deserialize_with = "null_def")] pub currency: String,
    #[serde(rename = "NetLiquidation", default, deserialize_with = "null_def")] pub net_liquidation: f64,
    #[serde(rename = "AvailableFunds", default, deserialize_with = "null_def")] pub available_funds: f64,
    #[serde(rename = "BuyingPower", default, deserialize_with = "null_def")] pub buying_power: f64,
    #[serde(rename = "UnrealizedPnL", default, deserialize_with = "null_def")] pub unrealized_pnl: f64,
    #[serde(rename = "RealizedPnL", default, deserialize_with = "null_def")] pub realized_pnl: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Market {
    pub id: String,
    pub name: String,
    #[serde(default)] pub exchange: String,
    #[serde(default)] pub tradable: bool,
    #[serde(default)] pub unit: String,
    #[serde(default)] pub decimals: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Instrument {
    pub key: String,
    pub market: String,
    pub symbol: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub exchange: String,
    #[serde(default)] pub currency: String,
    #[serde(default)] pub tradable: bool,
    #[serde(default)] pub unit: String,
    #[serde(default)] pub decimals: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    #[serde(default, deserialize_with = "null_def")] pub key: String,
    #[serde(default, deserialize_with = "null_def")] pub market: String,
    #[serde(default, deserialize_with = "null_def")] pub symbol: String,
    #[serde(default, deserialize_with = "null_def")] pub name: String,
    #[serde(default, deserialize_with = "null_def")] pub bid: f64,
    #[serde(default, deserialize_with = "null_def")] pub ask: f64,
    #[serde(default, deserialize_with = "null_def")] pub last: f64,
    #[serde(default, deserialize_with = "null_def")] pub mid: f64,
    #[serde(default, deserialize_with = "null_def")] pub spread_pip: f64,
    #[serde(default, deserialize_with = "null_def")] pub prev_close: f64,
    #[serde(default, deserialize_with = "null_def")] pub change_pct: f64,
    #[serde(default, deserialize_with = "null_def")] pub decimals: usize,
    #[serde(default, deserialize_with = "null_def")] pub tradable: bool,
    #[serde(default, deserialize_with = "null_def")] pub unit: String,
    #[serde(default)] pub time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MarketItem {
    pub id: String,
    pub name: String,
    #[serde(default)] pub subtitle: String,
    #[serde(default)] pub members: usize,
    #[serde(default)] pub key: String,
    #[serde(default)] pub symbol: String,
    #[serde(default)] pub last: f64,
    #[serde(default)] pub prev_close: f64,
    #[serde(default)] pub change_pct: f64,
    #[serde(default)] pub decimals: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct HeatItem {
    pub key: String,
    pub symbol: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub weight: f64,
    #[serde(default)] pub last: f64,
    #[serde(default)] pub change_pct: f64,
    #[serde(default)] pub decimals: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Breadth { #[serde(default)] pub up: usize, #[serde(default)] pub down: usize, #[serde(default)] pub total: usize }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Heatmap {
    pub group: String,
    pub name: String,
    #[serde(default)] pub items: Vec<HeatItem>,
    #[serde(default)] pub breadth: Option<Breadth>,
    #[serde(default)] pub missing: Vec<String>,
    #[serde(default)] pub age_s: f64,
    #[serde(default)] pub refreshing: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Bar { pub t: String, pub o: f64, pub h: f64, pub l: f64, pub c: f64, #[serde(default)] pub v: f64 }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Position {
    #[serde(default, deserialize_with = "null_def")] pub key: String,
    #[serde(default, deserialize_with = "null_def")] pub symbol: String,
    #[serde(default, deserialize_with = "null_def")] pub market: String,
    #[serde(default, deserialize_with = "null_def")] pub sec_type: String,
    #[serde(default, deserialize_with = "null_def")] pub position: f64,
    #[serde(default, deserialize_with = "null_def")] pub avg_cost: f64,
    #[serde(default, deserialize_with = "null_def")] pub market_price: f64,
    #[serde(default, deserialize_with = "null_def")] pub market_value: f64,
    #[serde(default, deserialize_with = "null_def")] pub unrealized_pnl: f64,
    #[serde(default, deserialize_with = "null_def")] pub realized_pnl: f64,
    #[serde(default, deserialize_with = "null_def")] pub decimals: usize,
    #[serde(default, deserialize_with = "null_def")] pub source: String,
    #[serde(default, deserialize_with = "null_def")] pub strategy: String,
    #[serde(default, deserialize_with = "null_def")] pub sources: Vec<PosSource>,
    #[serde(default, deserialize_with = "null_def")] pub recon: String,
    #[serde(default, deserialize_with = "null_def")] pub recon_note: String,
    /// currency of the local values; `usd` = market value / P&L above are USD (false → they are in `currency`)
    #[serde(default, deserialize_with = "null_def")] pub currency: String,
    #[serde(default = "d_true")] pub usd: bool,
    #[serde(default)] pub market_value_local: Option<f64>,
    #[serde(default)] pub unrealized_pnl_local: Option<f64>,
    /// cash balance the API cannot size (no price for the USD pair) — shown, never treated as flat
    #[serde(default, deserialize_with = "null_def")] pub unpriced: bool,
    #[serde(default, deserialize_with = "null_def")] pub cash_balance: f64,
    #[serde(default, deserialize_with = "null_def")] pub cash_currency: String,
    /// the account's cash balance disagrees with the tracked position (e.g. a trade made outside QOS)
    #[serde(default)] pub cash_warn: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PosSource {
    #[serde(default, deserialize_with = "null_def")] pub source: String,
    #[serde(default, deserialize_with = "null_def")] pub strategy: String,
    #[serde(default, deserialize_with = "null_def")] pub qty: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ReconRow {
    #[serde(default, deserialize_with = "null_def")] pub key: String,
    #[serde(default, deserialize_with = "null_def")] pub ib_qty: f64,
    #[serde(default, deserialize_with = "null_def")] pub node_qty: f64,
    #[serde(default, deserialize_with = "null_def")] pub other_qty: f64,
    #[serde(default, deserialize_with = "null_def")] pub expected: f64,
    #[serde(default, deserialize_with = "null_def")] pub diff: f64,
    #[serde(default, deserialize_with = "null_def")] pub status: String,
    #[serde(default, deserialize_with = "null_def")] pub note: String,
    /// manual/agent history holds a position while IBKR is flat
    #[serde(default, deserialize_with = "null_def")] pub drift: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Reconcile {
    #[serde(default, deserialize_with = "null_def")] pub ts: String,
    #[serde(default, deserialize_with = "null_def")] pub status: String,
    #[serde(default, deserialize_with = "null_def")] pub checked: i64,
    #[serde(default, deserialize_with = "null_def")] pub node_keys: i64,
    #[serde(default, deserialize_with = "null_def")] pub issues: i64,
    #[serde(default, deserialize_with = "null_def")] pub rows: Vec<ReconRow>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Order {
    #[serde(default, deserialize_with = "null_def")] pub order_id: i64,
    #[serde(default, deserialize_with = "null_def")] pub key: String,
    #[serde(default, deserialize_with = "null_def")] pub symbol: String,
    #[serde(default, deserialize_with = "null_def")] pub side: String,
    #[serde(rename = "type", default, deserialize_with = "null_def")] pub order_type: String,
    #[serde(default, deserialize_with = "null_def")] pub qty: f64,
    #[serde(default)] pub limit_price: Option<f64>,
    #[serde(default)] pub stop_price: Option<f64>,
    #[serde(default)] pub parent_id: Option<i64>,
    #[serde(default)] pub oca_group: Option<String>,
    #[serde(default, deserialize_with = "null_def")] pub tif: String,
    #[serde(rename = "ref", default, deserialize_with = "null_def")] pub order_ref: String,
    #[serde(default, deserialize_with = "null_def")] pub status: String,
    #[serde(default, deserialize_with = "null_def")] pub filled: f64,
    #[serde(default, deserialize_with = "null_def")] pub remaining: f64,
    #[serde(default, deserialize_with = "null_def")] pub source: String,
    #[serde(default, deserialize_with = "null_def")] pub strategy: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Fill {
    #[serde(default)] pub time: Option<String>,
    #[serde(default, deserialize_with = "null_def")] pub key: String,
    #[serde(default, deserialize_with = "null_def")] pub symbol: String,
    #[serde(default, deserialize_with = "null_def")] pub side: String,
    #[serde(default, deserialize_with = "null_def")] pub qty: f64,
    /// null for adjustment rows without a known price
    #[serde(default)] pub price: Option<f64>,
    #[serde(default)] pub commission: Option<f64>,
    #[serde(default)] pub realized_pnl: Option<f64>,
    #[serde(default, deserialize_with = "null_def")] pub source: String,
    #[serde(default, deserialize_with = "null_def")] pub strategy: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    #[serde(default, deserialize_with = "null_def")] pub account: Account,
    #[serde(default, deserialize_with = "null_def")] pub positions: Vec<Position>,
    #[serde(default, deserialize_with = "null_def")] pub orders: Vec<Order>,
    #[serde(default, deserialize_with = "null_def")] pub fills: Vec<Fill>,
    #[serde(default, deserialize_with = "null_def")] pub ib_connected: bool,
    #[serde(default, deserialize_with = "null_def")] pub mode: String,
    #[serde(default, deserialize_with = "null_def")] pub reconcile: Reconcile,
    #[serde(default, deserialize_with = "null_def")] pub running_nodes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct OrderRequest {
    pub key: String,
    pub side: String,
    pub qty: f64,
    pub order_type: String,
    pub price: Option<f64>,
    pub tif: String,
    #[serde(default)] pub take_profit: Option<f64>,
    #[serde(default)] pub stop_loss: Option<f64>,
    #[serde(default)] pub trail_amount: Option<f64>,
    #[serde(default)] pub cash_qty: Option<f64>,     // crypto: USD amount instead of coin quantity
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ClosedItem { pub key: String, #[serde(default)] pub qty: f64, #[serde(default)] pub status: String, #[serde(default)] pub avg_fill_price: f64 }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct OrderResult {
    #[serde(default)] pub order_id: i64,
    #[serde(default)] pub status: String,
    #[serde(default)] pub filled: f64,
    #[serde(default)] pub avg_fill_price: f64,
    #[serde(default)] pub message: String,
    #[serde(default)] pub child_order_ids: Vec<i64>,
}

// ---------------- server side helpers ----------------
#[cfg(feature = "server")]
mod http {
    use super::*;
    pub fn base() -> String {
        std::env::var("BOT_API").unwrap_or_else(|_| "http://127.0.0.1:8000".into())
    }
    /// Desktop app: one user, so the operator token is kept in this process.
    pub static OP_TOKEN: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
    /// Web Terminal (QOS_PUBLIC_TERMINAL=1): one server serves many browsers, so the token lives in each browser's
    /// HttpOnly cookie and is read per request; the process-wide token is never used (it would hand one operator's
    /// session to every visitor).
    pub const COOKIE: &str = "qos_op";
    pub fn web_mode() -> bool { std::env::var("QOS_PUBLIC_TERMINAL").map(|v| v == "1").unwrap_or(false) }
    fn cookie_token() -> Option<String> {
        let ctx = dioxus::fullstack::FullstackContext::current()?;
        let parts = ctx.parts_mut();
        let raw = parts.headers.get(reqwest::header::COOKIE)?.to_str().ok()?.to_string();
        crate::calc::cookie_value(&raw, COOKIE)
    }
    pub fn token() -> String {
        if web_mode() { return cookie_token().unwrap_or_default(); }
        OP_TOKEN.lock().map(|t| t.clone()).unwrap_or_default()
    }
    /// After sign-in / sign-out: the browser cookie (web) or the process token (desktop)
    pub fn set_token(tok: &str) {
        if web_mode() {
            if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
                if let Ok(v) = reqwest::header::HeaderValue::from_str(&crate::calc::op_cookie(COOKIE, tok)) {
                    ctx.add_response_header(reqwest::header::SET_COOKIE, v);
                }
            }
        } else if let Ok(mut t) = OP_TOKEN.lock() {
            *t = tok.to_string();
        }
    }
    /// Personal machine, no operator yet. Same file the engine writes (mode 0600).
    /// QOS_LOCAL_TOKEN_FILE, else QOS_DATA_DIR/local_token, else QOS_HOME/data/local_token,
    /// else ~/trading/data/local_token. Missing file → no header (the engine then stays read-only).
    fn local_token_path() -> Option<std::path::PathBuf> {
        let nonempty = |key: &str| std::env::var(key).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        if let Some(p) = nonempty("QOS_LOCAL_TOKEN_FILE") { return Some(std::path::PathBuf::from(p)); }
        if let Some(d) = nonempty("QOS_DATA_DIR") { return Some(std::path::PathBuf::from(d).join("local_token")); }
        if let Some(h) = nonempty("QOS_HOME") { return Some(std::path::PathBuf::from(h).join("data").join("local_token")); }
        let home = nonempty("HOME")?;
        Some(std::path::PathBuf::from(home).join("trading").join("data").join("local_token"))
    }
    fn auth_headers() -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        if let Ok(v) = reqwest::header::HeaderValue::from_str(&token()) { h.insert("x-operator-token", v); }
        if let Some(path) = local_token_path() {
            if let Ok(text) = std::fs::read_to_string(path) {
                let tok = text.trim();
                if !tok.is_empty() {
                    if let Ok(mut v) = reqwest::header::HeaderValue::from_str(tok) {
                        v.set_sensitive(true);
                        h.insert("x-qos-local-token", v);
                    }
                }
            }
        }
        h
    }
    pub fn client() -> Result<reqwest::Client, ServerFnError> {
        reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).default_headers(auth_headers()).build()
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    async fn handle<T: serde::de::DeserializeOwned>(r: reqwest::Response) -> Result<T, ServerFnError> {
        if !r.status().is_success() {
            let s = r.status();
            let body = r.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<serde_json::Value>(&body).ok()
                .and_then(|v| v.get("detail").and_then(|d| d.as_str().map(String::from)))
                .unwrap_or(body);
            return Err(ServerFnError::new(format!("{s}: {msg}")));
        }
        r.json().await.map_err(|e| ServerFnError::new(e.to_string()))
    }
    pub async fn get_slow<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, ServerFnError> {
        let c = reqwest::Client::builder().timeout(std::time::Duration::from_secs(90)).default_headers(auth_headers()).build()
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        let r = c.get(format!("{}{}", base(), path)).send().await
            .map_err(|e| ServerFnError::new(format!("bot API not responding: {e}")))?;
        handle(r).await
    }
    pub async fn get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, ServerFnError> {
        let r = client()?.get(format!("{}{}", base(), path)).send().await
            .map_err(|e| ServerFnError::new(format!("bot API not responding: {e}")))?;
        handle(r).await
    }
    pub async fn send<B: Serialize, T: serde::de::DeserializeOwned>(method: reqwest::Method, path: &str, body: &B) -> Result<T, ServerFnError> {
        let r = client()?.request(method, format!("{}{}", base(), path)).json(body).send().await
            .map_err(|e| ServerFnError::new(format!("bot API not responding: {e}")))?;
        handle(r).await
    }
    /// Commands that may take a while (e.g. stopping a Nautilus node waits up to 15 s for a clean shutdown)
    pub async fn send_slow<B: Serialize, T: serde::de::DeserializeOwned>(method: reqwest::Method, path: &str, body: &B) -> Result<T, ServerFnError> {
        send_timeout(method, path, body, 60).await
    }

    pub async fn send_timeout<B: Serialize, T: serde::de::DeserializeOwned>(method: reqwest::Method, path: &str, body: &B, secs: u64) -> Result<T, ServerFnError> {
        let c = reqwest::Client::builder().timeout(std::time::Duration::from_secs(secs)).default_headers(auth_headers()).build()
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        let r = c.request(method, format!("{}{}", base(), path)).json(body).send().await
            .map_err(|e| ServerFnError::new(format!("bot API not responding: {e}")))?;
        handle(r).await
    }
    pub fn enc(s: &str) -> String { s.replace('%', "%25").replace(' ', "%20").replace('&', "%26").replace('+', "%2B").replace('#', "%23") }
    /// "&tz=<IANA name>" for history requests ("today" / "week" start at the viewer's local midnight); empty = UTC
    pub fn tz_q(tz: &str) -> String { if tz.is_empty() { String::new() } else { format!("&tz={}", enc(tz)) } }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::path::PathBuf;
        use std::sync::{Mutex, MutexGuard};

        /// Serializes process-environment edits. Restores the previous values on drop.
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        const ENV_KEYS: [&str; 5] = ["QOS_LOCAL_TOKEN_FILE", "QOS_DATA_DIR", "QOS_HOME", "HOME", "QOS_PUBLIC_TERMINAL"];

        struct EnvGuard {
            saved: Vec<(&'static str, Option<String>)>,
            op: String,
            _lock: MutexGuard<'static, ()>,
        }
        impl EnvGuard {
            fn acquire() -> Self {
                let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
                let saved = ENV_KEYS.iter().map(|k| (*k, std::env::var(k).ok())).collect();
                let op = OP_TOKEN.lock().map(|t| t.clone()).unwrap_or_default();
                // Desktop token, not the web cookie path, so the operator header is deterministic.
                unsafe { std::env::remove_var("QOS_PUBLIC_TERMINAL") };
                Self { saved, op, _lock: lock }
            }
            fn set(&self, key: &str, val: &str) {
                // SAFETY: ENV_LOCK is held, so no other test mutates the process environment.
                unsafe { std::env::set_var(key, val) }
            }
            fn unset(&self, key: &str) {
                unsafe { std::env::remove_var(key) }
            }
        }
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                for (k, v) in &self.saved {
                    match v {
                        Some(val) => unsafe { std::env::set_var(k, val) },
                        None => unsafe { std::env::remove_var(k) },
                    }
                }
                if let Ok(mut t) = OP_TOKEN.lock() { *t = std::mem::take(&mut self.op); }
            }
        }

        fn clear_qos(env: &EnvGuard) {
            env.unset("QOS_LOCAL_TOKEN_FILE");
            env.unset("QOS_DATA_DIR");
            env.unset("QOS_HOME");
        }

        #[test]
        fn local_token_path_prefers_file_then_data_dir_then_home() {
            let env = EnvGuard::acquire();
            env.set("QOS_LOCAL_TOKEN_FILE", "  /tmp/explicit-token  ");
            env.set("QOS_DATA_DIR", "/var/qos-data");
            env.set("QOS_HOME", "/srv/qos/home");
            env.set("HOME", "/home/trader");
            assert_eq!(local_token_path(), Some(PathBuf::from("/tmp/explicit-token")));

            env.set("QOS_LOCAL_TOKEN_FILE", "   ");
            assert_eq!(local_token_path(), Some(PathBuf::from("/var/qos-data/local_token")));

            env.unset("QOS_DATA_DIR");
            assert_eq!(local_token_path(), Some(PathBuf::from("/srv/qos/home/data/local_token")));

            env.set("QOS_HOME", " ");
            assert_eq!(local_token_path(), Some(PathBuf::from("/home/trader/trading/data/local_token")));

            env.unset("HOME");
            assert_eq!(local_token_path(), None);
        }

        fn header_value(h: &reqwest::header::HeaderMap, name: &str) -> Option<String> {
            h.get(name).and_then(|v| v.to_str().ok()).map(str::to_string)
        }

        #[test]
        fn auth_headers_attach_trimmed_local_token_beside_operator_token() {
            let dir = std::env::temp_dir().join(format!("qos-local-token-{}-attach", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("local_token");
            std::fs::write(&path, " \n unit-test-local-token \t\n").unwrap();
            let env = EnvGuard::acquire();
            clear_qos(&env);
            env.set("QOS_LOCAL_TOKEN_FILE", path.to_str().unwrap());
            if let Ok(mut t) = OP_TOKEN.lock() { *t = "unit-test-operator".into(); }

            let h = auth_headers();
            assert_eq!(header_value(&h, "x-operator-token").as_deref(), Some("unit-test-operator"));
            assert_eq!(header_value(&h, "x-qos-local-token").as_deref(), Some("unit-test-local-token"));
            assert!(h.get("x-qos-local-token").unwrap().is_sensitive());
            // Debug must not show the file contents (HeaderValue is marked sensitive).
            let rendered = format!("{h:?}");
            assert!(!rendered.contains("unit-test-local-token"));
            drop(env);
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn missing_or_blank_token_file_adds_no_local_header() {
            let env = EnvGuard::acquire();
            clear_qos(&env);
            env.set("QOS_LOCAL_TOKEN_FILE", "/tmp/qos-local-token-does-not-exist");
            if let Ok(mut t) = OP_TOKEN.lock() { *t = "unit-test-operator".into(); }
            let missing = auth_headers();
            assert!(missing.get("x-qos-local-token").is_none());
            assert_eq!(header_value(&missing, "x-operator-token").as_deref(), Some("unit-test-operator"));

            let dir = std::env::temp_dir().join(format!("qos-local-token-{}-blank", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("local_token");
            std::fs::write(&path, " \n\t ").unwrap();
            env.set("QOS_LOCAL_TOKEN_FILE", path.to_str().unwrap());
            let blank = auth_headers();
            assert!(blank.get("x-qos-local-token").is_none());
            assert_eq!(header_value(&blank, "x-operator-token").as_deref(), Some("unit-test-operator"));
            drop(env);
            let _ = std::fs::remove_dir_all(&dir);
        }

        /// `client()` (used by `send()`) and the slow clients all share `auth_headers()`.
        /// This checks the bytes that leave the process.
        #[test]
        fn client_sends_local_token_header() {
            let dir = std::env::temp_dir().join(format!("qos-local-token-{}-wire", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("local_token");
            std::fs::write(&path, "  wire-local-token\n").unwrap();
            let env = EnvGuard::acquire();
            clear_qos(&env);
            env.set("QOS_LOCAL_TOKEN_FILE", path.to_str().unwrap());
            if let Ok(mut t) = OP_TOKEN.lock() { *t = "wire-operator".into(); }

            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            listener.set_nonblocking(true).unwrap();
            let server = std::thread::spawn(move || {
                let mut waited = 0;
                let (mut sock, _) = loop {
                    match listener.accept() {
                        Ok(pair) => break pair,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            if waited > 40 { return String::new(); }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            waited += 1;
                        }
                        Err(e) => panic!("accept: {e}"),
                    }
                };
                sock.set_nonblocking(false).unwrap();
                sock.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();
                let mut buf = Vec::new();
                let mut tmp = [0u8; 1024];
                while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    match std::io::Read::read(&mut sock, &mut tmp) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                    }
                }
                let _ = std::io::Write::write_all(
                    &mut sock,
                    b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{}",
                );
                String::from_utf8_lossy(&buf).into_owned()
            });

            let url = format!("http://{addr}/ping");
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
            let sent = rt.block_on(async { client().unwrap().get(url).send().await });
            let raw = server.join().expect("server thread");
            assert!(sent.is_ok(), "request did not complete");
            let lower = raw.to_ascii_lowercase();
            assert!(lower.contains("\r\nx-qos-local-token: wire-local-token\r\n"), "local token header was not sent");
            assert!(lower.contains("\r\nx-operator-token: wire-operator\r\n"), "operator token header was not sent");
            drop(env);
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

/// WebSocket address of the bot API (derived from BOT_API, so the realtime stream follows the configured host/port)
#[cfg(not(feature = "demo"))]
#[get("/api/ws_url")]
pub async fn get_ws_url() -> Result<String, ServerFnError> {
    // web Terminal behind a reverse proxy: QOS_WS_URL=/ws (same origin, the browser sends the operator cookie)
    if let Ok(u) = std::env::var("QOS_WS_URL") { if !u.trim().is_empty() { return Ok(u.trim().to_string()); } }
    // desktop: a browser WebSocket cannot send headers, so the operator token goes in the address (needed when the API
    // runs with QOS_PUBLIC_TERMINAL=1; ignored otherwise)
    let tok = http::token();
    let u = crate::calc::ws_url_from(&http::base());
    Ok(if tok.is_empty() || http::web_mode() { u } else { format!("{u}?op_token={}", http::enc(&tok)) })
}

#[cfg(not(feature = "demo"))]
#[get("/api/markets")]
pub async fn get_markets() -> Result<Vec<Market>, ServerFnError> {
    #[derive(Deserialize)] struct R { markets: Vec<Market> }
    let r: R = http::get("/markets").await?;
    Ok(r.markets)
}

#[cfg(not(feature = "demo"))]
#[get("/api/instruments?market&q")]
pub async fn get_instruments(market: String, q: String) -> Result<Vec<Instrument>, ServerFnError> {
    #[derive(Deserialize)] struct R { instruments: Vec<Instrument> }
    let r: R = http::get(&format!("/instruments?market={}&q={}&limit=300", market, http::enc(&q))).await?;
    Ok(r.instruments)
}

#[cfg(not(feature = "demo"))]
#[get("/api/watchlist")]
pub async fn get_watchlist() -> Result<Vec<Instrument>, ServerFnError> {
    #[derive(Deserialize)] struct R { items: Vec<Instrument> }
    let r: R = http::get("/watchlist").await?;
    Ok(r.items)
}

#[cfg(not(feature = "demo"))]
#[post("/api/watchlist/add")]
pub async fn add_watch(key: String) -> Result<Vec<Instrument>, ServerFnError> {
    #[derive(Deserialize)] struct R { items: Vec<Instrument> }
    let r: R = http::send(reqwest::Method::POST, &format!("/watchlist/{}", http::enc(&key)), &serde_json::json!({})).await?;
    Ok(r.items)
}

#[cfg(not(feature = "demo"))]
#[post("/api/watchlist/remove")]
pub async fn remove_watch(key: String) -> Result<Vec<Instrument>, ServerFnError> {
    #[derive(Deserialize)] struct R { items: Vec<Instrument> }
    let r: R = http::send(reqwest::Method::DELETE, &format!("/watchlist/{}", http::enc(&key)), &serde_json::json!({})).await?;
    Ok(r.items)
}

#[cfg(not(feature = "demo"))]
#[get("/api/quotes")]
pub async fn get_quotes() -> Result<Vec<Quote>, ServerFnError> {
    #[derive(Deserialize)] struct R { quotes: Vec<Quote> }
    let r: R = http::get("/quotes").await?;
    Ok(r.quotes)
}

#[cfg(not(feature = "demo"))]
#[get("/api/market")]
pub async fn get_market() -> Result<Vec<MarketItem>, ServerFnError> {
    #[derive(Deserialize)] struct R { items: Vec<MarketItem> }
    let r: R = http::get("/market").await?;
    Ok(r.items)
}

/// Bars plus the instrument's display decimals from /bars (accepts a plain bar list too, e.g. from the web demo backend)
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct BarsResp { pub bars: Vec<Bar>, pub decimals: Option<usize> }
impl<'de> Deserialize<'de> for BarsResp {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)] #[serde(untagged)] enum W { Full { bars: Vec<Bar>, #[serde(default)] decimals: Option<usize> }, Plain(Vec<Bar>) }
        Ok(match W::deserialize(d)? { W::Full { bars, decimals } => BarsResp { bars, decimals }, W::Plain(bars) => BarsResp { bars, decimals: None } })
    }
}

#[cfg(not(feature = "demo"))]
#[get("/api/bars?key&bar_size&duration")]
pub async fn get_bars(key: String, bar_size: String, duration: String) -> Result<BarsResp, ServerFnError> {
    http::get(&format!("/bars?key={}&bar_size={}&duration={}", http::enc(&key), http::enc(&bar_size), http::enc(&duration))).await
}

#[cfg(not(feature = "demo"))]
#[get("/api/snapshot")]
pub async fn get_snapshot() -> Result<Snapshot, ServerFnError> {
    #[derive(Deserialize)] struct H { ib_connected: bool, #[serde(default)] mode: String, #[serde(default)] running_nodes: Vec<String> }
    #[derive(Deserialize)] struct P { positions: Vec<Position>, #[serde(default)] reconcile: Option<Reconcile> }
    #[derive(Deserialize)] struct O { orders: Vec<Order> }
    #[derive(Deserialize)] struct F { fills: Vec<Fill> }
    let h: H = http::get("/health").await?;
    // a failing /account must not blank the whole snapshot: empty account → the UI keeps the last known values, marked stale
    let account: Account = http::get("/account").await.unwrap_or_default();
    let p: P = http::get("/positions").await?;
    let o: O = http::get("/orders").await?;
    let f: F = http::get("/fills?limit=30").await?;
    Ok(Snapshot { account, positions: p.positions, orders: o.orders, fills: f.fills, ib_connected: h.ib_connected, mode: h.mode, reconcile: p.reconcile.unwrap_or_default(), running_nodes: h.running_nodes })
}

#[cfg(not(feature = "demo"))]
#[post("/api/order")]
pub async fn place_order(req: OrderRequest) -> Result<OrderResult, ServerFnError> {
    http::send(reqwest::Method::POST, "/order", &req).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/order/cancel")]
pub async fn cancel_order(order_id: i64) -> Result<OrderResult, ServerFnError> {
    http::send(reqwest::Method::POST, &format!("/order/{order_id}/cancel"), &serde_json::json!({})).await
}

/// force = close even when a running node holds the position (the API answers 409 otherwise)
#[cfg(not(feature = "demo"))]
#[post("/api/position/close")]
pub async fn close_position(key: String, force: bool) -> Result<OrderResult, ServerFnError> {
    http::send(reqwest::Method::POST, &format!("/positions/{}/close?force={force}", http::enc(&key)), &serde_json::json!({})).await
}

// ---------------- trading mode ----------------
#[cfg(not(feature = "demo"))]
#[post("/api/mode")]
pub async fn set_mode(mode: String) -> Result<String, ServerFnError> {
    #[derive(Deserialize)] struct R { mode: String }
    let r: R = http::send(reqwest::Method::PUT, "/mode", &serde_json::json!({ "mode": mode })).await?;
    Ok(r.mode)
}

#[cfg(not(feature = "demo"))]
#[get("/api/heatmap?group")]
pub async fn get_heatmap(group: String) -> Result<Heatmap, ServerFnError> {
    http::get_slow(&format!("/heatmap?group={group}")).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/order/modify")]
pub async fn modify_order(order_id: i64, qty: Option<f64>, price: Option<f64>) -> Result<OrderResult, ServerFnError> {
    http::send(reqwest::Method::PUT, &format!("/order/{order_id}"), &serde_json::json!({ "qty": qty, "price": price })).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/positions/close_all")]
pub async fn close_all_positions() -> Result<Vec<ClosedItem>, ServerFnError> {
    #[derive(Deserialize)] struct R { closed: Vec<ClosedItem> }
    let r: R = http::send(reqwest::Method::POST, "/positions/close_all", &serde_json::json!({})).await?;
    Ok(r.closed)
}

#[cfg(not(feature = "demo"))]
#[post("/api/position/reverse")]
pub async fn reverse_position(key: String) -> Result<OrderResult, ServerFnError> {
    http::send(reqwest::Method::POST, &format!("/positions/{}/reverse", http::enc(&key)), &serde_json::json!({})).await
}

// ---------------- risk / audit ----------------
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RiskSettings {
    #[serde(default)] pub allow_live: Option<bool>,
    #[serde(default)] pub max_daily_loss_usd: f64,
    #[serde(default)] pub max_order_notional_usd: f64,
    #[serde(default)] pub max_position_notional_usd: f64,
    #[serde(default)] pub max_open_positions: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DailyPnl {
    #[serde(default, deserialize_with = "null_def")] pub date: String,
    #[serde(default, deserialize_with = "null_def")] pub start_netliq: f64,
    #[serde(default, deserialize_with = "null_def")] pub netliq: f64,
    #[serde(default, deserialize_with = "null_def")] pub pnl: f64,
    #[serde(default, deserialize_with = "null_def")] pub limit: f64,
    #[serde(default, deserialize_with = "null_def")] pub used_pct: f64,
    /// where today's start NetLiq came from: history | first_seen | operator | unconfirmed
    #[serde(default, deserialize_with = "null_def")] pub baseline: String,
    #[serde(default = "d_true")] pub baseline_confirmed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RiskAccount {
    #[serde(default, deserialize_with = "null_def")] pub net_liquidation: f64,
    #[serde(default, deserialize_with = "null_def")] pub init_margin: f64,
    #[serde(default, deserialize_with = "null_def")] pub maint_margin: f64,
    #[serde(default, deserialize_with = "null_def")] pub excess_liquidity: f64,
    #[serde(default, deserialize_with = "null_def")] pub available_funds: f64,
    #[serde(default, deserialize_with = "null_def")] pub buying_power: f64,
    #[serde(default, deserialize_with = "null_def")] pub gross_exposure: f64,
    #[serde(default, deserialize_with = "null_def")] pub leverage: f64,
    /// a position could not be valued in USD → gross exposure is a lower bound
    #[serde(default, deserialize_with = "null_def")] pub gross_exposure_partial: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Exposure {
    pub key: String,
    pub symbol: String,
    #[serde(default, deserialize_with = "null_def")] pub qty: f64,
    #[serde(default, deserialize_with = "null_def")] pub market_value: f64,
    #[serde(default, deserialize_with = "null_def")] pub unrealized_pnl: f64,
    #[serde(default = "d_true")] pub usd: bool,
    #[serde(default, deserialize_with = "null_def")] pub currency: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistantCfg {
    #[serde(default)] pub write_enabled: bool, #[serde(default)] pub live: bool,
    #[serde(default)] pub max_order_notional_usd: f64, #[serde(default)] pub max_orders_per_hour: i64,
    #[serde(default)] pub allow_short: bool, #[serde(default = "d_true")] pub allow_agents: bool,
    #[serde(default)] pub confirm_ttl_s: i64, #[serde(default)] pub orders_last_hour: i64, #[serde(default)] pub operator_exists: bool,
    /// on = the assistant confirms its own orders; off = an operator approves each one in the Terminal
    #[serde(default)] pub auto_confirm: bool,
}

/// an assistant order waiting for the operator's approval (auto-confirm off)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AssistantTicket {
    #[serde(default)] pub token: String, #[serde(default)] pub key: String, #[serde(default)] pub side: String,
    #[serde(default)] pub qty: f64, #[serde(default)] pub order_type: String, #[serde(default)] pub notional_usd: f64,
    #[serde(default)] pub expires_in_s: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TicketResult { #[serde(default)] pub status: String }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RiskInfo {
    #[serde(default)] pub assistant: AssistantCfg,
    #[serde(default)] pub markets_tradable: std::collections::BTreeMap<String, bool>,
    #[serde(default)] pub markets_probe: MarketsProbe,
    #[serde(default)] pub settings: RiskSettings,
    #[serde(default)] pub halted: bool,
    #[serde(default)] pub halt_reason: String,
    #[serde(default)] pub daily: DailyPnl,
    #[serde(default)] pub account: RiskAccount,
    #[serde(default)] pub exposure: Vec<Exposure>,
    #[serde(default)] pub open_positions: i64,
    #[serde(default)] pub working_orders: i64,
    #[serde(default)] pub active_agents: Vec<String>,
    #[serde(default)] pub mode: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditItem {
    #[serde(default)] pub ts: String,
    #[serde(default)] pub event: String,
    #[serde(flatten)] pub fields: serde_json::Map<String, serde_json::Value>,
}

#[cfg(not(feature = "demo"))]
#[get("/api/risk")]
pub async fn get_risk() -> Result<RiskInfo, ServerFnError> { http::get("/risk").await }

#[cfg(not(feature = "demo"))]
#[post("/api/risk/settings")]
pub async fn put_risk(settings: RiskSettings) -> Result<RiskInfo, ServerFnError> {
    http::send(reqwest::Method::PUT, "/risk", &settings).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/risk/kill")]
pub async fn kill_switch() -> Result<bool, ServerFnError> {
    #[derive(Deserialize)] struct R { halted: bool }
    let r: R = http::send(reqwest::Method::POST, "/risk/kill", &serde_json::json!({})).await?;
    Ok(r.halted)
}

/// Confirm today's daily-loss baseline (start NetLiq); None = accept the current NetLiq
#[cfg(not(feature = "demo"))]
#[post("/api/risk/daily_baseline")]
pub async fn confirm_daily_baseline(netliq: Option<f64>) -> Result<DailyPnl, ServerFnError> {
    http::send(reqwest::Method::POST, "/risk/daily_baseline", &match netliq { Some(n) => serde_json::json!({"netliq": n}), None => serde_json::json!({}) }).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/risk/resume")]
pub async fn resume_trading() -> Result<bool, ServerFnError> {
    #[derive(Deserialize)] struct R { halted: bool }
    let r: R = http::send(reqwest::Method::POST, "/risk/resume", &serde_json::json!({})).await?;
    Ok(r.halted)
}

#[cfg(not(feature = "demo"))]
#[get("/api/audit?limit")]
pub async fn get_audit(limit: i64) -> Result<Vec<AuditItem>, ServerFnError> {
    #[derive(Deserialize)] struct R { items: Vec<AuditItem> }
    let r: R = http::get(&format!("/audit?limit={limit}")).await?;
    Ok(r.items)
}

// ---------------- history / performance ----------------
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Trade {
    #[serde(default, deserialize_with = "null_def")] pub account: String,
    #[serde(default, deserialize_with = "null_def")] pub id: i64,
    #[serde(default, deserialize_with = "null_def")] pub key: String,
    #[serde(default, deserialize_with = "null_def")] pub symbol: String,
    #[serde(default, deserialize_with = "null_def")] pub source: String,
    #[serde(default, deserialize_with = "null_def")] pub node: String,
    #[serde(default, deserialize_with = "null_def")] pub strategy: String,
    #[serde(default, deserialize_with = "null_def")] pub side: String,
    #[serde(default, deserialize_with = "null_def")] pub entry_ts: String,
    #[serde(default)] pub exit_ts: Option<String>,
    #[serde(default, deserialize_with = "null_def")] pub qty: f64,
    #[serde(default)] pub entry_price: Option<f64>,
    #[serde(default)] pub exit_price: Option<f64>,
    /// null (with pnl_unknown) when a cross-pair P&L cannot be converted to USD
    #[serde(default)] pub pnl: Option<f64>,
    #[serde(default, deserialize_with = "null_def")] pub commission: f64,
    #[serde(default)] pub net_pnl: Option<f64>,
    #[serde(default, deserialize_with = "null_def")] pub pnl_unknown: i64,
    #[serde(default)] pub holding_sec: Option<f64>,
    #[serde(default, deserialize_with = "null_def")] pub n_fills: i64,
    #[serde(default, deserialize_with = "null_def")] pub first_exec: String,
    #[serde(default, deserialize_with = "null_def")] pub open: i64,
    #[serde(default, deserialize_with = "null_def")] pub note: String,
    #[serde(default, deserialize_with = "null_def")] pub tags: String,
    #[serde(default, deserialize_with = "null_def")] pub rating: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EquityPoint { pub ts: String, #[serde(default, deserialize_with = "null_def")] pub netliq: f64, #[serde(default, deserialize_with = "null_def")] pub unrealized: f64, #[serde(default, deserialize_with = "null_def")] pub realized: f64,
    /// "All accounts": some account has no data at this time and its first/last value is held flat
    #[serde(default)] pub partial: Option<bool> }

/// true when any point of an "All accounts" curve is flat-extended for some account
pub fn equity_partial(points: &[EquityPoint]) -> bool { points.iter().any(|p| p.partial == Some(true)) }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GroupStat { pub name: String, #[serde(default, deserialize_with = "null_def")] pub trades: i64, #[serde(default, deserialize_with = "null_def")] pub wins: i64, #[serde(default, deserialize_with = "null_def")] pub net_pnl: f64, #[serde(default, deserialize_with = "null_def")] pub win_rate: f64 }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct HourStat { pub hour: i64, #[serde(default, deserialize_with = "null_def")] pub trades: i64, #[serde(default, deserialize_with = "null_def")] pub net_pnl: f64 }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Stats {
    #[serde(default, deserialize_with = "null_def")] pub range: String,
    #[serde(default, deserialize_with = "null_def")] pub trades: i64, #[serde(default, deserialize_with = "null_def")] pub wins: i64, #[serde(default, deserialize_with = "null_def")] pub losses: i64, #[serde(default, deserialize_with = "null_def")] pub breakeven: i64,
    #[serde(default, deserialize_with = "null_def")] pub win_rate: f64, #[serde(default, deserialize_with = "null_def")] pub net_pnl: f64,
    #[serde(default, deserialize_with = "null_def")] pub gross_win: f64, #[serde(default, deserialize_with = "null_def")] pub gross_loss: f64,
    #[serde(default, deserialize_with = "null_def")] pub avg_win: f64, #[serde(default, deserialize_with = "null_def")] pub avg_loss: f64,
    #[serde(default, deserialize_with = "null_def")] pub profit_factor: f64, #[serde(default, deserialize_with = "null_def")] pub expectancy: f64,
    #[serde(default, deserialize_with = "null_def")] pub avg_holding_sec: f64, #[serde(default, deserialize_with = "null_def")] pub commission: f64,
    #[serde(default, deserialize_with = "null_def")] pub max_drawdown: f64, #[serde(default, deserialize_with = "null_def")] pub max_drawdown_pct: f64, #[serde(default, deserialize_with = "null_def")] pub sharpe: f64,
    #[serde(default, deserialize_with = "null_def")] pub equity_start: f64, #[serde(default, deserialize_with = "null_def")] pub equity_end: f64,
    #[serde(default, deserialize_with = "null_def")] pub by_symbol: Vec<GroupStat>, #[serde(default, deserialize_with = "null_def")] pub by_source: Vec<GroupStat>,
    #[serde(default, deserialize_with = "null_def")] pub by_weekday: Vec<GroupStat>, #[serde(default, deserialize_with = "null_def")] pub by_hour: Vec<HourStat>,
    #[serde(default, deserialize_with = "null_def")] pub open_trades: i64,
    #[serde(default, deserialize_with = "null_def")] pub unknown_pnl_trades: i64,
    #[serde(default)] pub equity_partial: Option<bool>,
}

#[cfg(not(feature = "demo"))]
#[get("/api/history/trades?range&key&source&account&tz")]
pub async fn get_trades(range: String, key: String, source: String, account: String, tz: String) -> Result<Vec<Trade>, ServerFnError> {
    #[derive(Deserialize)] struct R { trades: Vec<Trade> }
    let mut q = format!("/history/trades?range={range}&limit=500&account={}{}", http::enc(&account), http::tz_q(&tz));
    if !key.is_empty() { q.push_str(&format!("&key={}", http::enc(&key))); }
    if !source.is_empty() { q.push_str(&format!("&source={}", http::enc(&source))); }
    let r: R = http::get(&q).await?;
    Ok(r.trades)
}

#[cfg(not(feature = "demo"))]
#[get("/api/history/equity?range&account&tz")]
pub async fn get_equity(range: String, account: String, tz: String) -> Result<Vec<EquityPoint>, ServerFnError> {
    #[derive(Deserialize)] struct R { points: Vec<EquityPoint>, #[serde(default)] partial: Option<bool> }
    let mut r: R = http::get(&format!("/history/equity?range={range}&account={}{}", http::enc(&account), http::tz_q(&tz))).await?;
    // top-level flag without flagged points (older history.py): carry it on the first point so the UI still shows it
    if r.partial == Some(true) && !equity_partial(&r.points) { if let Some(p) = r.points.first_mut() { p.partial = Some(true); } }
    Ok(r.points)
}

#[cfg(not(feature = "demo"))]
#[get("/api/history/stats?range&account&source&tz")]
pub async fn get_stats(range: String, account: String, source: String, tz: String) -> Result<Stats, ServerFnError> {
    http::get(&format!("/history/stats?range={range}&account={}&source={}{}", http::enc(&account), http::enc(&source), http::tz_q(&tz))).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/history/journal")]
pub async fn set_journal(first_exec: String, note: String, tags: String, rating: i64) -> Result<bool, ServerFnError> {
    #[derive(Deserialize)] struct R { ok: bool }
    let r: R = http::send(reqwest::Method::PUT, &format!("/history/trades/{}/journal", http::enc(&first_exec)),
        &serde_json::json!({ "note": note, "tags": tags, "rating": rating })).await?;
    Ok(r.ok)
}

#[cfg(not(feature = "demo"))]
#[get("/api/history/export?what&range&account&tz")]
pub async fn get_export(what: String, range: String, account: String, tz: String) -> Result<String, ServerFnError> {
    let r = http::client()?.get(format!("{}/history/export?what={what}&range={range}&account={}{}", http::base(), http::enc(&account), http::tz_q(&tz))).send().await
        .map_err(|e| ServerFnError::new(format!("bot API not responding: {e}")))?;
    // an error page must never be saved as the CSV
    if !r.status().is_success() {
        let s = r.status();
        let body = r.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("{s}: {body}")));
    }
    r.text().await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[cfg(not(feature = "demo"))]
#[post("/api/history/sync")]
pub async fn sync_databricks() -> Result<serde_json::Value, ServerFnError> {
    // same operator-token client as every other call (the API answers 401 without it once operators exist)
    http::send_timeout(reqwest::Method::POST, "/history/sync_databricks", &serde_json::json!({}), 600).await
}

// ---------------- notifications ----------------
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NotifyCfg { #[serde(default)] pub enabled: bool, #[serde(default)] pub events: std::collections::BTreeMap<String, bool>, #[serde(default)] pub configured: bool }

#[cfg(not(feature = "demo"))]
#[get("/api/notify")]
pub async fn get_notify() -> Result<NotifyCfg, ServerFnError> { http::get("/notify").await }

#[cfg(not(feature = "demo"))]
#[post("/api/notify")]
pub async fn put_notify(cfg: NotifyCfg) -> Result<NotifyCfg, ServerFnError> {
    http::send(reqwest::Method::PUT, "/notify", &serde_json::json!({ "enabled": cfg.enabled, "events": cfg.events })).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/notify/test")]
pub async fn notify_test() -> Result<bool, ServerFnError> {
    #[derive(Deserialize)] struct R { ok: bool }
    let r: R = http::send(reqwest::Method::POST, "/notify/test", &serde_json::json!({})).await?;
    Ok(r.ok)
}

// ---------------- round 6: alerts, news, calendar, UI settings, health ----------------
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    #[serde(default)] pub id: String,
    #[serde(default)] pub key: String,
    #[serde(default)] pub kind: String,     // above | below | change_pct
    #[serde(default)] pub level: f64,
    #[serde(default)] pub note: String,
    #[serde(default)] pub repeat: bool,
    #[serde(default)] pub armed: bool,
    #[serde(default)] pub created: String,
    #[serde(default, alias = "triggered")] pub fired_at: Option<String>,
    #[serde(default)] pub last_price: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NewsItem {
    #[serde(default)] pub time: String,
    #[serde(default)] pub provider: String,
    #[serde(default)] pub article_id: String,
    #[serde(default)] pub headline: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CalItem {
    #[serde(default)] pub title: String,
    #[serde(default)] pub country: String,
    #[serde(default)] pub impact: String,
    #[serde(default)] pub date: String,
    #[serde(default)] pub forecast: String,
    #[serde(default)] pub previous: String,
    #[serde(default)] pub actual: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Calendar { #[serde(default)] pub items: Vec<CalItem>, #[serde(default)] pub news_pause_minutes: i64,
    /// the economic calendar has not been refreshed for > 6 h — the news pause works on old data
    #[serde(default)] pub calendar_stale: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSettings {
    #[serde(default = "d_wl")] pub wl_w: f64,
    #[serde(default = "d_tk")] pub ticket_w: f64,
    #[serde(default = "d_bh")] pub bottom_h: f64,
    #[serde(default = "d_one")] pub layout: usize,
    #[serde(default)] pub pane_key: Vec<String>,
    #[serde(default)] pub pane_tf: Vec<String>,
    #[serde(default = "d_true")] pub sound: bool,
    #[serde(default)] pub preset: String,
}
fn d_wl() -> f64 { 300.0 } fn d_tk() -> f64 { 280.0 } fn d_bh() -> f64 { 190.0 } fn d_one() -> usize { 1 } fn d_true() -> bool { true }
impl Default for UiSettings {
    fn default() -> Self { UiSettings { wl_w: 300.0, ticket_w: 280.0, bottom_h: 190.0, layout: 1, pane_key: vec![], pane_tf: vec![], sound: true, preset: "Trading".into() } }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConnInfo {
    #[serde(default)] pub state: String,
    #[serde(default)] pub reconnects: i64,
    #[serde(default)] pub last_error: String,
    #[serde(default)] pub last_reconcile: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Health {
    #[serde(default)] pub ok: bool,
    #[serde(default)] pub ib_connected: bool,
    #[serde(default)] pub mode: String,
    #[serde(default)] pub conn: ConnInfo,
    #[serde(default)] pub halted: bool,
    #[serde(default)] pub active_agents: Vec<String>,
    #[serde(default)] pub ws_clients: i64,
    #[serde(default)] pub version: String,
    #[serde(default)] pub time_utc: String,
}

#[cfg(not(feature = "demo"))]
#[get("/api/alerts")]
pub async fn get_alerts() -> Result<Vec<Alert>, ServerFnError> {
    #[derive(Deserialize)] struct R { #[serde(default)] alerts: Vec<Alert> }
    Ok(http::get::<R>("/alerts").await?.alerts)
}
#[cfg(not(feature = "demo"))]
#[post("/api/alerts/add")]
pub async fn add_alert(key: String, kind: String, level: f64, note: String, repeat: bool) -> Result<Alert, ServerFnError> {
    http::send(reqwest::Method::POST, "/alerts", &serde_json::json!({"key": key, "kind": kind, "level": level, "note": note, "repeat": repeat})).await
}
#[cfg(not(feature = "demo"))]
#[post("/api/alerts/delete")]
pub async fn delete_alert(id: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::DELETE, &format!("/alerts/{id}"), &serde_json::json!({})).await?;
    Ok(true)
}
#[cfg(not(feature = "demo"))]
#[get("/api/news?key")]
pub async fn get_news(key: String) -> Result<Vec<NewsItem>, ServerFnError> {
    #[derive(Deserialize)] struct R { #[serde(default)] items: Vec<NewsItem> }
    Ok(http::get::<R>(&format!("/news?key={}&limit=25", key)).await?.items)
}
#[cfg(not(feature = "demo"))]
#[get("/api/news/article?provider&article_id")]
pub async fn get_article(provider: String, article_id: String) -> Result<String, ServerFnError> {
    #[derive(Deserialize)] struct R { #[serde(default)] text: String }
    Ok(http::get::<R>(&format!("/news/article?provider={}&article_id={}", provider, article_id)).await?.text)
}
#[cfg(not(feature = "demo"))]
#[get("/api/calendar")]
pub async fn get_calendar() -> Result<Calendar, ServerFnError> { http::get("/calendar").await }
#[cfg(not(feature = "demo"))]
#[post("/api/calendar/pause")]
pub async fn set_news_pause(minutes: i64) -> Result<i64, ServerFnError> {
    let v: serde_json::Value = http::send(reqwest::Method::PUT, "/calendar/pause", &serde_json::json!({"minutes": minutes})).await?;
    Ok(v.get("news_pause_minutes").and_then(|x| x.as_i64()).unwrap_or(0))
}
#[cfg(not(feature = "demo"))]
#[get("/api/ui")]
pub async fn get_ui() -> Result<UiSettings, ServerFnError> { http::get("/ui").await }
#[cfg(not(feature = "demo"))]
#[post("/api/ui")]
pub async fn put_ui(s: UiSettings) -> Result<UiSettings, ServerFnError> { http::send(reqwest::Method::PUT, "/ui", &s).await }
#[cfg(not(feature = "demo"))]
#[get("/api/health")]
pub async fn get_health() -> Result<Health, ServerFnError> { http::get("/health").await }

// ---------------- setup / login (Interactive Brokers only) ----------------
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AcctSummary { #[serde(default)] pub account_id: String, #[serde(default)] pub net_liquidation: f64, #[serde(default)] pub currency: String }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SetupProgress { #[serde(default)] pub state: String, #[serde(default)] pub message: String }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SetupStatus {
    #[serde(default)] pub logged_in: bool,
    #[serde(default)] pub ib_connected: bool,
    #[serde(default)] pub gateway_running: bool,
    #[serde(default)] pub configured: bool,
    #[serde(default)] pub username_masked: String,
    #[serde(default)] pub mode: String,
    #[serde(default)] pub accounts: Vec<AcctSummary>,
    #[serde(default)] pub selected_account: String,
    #[serde(default)] pub setup: SetupProgress,
    #[serde(default)] pub broker: String,
    #[serde(default)] pub gateway_id: String,
    #[serde(default)] pub gateways: Vec<GatewayRow>,
    #[serde(default)] pub live_allowed: bool,
    #[serde(default)] pub live_code_set: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GatewayRow {
    #[serde(default)] pub id: String, #[serde(default)] pub name: String, #[serde(default)] pub username_masked: String,
    #[serde(default)] pub mode: String, #[serde(default)] pub api_port: i64, #[serde(default)] pub vnc_port: i64,
    #[serde(default)] pub running: bool, #[serde(default)] pub active: bool, #[serde(default)] pub last_login: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Operator { #[serde(default)] pub id: String, #[serde(default)] pub name: String, #[serde(default)] pub role: String, #[serde(default = "d_true")] pub active: bool, #[serde(default)] pub created: String }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OpStatus {
    #[serde(default)] pub configured: bool,
    #[serde(default)] pub operator: Option<Operator>,
    #[serde(default)] pub idle_sec: i64,
    #[serde(default)] pub live_code_set: bool,
    #[serde(default)] pub operators: Vec<Operator>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OpLoginResult { #[serde(default)] pub token: String, #[serde(default)] pub operator: Operator }

#[cfg(not(feature = "demo"))]
#[get("/api/op/status")]
pub async fn op_status() -> Result<OpStatus, ServerFnError> { http::get("/operators/status").await }
#[cfg(not(feature = "demo"))]
#[post("/api/op/login")]
pub async fn op_login(id: String, pin: String) -> Result<Operator, ServerFnError> {
    let r: OpLoginResult = http::send(reqwest::Method::POST, "/operators/login", &serde_json::json!({"id": id, "pin": pin})).await?;
    http::set_token(&r.token);
    Ok(r.operator)
}
#[cfg(not(feature = "demo"))]
#[post("/api/op/bootstrap")]
pub async fn op_bootstrap(name: String, pin: String) -> Result<Operator, ServerFnError> {
    let r: OpLoginResult = http::send(reqwest::Method::POST, "/operators/bootstrap", &serde_json::json!({"name": name, "role": "admin", "pin": pin})).await?;
    http::set_token(&r.token);
    Ok(r.operator)
}
#[cfg(not(feature = "demo"))]
#[post("/api/op/logout")]
pub async fn op_logout() -> Result<bool, ServerFnError> {
    let _: Result<serde_json::Value, _> = http::send(reqwest::Method::POST, "/operators/logout", &serde_json::json!({})).await;
    http::set_token("");
    Ok(true)
}
#[cfg(not(feature = "demo"))]
#[get("/api/op/list")]
pub async fn op_list() -> Result<Vec<Operator>, ServerFnError> {
    #[derive(Deserialize)] struct R { #[serde(default)] operators: Vec<Operator> }
    Ok(http::get::<R>("/operators").await?.operators)
}
#[cfg(not(feature = "demo"))]
#[post("/api/op/create")]
pub async fn op_create(name: String, role: String, pin: String) -> Result<Operator, ServerFnError> {
    http::send(reqwest::Method::POST, "/operators", &serde_json::json!({"name": name, "role": role, "pin": pin})).await
}
#[cfg(not(feature = "demo"))]
#[post("/api/op/update")]
pub async fn op_update(id: String, role: Option<String>, active: Option<bool>, pin: Option<String>) -> Result<Operator, ServerFnError> {
    http::send(reqwest::Method::PUT, &format!("/operators/{id}"), &serde_json::json!({"role": role, "active": active, "pin": pin})).await
}
#[cfg(not(feature = "demo"))]
#[post("/api/security/live_code")]
pub async fn set_live_code(code: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::PUT, "/security/live_code", &serde_json::json!({"code": code})).await?;
    Ok(true)
}
#[cfg(not(feature = "demo"))]
#[post("/api/setup/connect")]
pub async fn setup_connect(gateway_id: String, live_code: String) -> Result<bool, ServerFnError> {
    #[derive(Deserialize)] struct R { #[serde(default)] starting: bool }
    let r: R = http::send(reqwest::Method::POST, "/setup/connect", &serde_json::json!({"gateway_id": gateway_id, "live_code": if live_code.is_empty() { None } else { Some(live_code) }})).await?;
    Ok(r.starting)
}
#[cfg(not(feature = "demo"))]
#[post("/api/setup/gateway_stop")]
pub async fn gateway_stop(gateway_id: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::POST, &format!("/setup/gateways/{gateway_id}/stop"), &serde_json::json!({})).await?;
    Ok(true)
}
#[cfg(not(feature = "demo"))]
#[post("/api/setup/gateway_remove")]
pub async fn gateway_remove(gateway_id: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::DELETE, &format!("/setup/gateways/{gateway_id}"), &serde_json::json!({})).await?;
    Ok(true)
}

#[cfg(not(feature = "demo"))]
#[get("/api/setup/status")]
pub async fn get_setup_status() -> Result<SetupStatus, ServerFnError> { http::get("/setup/status").await }
#[cfg(not(feature = "demo"))]
#[post("/api/setup/login")]
pub async fn setup_login(username: String, password: String, mode: String, live_code: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::POST, "/setup/login", &serde_json::json!({"username": username, "password": password, "mode": mode,
        "live_code": if live_code.is_empty() { None } else { Some(live_code) }})).await?;
    Ok(true)
}
#[cfg(not(feature = "demo"))]
#[post("/api/setup/account")]
pub async fn setup_account(account_id: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::POST, "/setup/account", &serde_json::json!({"account_id": account_id})).await?;
    Ok(true)
}
#[cfg(not(feature = "demo"))]
#[post("/api/setup/logout")]
pub async fn setup_logout(stop_gateway: bool) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::POST, &format!("/setup/logout?stop_gateway={stop_gateway}"), &serde_json::json!({})).await?;
    Ok(true)
}

/// AI control in the top bar: read-only (write_enabled off) · ask first (on, auto_confirm off) · auto (both on)
#[cfg(not(feature = "demo"))]
#[post("/api/assistant/mode")]
pub async fn set_assistant_mode(write_enabled: bool, auto_confirm: bool) -> Result<AssistantCfg, ServerFnError> {
    http::send(reqwest::Method::PUT, "/assistant", &serde_json::json!({"write_enabled": write_enabled, "auto_confirm": auto_confirm})).await
}

#[cfg(not(feature = "demo"))]
#[get("/api/assistant/tickets")]
pub async fn get_assistant_tickets() -> Result<Vec<AssistantTicket>, ServerFnError> {
    #[derive(Deserialize)] struct R { #[serde(default)] tickets: Vec<AssistantTicket> }
    Ok(http::get::<R>("/assistant/tickets").await?.tickets)
}

#[cfg(not(feature = "demo"))]
#[post("/api/assistant/ticket")]
pub async fn decide_assistant_ticket(token: String, approve: bool) -> Result<TicketResult, ServerFnError> {
    http::send(reqwest::Method::POST, &format!("/assistant/tickets/{token}/{}", if approve { "approve" } else { "reject" }), &serde_json::json!({})).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/assistant")]
pub async fn put_assistant(write_enabled: Option<bool>, live: Option<bool>, max_order_notional_usd: Option<f64>, max_orders_per_hour: Option<i64>, allow_short: Option<bool>, allow_agents: Option<bool>) -> Result<AssistantCfg, ServerFnError> {
    http::send(reqwest::Method::PUT, "/assistant", &serde_json::json!({"write_enabled": write_enabled, "live": live, "max_order_notional_usd": max_order_notional_usd,
        "max_orders_per_hour": max_orders_per_hour, "allow_short": allow_short, "allow_agents": allow_agents})).await
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProbeResult { #[serde(default)] pub market: String, #[serde(default)] pub allowed: bool, #[serde(default)] pub reason: String }
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MarketsProbe { #[serde(default)] pub ts: String, #[serde(default)] pub account: String, #[serde(default)] pub results: Vec<ProbeResult> }

#[cfg(not(feature = "demo"))]
#[post("/api/risk/markets/probe")]
pub async fn probe_markets() -> Result<Vec<ProbeResult>, ServerFnError> {
    #[derive(Deserialize)] struct R { #[serde(default)] results: Vec<ProbeResult> }
    Ok(http::send::<_, R>(reqwest::Method::POST, "/risk/markets/probe?force=true", &serde_json::json!({})).await?.results)
}

#[cfg(not(feature = "demo"))]
#[post("/api/risk/markets")]
pub async fn set_market_tradable(market: String, tradable: bool) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::PUT, "/risk/markets", &serde_json::json!({"market": market, "tradable": tradable})).await?;
    Ok(true)
}

// ---------------- Nautilus backtest reports (migration step 5) ----------------
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BtReport {
    #[serde(default)] pub kind: String,          // backtest | parity | walkforward
    #[serde(default)] pub file: String,
    #[serde(default)] pub created: String,
    #[serde(default)] pub strategy: String,
    #[serde(default)] pub instrument: String,
    #[serde(default)] pub tf_minutes: Option<i64>,
    #[serde(default)] pub start: Option<String>,
    #[serde(default)] pub end: Option<String>,
    #[serde(default)] pub label: String,
    #[serde(default)] pub pnl: Option<f64>,
    #[serde(default)] pub trades: Option<i64>,
}

#[cfg(not(feature = "demo"))]
#[get("/api/backtests")]
pub async fn get_bt_reports() -> Result<Vec<BtReport>, ServerFnError> {
    #[derive(Deserialize)] struct R { items: Vec<BtReport> }
    Ok(http::get::<R>("/backtests?limit=150").await?.items)
}

/// Full report as JSON text (structure varies by kind — the UI reads it as serde_json::Value)
#[cfg(not(feature = "demo"))]
#[get("/api/backtests/report?kind&file")]
pub async fn get_bt_report(kind: String, file: String) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get(&format!("/backtests/{}/{}", http::enc(&kind), http::enc(&file))).await?;
    Ok(v.to_string())
}

// ---------------- Nautilus node manager (migration step 6) ----------------
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeInfo {
    #[serde(default)] pub id: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub config_file: String,
    #[serde(default)] pub overrides: serde_json::Value,
    #[serde(default)] pub auto_restart: bool,
    #[serde(default)] pub desired: bool,
    #[serde(default)] pub state: String,         // running | starting | degraded | backoff | crashed | stopped
    #[serde(default)] pub running: bool,
    #[serde(default)] pub pid: Option<i64>,
    #[serde(default)] pub uptime_s: Option<i64>,
    #[serde(default)] pub restarts_15m: i64,
    #[serde(default)] pub last_exit: Option<i64>,
    #[serde(default)] pub reason: String,
    #[serde(default)] pub client_id: Option<i64>,
    #[serde(default)] pub gateway: serde_json::Value,
    #[serde(default)] pub config: serde_json::Value,
    #[serde(default)] pub health: serde_json::Value,
    #[serde(default)] pub live: serde_json::Value,   // from the event bus: positions, last_signal, heartbeat_age_s
    #[serde(default)] pub summary: String,           // e.g. "EMA 10/30", "Donchian 20/10"
    #[serde(default)] pub strategy_ref: Option<String>,     // name@version this process is running
    #[serde(default)] pub strategy_latest: Option<String>,  // name@version a restart would use now
    #[serde(default)] pub allow_live: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeConfigFile {
    #[serde(default)] pub file: String,
    #[serde(default)] pub strategy: String,
    #[serde(default)] pub instrument: String,
    #[serde(default)] pub tf_minutes: Option<i64>,
    #[serde(default)] pub params: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodesResp {
    #[serde(default)] pub nodes: Vec<NodeInfo>,
    #[serde(default)] pub configs: Vec<NodeConfigFile>,
    #[serde(default)] pub gateway: serde_json::Value,
}

#[cfg(not(feature = "demo"))]
#[get("/api/nodes")]
pub async fn get_nodes() -> Result<NodesResp, ServerFnError> { http::get("/nodes").await }

/// Create (empty id) or edit a node — overrides is a JSON object as text
#[cfg(not(feature = "demo"))]
#[post("/api/nodes/save")]
pub async fn save_node(id: String, name: String, config_file: String, overrides: String, auto_restart: bool) -> Result<NodeInfo, ServerFnError> {
    let ov: serde_json::Value = serde_json::from_str(&overrides).unwrap_or(serde_json::json!({}));
    if id.is_empty() {
        http::send(reqwest::Method::POST, "/nodes", &serde_json::json!({"name": name, "config_file": config_file, "overrides": ov, "auto_restart": auto_restart})).await
    } else {
        http::send(reqwest::Method::PUT, &format!("/nodes/{}", http::enc(&id)), &serde_json::json!({"name": name, "config_file": config_file, "overrides": ov, "auto_restart": auto_restart})).await
    }
}

/// action: start | stop | restart | delete
#[cfg(not(feature = "demo"))]
#[post("/api/nodes/action")]
pub async fn node_action(id: String, action: String) -> Result<String, ServerFnError> {
    let path = format!("/nodes/{}", http::enc(&id));
    let v: serde_json::Value = match action.as_str() {
        "delete" => http::send(reqwest::Method::DELETE, &path, &serde_json::json!({})).await?,
        "start" | "stop" | "restart" => http::send_slow(reqwest::Method::POST, &format!("{path}/{action}"), &serde_json::json!({})).await?,
        _ => return Err(ServerFnError::new("unknown action")),
    };
    Ok(v.get("state").and_then(|s| s.as_str()).unwrap_or("ok").to_string())
}

#[cfg(not(feature = "demo"))]
#[get("/api/nodes/log?id&level")]
pub async fn get_node_log(id: String, level: String) -> Result<Vec<String>, ServerFnError> {
    #[derive(Deserialize)] struct R { lines: Vec<String> }
    Ok(http::get::<R>(&format!("/nodes/{}/log?lines=400&level={}", http::enc(&id), http::enc(&level))).await?.lines)
}

/// latest events from Nautilus nodes (JSON: {"events": [...], "status": {...}})
#[cfg(not(feature = "demo"))]
#[get("/api/events?node&types")]
pub async fn get_node_events(node: String, types: String) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get(&format!("/events?limit=200&node={}&types={}", http::enc(&node), http::enc(&types))).await?;
    Ok(v.to_string())
}

// ---------------- Strategy lab (migration step 9) — JSON text in/out (flexible structure) ----------------
#[cfg(not(feature = "demo"))]
#[get("/api/strategies")]
pub async fn get_strategies() -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get("/strategies").await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/strategies/save")]
pub async fn save_strategy(file: String, config: String, overwrite: bool) -> Result<String, ServerFnError> {
    let cfg: serde_json::Value = serde_json::from_str(&config).map_err(|e| ServerFnError::new(e.to_string()))?;
    let v: serde_json::Value = http::send(reqwest::Method::PUT, &format!("/strategies/{}", http::enc(&file)), &serde_json::json!({"config": cfg, "overwrite": overwrite})).await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/strategies/delete")]
pub async fn delete_strategy(file: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::DELETE, &format!("/strategies/{}", http::enc(&file)), &serde_json::json!({})).await?;
    Ok(true)
}

/// kind: backtest | parity | walkforward · opts: {"days":90,"train":90,"test":30}
#[cfg(not(feature = "demo"))]
#[post("/api/strategies/run")]
pub async fn run_strategy_job(file: String, kind: String, opts: String) -> Result<String, ServerFnError> {
    let mut body: serde_json::Value = serde_json::from_str(&opts).unwrap_or(serde_json::json!({}));
    body["kind"] = serde_json::json!(kind);
    let v: serde_json::Value = http::send(reqwest::Method::POST, &format!("/strategies/{}/run", http::enc(&file)), &body).await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[get("/api/strategies/job?id")]
pub async fn get_strategy_job(id: String) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get(&format!("/strategies/jobs/{}", http::enc(&id))).await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/strategies/job/cancel")]
pub async fn cancel_strategy_job(id: String) -> Result<bool, ServerFnError> {
    let _: serde_json::Value = http::send(reqwest::Method::POST, &format!("/strategies/jobs/{}/cancel", http::enc(&id)), &serde_json::json!({})).await?;
    Ok(true)
}

#[cfg(not(feature = "demo"))]
#[get("/api/strategies/deploy_check?file")]
pub async fn get_deploy_check(file: String) -> Result<Vec<String>, ServerFnError> {
    #[derive(Deserialize)] struct R { warnings: Vec<String> }
    Ok(http::get::<R>(&format!("/strategies/{}/deploy_check", http::enc(&file))).await?.warnings)
}

#[cfg(not(feature = "demo"))]
#[post("/api/strategies/deploy")]
pub async fn deploy_strategy(file: String, name: String, auto_restart: bool, start: bool, force: bool) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send_slow(reqwest::Method::POST, &format!("/strategies/{}/deploy", http::enc(&file)),
        &serde_json::json!({"name": name, "auto_restart": auto_restart, "start": start, "force": force})).await?;
    Ok(v.to_string())
}

// ---------------- strategy packages + lifecycle (steps 19–21) — JSON text ----------------
#[cfg(not(feature = "demo"))]
#[get("/api/packages")]
pub async fn get_packages() -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get_slow("/strategies/packages").await?;
    Ok(v.to_string())
}

/// source: a folder containing strategy.py, a strategy.py file or a git url → Strategy lab job (qos install)
#[cfg(not(feature = "demo"))]
#[post("/api/packages/install")]
pub async fn install_package(source: String, force: bool) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send(reqwest::Method::POST, "/strategies/packages/install", &serde_json::json!({"source": source, "force": force})).await?;
    Ok(v.to_string())
}

/// opts: {"only":"backtest"|"walkforward","days":90,"wf_days":365,"train":90,"test":30}
#[cfg(not(feature = "demo"))]
#[post("/api/packages/evaluate")]
pub async fn evaluate_package(pkg: String, opts: String) -> Result<String, ServerFnError> {
    let body: serde_json::Value = serde_json::from_str(&opts).unwrap_or(serde_json::json!({}));
    let v: serde_json::Value = http::send(reqwest::Method::POST, &format!("/strategies/packages/{}/evaluate", http::enc(&pkg)), &body).await?;
    Ok(v.to_string())
}

/// body: {"name","instrument","overrides":{…},"auto_restart","start"}
#[cfg(not(feature = "demo"))]
#[post("/api/packages/deploy")]
pub async fn deploy_package(pkg: String, body: String) -> Result<String, ServerFnError> {
    let b: serde_json::Value = serde_json::from_str(&body).map_err(|e| ServerFnError::new(e.to_string()))?;
    let v: serde_json::Value = http::send_slow(reqwest::Method::POST, &format!("/strategies/packages/{}/deploy", http::enc(&pkg)), &b).await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/nodes/allow_live")]
pub async fn node_allow_live(id: String, enabled: bool, live_code: String) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send(reqwest::Method::POST, &format!("/nodes/{}/allow_live", http::enc(&id)),
        &serde_json::json!({"enabled": enabled, "live_code": live_code})).await?;
    Ok(v.to_string())
}

/// Step 14: action = advance | demote | resume | reset
#[cfg(not(feature = "demo"))]
#[post("/api/nodes/ramp")]
pub async fn node_ramp_action(id: String, action: String, live_code: String, force: bool) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send(reqwest::Method::POST, &format!("/nodes/{}/ramp/{}", http::enc(&id), http::enc(&action)),
        &serde_json::json!({"live_code": live_code, "force": force})).await?;
    Ok(v.to_string())
}

// ---------------- portfolio risk (step 16) ----------------
#[cfg(not(feature = "demo"))]
#[get("/api/portfolio/risk")]
pub async fn get_portfolio_risk() -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get("/portfolio/risk").await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/portfolio/limits")]
pub async fn put_portfolio_limits(body: String) -> Result<String, ServerFnError> {
    let b: serde_json::Value = serde_json::from_str(&body).map_err(|e| ServerFnError::new(e.to_string()))?;
    let v: serde_json::Value = http::send(reqwest::Method::PUT, "/portfolio/limits", &b).await?;
    Ok(v.to_string())
}

// ---------------- system health (step 15) ----------------
#[cfg(not(feature = "demo"))]
#[get("/api/system/health")]
pub async fn get_system_health() -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get("/system/health").await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/system/backup")]
pub async fn run_backup() -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send_timeout(reqwest::Method::POST, "/system/backup/run", &serde_json::json!({}), 300).await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/nodes/release_hold")]
pub async fn release_hold(id: String) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send(reqwest::Method::POST, &format!("/nodes/{}/release_hold", http::enc(&id)), &serde_json::json!({})).await?;
    Ok(v.to_string())
}

// ---------------- QOS Cloud connector (step 34) ----------------
#[cfg(not(feature = "demo"))]
#[get("/api/cloud/status")]
pub async fn get_cloud_status() -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get("/cloud/status").await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/cloud/action")]
pub async fn cloud_action(what: String, arg: String) -> Result<String, ServerFnError> {
    let empty = serde_json::json!({});
    let v: serde_json::Value = match what.as_str() {
        "login" => http::send_timeout(reqwest::Method::POST, "/cloud/login", &serde_json::json!({"force": arg == "force"}), 30).await?,
        "cancel" => http::send(reqwest::Method::DELETE, "/cloud/login", &empty).await?,
        "open" => http::send(reqwest::Method::POST, "/cloud/login/open", &empty).await?,
        "logout" => http::send_timeout(reqwest::Method::POST, "/cloud/logout", &empty, 30).await?,
        "sync" => http::send_timeout(reqwest::Method::POST, "/cloud/sync", &empty, 60).await?,
        "devices" => http::get("/cloud/devices").await?,
        "remove" => http::send_timeout(reqwest::Method::DELETE, &format!("/cloud/devices/{}", http::enc(&arg)), &empty, 30).await?,
        "datasets" => http::send_timeout(reqwest::Method::GET, "/cloud/datasets", &empty, 30).await?,
        "pull" => http::send_timeout(reqwest::Method::POST, &format!("/cloud/datasets/{}/pull", http::enc(&arg)), &empty, 900).await?,
        "drop" => http::send_timeout(reqwest::Method::DELETE, &format!("/cloud/datasets/{}", http::enc(&arg)), &empty, 60).await?,
        _ => return Err(ServerFnError::new(format!("unknown cloud action {what}"))),
    };
    Ok(v.to_string())
}

// ---------------- QOS Store (step 38) ----------------
#[cfg(not(feature = "demo"))]
#[get("/api/cloud/store")]
pub async fn get_store() -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send_timeout(reqwest::Method::GET, "/cloud/store", &serde_json::json!({}), 30).await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[get("/api/cloud/store/detail?slug")]
pub async fn get_store_detail(slug: String) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::send_timeout(reqwest::Method::GET, &format!("/cloud/store/{}", http::enc(&slug)), &serde_json::json!({}), 30).await?;
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[post("/api/cloud/store/install")]
pub async fn store_install(slug: String, version: String) -> Result<String, ServerFnError> {
    let body = if version.is_empty() { serde_json::json!({}) } else { serde_json::json!({"version": version}) };
    let v: serde_json::Value = http::send_timeout(reqwest::Method::POST, &format!("/cloud/store/{}/install", http::enc(&slug)), &body, 300).await?;
    Ok(v.to_string())
}

// ---------------- payments (step 40) ----------------
#[cfg(not(feature = "demo"))]
#[post("/api/cloud/billing")]
pub async fn billing_action(what: String, arg: String) -> Result<String, ServerFnError> {
    let empty = serde_json::json!({});
    let v: serde_json::Value = match what.as_str() {
        "checkout" => http::send_timeout(reqwest::Method::POST, "/cloud/billing/checkout", &serde_json::json!({"price_id": arg}), 60).await?,
        "portal" => http::send_timeout(reqwest::Method::POST, "/cloud/billing/portal", &empty, 60).await?,
        "status" => http::send_timeout(reqwest::Method::GET, "/cloud/billing", &empty, 30).await?,
        "purchase" => http::send_timeout(reqwest::Method::GET, &format!("/cloud/billing/purchase/{}", http::enc(&arg)), &empty, 30).await?,
        _ => return Err(ServerFnError::new(format!("unknown billing action {what}"))),
    };
    Ok(v.to_string())
}

#[cfg(not(feature = "demo"))]
#[get("/api/live/readiness?node")]
pub async fn get_live_readiness(node: String) -> Result<String, ServerFnError> {
    let v: serde_json::Value = http::get(&format!("/live/readiness?node={}", http::enc(&node))).await?;
    Ok(v.to_string())
}

// ---------------- paper soak + shadow parity (step 12) ----------------
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SoakCheck {
    #[serde(default)] pub name: String,
    #[serde(default)] pub value: serde_json::Value,
    #[serde(default)] pub status: String,
    #[serde(default)] pub target: String,
    #[serde(default)] pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SoakDay {
    #[serde(default)] pub day: String,
    #[serde(default)] pub alive_min: i64,
    #[serde(default)] pub signals: i64,
    #[serde(default)] pub actions: i64,
    #[serde(default)] pub fills: i64,
    #[serde(default)] pub rejects: i64,
    #[serde(default)] pub denied: i64,
    #[serde(default)] pub gaps: i64,
    #[serde(default)] pub crashes: i64,
    #[serde(default)] pub reconcile: i64,
    #[serde(default)] pub avg_slippage_pips: Option<f64>,
    #[serde(default)] pub p95_latency_s: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SoakNode {
    #[serde(default)] pub node: String,
    #[serde(default)] pub verdict: String,
    #[serde(default)] pub since: Option<String>,
    #[serde(default)] pub days: i64,
    #[serde(default)] pub totals: serde_json::Value,
    #[serde(default)] pub avg_slippage_pips: Option<f64>,
    #[serde(default)] pub p95_latency_s: Option<f64>,
    #[serde(default)] pub decision_match: Option<f64>,
    #[serde(default)] pub parity_ts: Option<String>,
    #[serde(default)] pub checks: Vec<SoakCheck>,
    #[serde(default)] pub note: String,
    #[serde(default)] pub daily: Vec<SoakDay>,
    #[serde(default)] pub incidents: Vec<serde_json::Value>,
    #[serde(default)] pub executions: Vec<serde_json::Value>,
    #[serde(default)] pub parity: serde_json::Value,
    #[serde(default)] pub job: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SoakResp {
    #[serde(default)] pub criteria: serde_json::Value,
    #[serde(default)] pub nodes: Vec<SoakNode>,
    #[serde(default)] pub jobs: serde_json::Value,
}

#[cfg(not(feature = "demo"))]
#[get("/api/soak")]
pub async fn get_soak() -> Result<SoakResp, ServerFnError> { http::get("/soak").await }

#[cfg(not(feature = "demo"))]
#[get("/api/soak/detail?id")]
pub async fn get_soak_detail(id: String) -> Result<SoakNode, ServerFnError> {
    http::get(&format!("/soak/{}", http::enc(&id))).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/soak/parity")]
pub async fn soak_parity(id: String) -> Result<serde_json::Value, ServerFnError> {
    http::send_timeout(reqwest::Method::POST, &format!("/soak/{}/parity", http::enc(&id)), &serde_json::json!({}), 600).await
}

#[cfg(not(feature = "demo"))]
#[post("/api/soak/reset")]
pub async fn soak_reset(id: String) -> Result<serde_json::Value, ServerFnError> {
    http::send(reqwest::Method::POST, &format!("/soak/{}/reset", http::enc(&id)), &serde_json::json!({})).await
}

// ---------------- agent detail (node summary) ----------------
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CurvePt { #[serde(default)] pub t: String, #[serde(default)] pub v: f64 }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NodeSummary {
    #[serde(default)] pub node: NodeInfo,
    #[serde(default)] pub account: String,
    #[serde(default)] pub range: String,
    #[serde(default)] pub stats: Stats,
    #[serde(default)] pub trades: Vec<Trade>,
    #[serde(default)] pub curve: Vec<CurvePt>,
    #[serde(default)] pub research: serde_json::Value,
    #[serde(default)] pub live_perf: PerfBlock,
    #[serde(default)] pub expected_perf: Option<ExpectedPerf>,
    #[serde(default)] pub soak: SoakNode,
    #[serde(default)] pub events: Vec<serde_json::Value>,
    #[serde(default)] pub package: serde_json::Value,     // {ref, state, note} of the package the node uses (step 21)
    #[serde(default)] pub ramp: serde_json::Value,        // step 14: live ramp-up stage + criteria (null = none)
}

#[cfg(not(feature = "demo"))]
#[get("/api/nodes/summary?id&range&tz")]
pub async fn get_node_summary(id: String, range: String, tz: String) -> Result<NodeSummary, ServerFnError> {
    http::get(&format!("/nodes/{}/summary?range={}{}", http::enc(&id), http::enc(&range), http::tz_q(&tz))).await
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DayPnl { #[serde(default)] pub d: String, #[serde(default)] pub pnl: f64, #[serde(default)] pub n: i64 }

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PerfMetrics {
    #[serde(default)] pub net: f64, #[serde(default)] pub return_pct: f64, #[serde(default)] pub trades: f64,
    #[serde(default)] pub win_rate: f64, #[serde(default)] pub profit_factor: f64, #[serde(default)] pub expectancy: f64,
    #[serde(default)] pub avg_win: f64, #[serde(default)] pub avg_loss: f64, #[serde(default)] pub best_trade: f64, #[serde(default)] pub worst_trade: f64,
    #[serde(default)] pub max_drawdown: f64, #[serde(default)] pub max_drawdown_pct: f64, #[serde(default)] pub longest_drawdown_days: f64,
    #[serde(default)] pub days: f64, #[serde(default)] pub active_days: f64, #[serde(default)] pub positive_days_pct: f64,
    #[serde(default)] pub best_day: f64, #[serde(default)] pub worst_day: f64, #[serde(default)] pub sharpe: f64,
    #[serde(default)] pub from: String, #[serde(default)] pub to: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PerfBlock {
    #[serde(default)] pub base: f64,
    #[serde(default)] pub curve: Vec<(String, f64)>,
    #[serde(default)] pub daily: Vec<DayPnl>,
    #[serde(default)] pub monthly: Vec<MonthPnl>,
    #[serde(default)] pub trade_pnl: Vec<f64>,
    #[serde(default)] pub metrics: PerfMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MonthPnl {
    #[serde(default)] pub m: String, #[serde(default)] pub pnl: f64, #[serde(default)] pub n: i64, #[serde(default)] pub return_pct: f64,
    #[serde(default)] pub positive_days: i64, #[serde(default)] pub active_days: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ExpectedPerf {
    #[serde(default)] pub from: String,
    #[serde(default)] pub to: String,
    #[serde(default)] pub ts: String,
    #[serde(flatten)] pub perf: PerfBlock,
}

// web demo (../demo): the same set of functions, calling the demo backend instead of server functions
#[cfg(feature = "demo")]
include!("api_demo.rs");
