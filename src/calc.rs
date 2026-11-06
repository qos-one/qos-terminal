//! Pure calculations used by the Terminal UI: number formatting, currency conversion to USD for the
//! order ticket, tick rounding, and form validation. No Dioxus here, so it is unit-tested directly
//! (`cargo test calc`) and by qa/terminal.
//!
//! Currency conversion mirrors tr/bot/qx_strategies/fx.py: `rates` maps a currency to the USD value of
//! one unit of it, and every function returns None when it cannot price — callers must show "—" and
//! must not size or risk-check on a guess.
#![allow(dead_code)]
use std::collections::HashMap;

pub type Rates = HashMap<String, f64>;

/// serde: a JSON null becomes the type's default (one null field must never reject a whole snapshot / list)
pub fn null_def<'de, D: serde::Deserializer<'de>, T: Default + serde::Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    Ok(<Option<T> as serde::Deserialize>::deserialize(d)?.unwrap_or_default())
}

// ---------------- formatting ----------------

fn ok_pos(x: f64) -> bool { x.is_finite() && x > 0.0 }

/// Price with `dec` decimals; "—" for missing (≤ 0) or non-finite values.
pub fn px(v: f64, dec: usize) -> String {
    if !ok_pos(v) { return "—".into(); }
    format!("{:.*}", dec, v)
}

/// Money with thousands separators and 2 decimals. Rounds to cents first (so 2.999 → "3.00"),
/// never shows "-0.00", and shows "—" for NaN / infinity / IB's unset value (1.797e308).
pub fn money(v: f64) -> String {
    if !v.is_finite() || v.abs() >= 1e15 { return "—".into(); }
    let cents = (v.abs() * 100.0).round() as i64;
    let neg = v < 0.0 && cents != 0;
    let mut s = (cents / 100).to_string();
    let frac = cents % 100;
    let mut out = String::new();
    while s.len() > 3 {
        let tail = s.split_off(s.len() - 3);
        out = format!(",{tail}{out}");
    }
    format!("{}{s}{out}.{frac:02}", if neg { "-" } else { "" })
}

/// Money that is USD when `usd`, otherwise in `ccy` — labelled so it is never read as USD ("12,345.00 JPY").
pub fn money_in(v: f64, usd: bool, ccy: &str) -> String {
    if usd || ccy.is_empty() || ccy.eq_ignore_ascii_case("USD") { money(v) } else { format!("{} {}", money(v), ccy.to_uppercase()) }
}

/// Sum of the USD values only, plus how many rows were left out because they are in another currency.
pub fn usd_total<'a>(rows: impl IntoIterator<Item = (f64, bool)>) -> (f64, usize) {
    let (mut t, mut skipped) = (0.0, 0);
    for (v, usd) in rows { if usd && v.is_finite() { t += v } else { skipped += 1 } }
    (t, skipped)
}

/// Quantity: thousands separators for whole numbers, up to 8 decimals otherwise (no float noise).
pub fn qty_fmt(v: f64) -> String {
    if !v.is_finite() { return "—".into(); }
    let r = (v * 1e8).round() / 1e8;
    if r.fract() == 0.0 {
        let s = money(r);
        return s.trim_end_matches("00").trim_end_matches('.').to_string();
    }
    let s = format!("{r:.8}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    // thousands separators on the whole part too ("1,234.5")
    let (int_part, frac) = s.split_once('.').unwrap_or((s, ""));
    let whole = qty_fmt(int_part.parse::<f64>().unwrap_or(0.0).abs());
    let neg = if r < 0.0 { "-" } else { "" };
    if frac.is_empty() { format!("{neg}{whole}") } else { format!("{neg}{whole}.{frac}") }
}

/// Colour class for a P&L value, consistent with what `money` displays (|v| < 0.005 → neutral).
pub fn pnl_class(v: f64) -> &'static str {
    if !v.is_finite() || v.abs() < 0.005 { "text-neutral" } else if v > 0.0 { "text-up" } else { "text-down" }
}

/// Day change in percent, or "—" when there is no previous close to compare with.
pub fn chg_pct(change_pct: f64, prev_close: f64) -> String {
    if !ok_pos(prev_close) || !change_pct.is_finite() { return "—".into(); }
    format!("{change_pct:+.2}%")
}

/// "12.5%" for a fraction 0.125; "—" when missing / non-finite.
pub fn frac_pct(v: Option<f64>, dec: usize) -> String {
    match v { Some(x) if x.is_finite() => format!("{:.*}%", dec, x * 100.0), _ => "—".into() }
}

/// Share of a base in percent for chart axis labels; empty when the base is not positive.
pub fn pct_of(v: f64, base: f64) -> String {
    if !ok_pos(base) || !v.is_finite() { return String::new(); }
    format!(" ({:+.2}%)", v / base * 100.0)
}

/// Display decimals for an instrument key, same rule as server.py decimals_for (used when no quote is loaded).
pub fn decimals_for_key(key: &str) -> usize {
    let (mkt, sym) = key.split_once(':').unwrap_or(("FX", key));
    match mkt {
        "FX" => if sym.ends_with("JPY") { 3 } else { 5 },
        "CRYPTO" => if sym == "BTC" { 0 } else { 2 },
        _ => 2,
    }
}

/// Round a price to the instrument's displayed decimals (the price grid the Terminal shows and IB accepts for FX/stocks).
pub fn round_to_dec(price: f64, dec: usize) -> f64 {
    let f = 10f64.powi(dec as i32);
    (price * f).round() / f
}

/// IB's "unset" doubles (1.797e308) and NaN are not values.
pub fn finite_opt(v: Option<f64>) -> Option<f64> {
    v.filter(|x| x.is_finite() && x.abs() < 1e15)
}

// ---------------- currency conversion (port of qx_strategies/fx.py) ----------------

/// "EURUSD" | "EUR/USD" | "FX:EURUSD" → ("EUR","USD"); non-FX symbols → (sym, "USD").
pub fn split_pair(sym: &str) -> (String, String) {
    let s = sym.rsplit(':').next().unwrap_or(sym).split('.').next().unwrap_or("").to_uppercase();
    if let Some((b, q)) = s.split_once('/') { return (b.to_string(), if q.is_empty() { "USD".into() } else { q.to_string() }); }
    if s.len() == 6 && s.chars().all(|c| c.is_ascii_alphabetic()) { return (s[..3].to_string(), s[3..].to_string()); }
    (s, "USD".into())
}

pub fn usd_rate(ccy: &str, rates: &Rates) -> Option<f64> {
    if ccy.eq_ignore_ascii_case("USD") { return Some(1.0); }
    rates.get(&ccy.to_uppercase()).copied().filter(|r| ok_pos(*r))
}

/// Rates from FX mid prices the Terminal already has: {"EURUSD": 1.08, "USDJPY": 150.0} → {EUR: 1.08, JPY: 1/150, USD: 1}.
pub fn rates_from_prices<'a>(prices: impl IntoIterator<Item = (&'a str, f64)>) -> Rates {
    let mut out: Rates = HashMap::new();
    out.insert("USD".into(), 1.0);
    for (pair, p) in prices {
        if !ok_pos(p) { continue; }
        let (b, q) = split_pair(pair);
        if b.len() != 3 || q.len() != 3 { continue; }
        if q == "USD" && b != "USD" { out.insert(b, p); } else if b == "USD" && q != "USD" { out.insert(q, 1.0 / p); }
    }
    out
}

/// USD value of one unit of the pair's quote currency (multiply quote-currency P&L by this).
pub fn quote_to_usd(sym: &str, price: f64, rates: &Rates) -> Option<f64> {
    let (b, q) = split_pair(sym);
    if q == "USD" { return Some(1.0); }
    if b == "USD" { return if ok_pos(price) { Some(1.0 / price) } else { usd_rate(&q, rates) }; }
    usd_rate(&q, rates)
}

/// USD value of one unit of the base currency (or one share / coin).
pub fn base_to_usd(sym: &str, price: f64, rates: &Rates) -> Option<f64> {
    let (b, q) = split_pair(sym);
    if b == "USD" { return Some(1.0); }
    if q == "USD" { return if ok_pos(price) { Some(price) } else { usd_rate(&b, rates) }; }
    if let Some(d) = usd_rate(&b, rates) { return Some(d); }
    let qr = usd_rate(&q, rates)?;
    if ok_pos(price) { Some(price * qr) } else { None }
}

pub fn notional_usd(sym: &str, qty: f64, price: f64, rates: &Rates) -> Option<f64> {
    if !qty.is_finite() { return None; }
    base_to_usd(sym, price, rates).map(|b| qty.abs() * b)
}

// ---------------- order ticket ----------------

#[derive(Debug, Clone, PartialEq)]
pub struct TicketMath {
    pub notional_usd: Option<f64>,
    pub risk_at_sl_usd: Option<f64>,
    pub reward_at_tp_usd: Option<f64>,
    pub rr: Option<f64>,
    /// USD per 1.0 of price move per unit (None = no conversion rate available)
    pub usd_per_price_unit: Option<f64>,
}

/// Ticket figures in USD. `symbol` is the instrument symbol ("EURJPY", "AAPL", "BTC"); `entry` the expected
/// entry price; quantities are in base units (or USD amount when `qty_is_usd`).
pub fn ticket_math(market: &str, symbol: &str, qty: f64, qty_is_usd: bool, entry: f64, sl: Option<f64>, tp: Option<f64>, rates: &Rates) -> TicketMath {
    let fx_sym = if market == "FX" { symbol.to_string() } else { format!("{symbol}USD_") }; // non-FX: quoted in USD
    let q2u = if market == "FX" { quote_to_usd(&fx_sym, entry, rates) } else { Some(1.0) };
    // USD-base pairs (USDJPY …): a loss / gain is realised at the SL / TP price, so convert it at that price
    let usd_base = market == "FX" && split_pair(&fx_sym).0 == "USD";
    let at = |p: Option<f64>| if usd_base { p.filter(|x| ok_pos(*x)).map(|x| 1.0 / x).or(q2u) } else { q2u };
    let (k_sl, k_tp) = (at(sl), at(tp));
    let notional = if qty_is_usd { Some(qty.abs()) } else if market == "FX" { notional_usd(&fx_sym, qty, entry, rates) } else if ok_pos(entry) { Some(qty.abs() * entry) } else { None };
    let units = if qty_is_usd { if ok_pos(entry) { qty / entry } else { f64::NAN } } else { qty };
    let risk = match (sl, k_sl) { (Some(s), Some(k)) if ok_pos(entry) && units.is_finite() => Some((entry - s).abs() * units.abs() * k), _ => None };
    let reward = match (tp, k_tp) { (Some(t), Some(k)) if ok_pos(entry) && units.is_finite() => Some((t - entry).abs() * units.abs() * k), _ => None };
    let rr = match (risk, reward) { (Some(r), Some(w)) if r > 0.0 => Some(w / r), _ => None };
    // sizing by risk uses the same SL-price conversion as "Risk at SL"
    TicketMath { notional_usd: notional, risk_at_sl_usd: risk, reward_at_tp_usd: reward, rr, usd_per_price_unit: if sl.is_some() { k_sl } else { q2u } }
}

/// Size that risks `risk_usd` with a stop `stop_dist` away; rounded down to the market's lot step. None if not computable.
pub fn size_by_risk(market: &str, risk_usd: f64, stop_dist: f64, usd_per_price_unit: Option<f64>) -> Option<f64> {
    let k = usd_per_price_unit?;
    if !ok_pos(risk_usd) || !ok_pos(stop_dist) || !ok_pos(k) { return None; }
    // tiny epsilon: 100 / (0.5 × 1/150) is 30,000 exactly but evaluates to 29,999.999… in floating point
    let raw = risk_usd / (stop_dist * k) * (1.0 + 1e-9);
    let r = match market { "STK" => raw.floor(), "CRYPTO" => (raw * 1000.0).floor() / 1000.0, _ => (raw / 1000.0).floor() * 1000.0 };
    if r > 0.0 { Some(r) } else { None }
}

// ---------------- form parsing ----------------

/// Parse a required number from a form field ("1,000" allowed). Blank / invalid / negative → Err with the field name.
pub fn parse_required(label: &str, s: &str, min: f64) -> Result<f64, String> {
    let t = s.trim().replace(',', "");
    if t.is_empty() { return Err(format!("{label} is empty")); }
    match t.parse::<f64>() {
        Ok(v) if v.is_finite() && v >= min => Ok(v),
        Ok(_) => Err(format!("{label} must be at least {min}")),
        Err(_) => Err(format!("{label} is not a number")),
    }
}

/// Whole minutes from a form field, 0..=max. Blank / invalid / fractional → Err (never a silent 0).
pub fn parse_minutes(label: &str, s: &str, max: i64) -> Result<i64, String> {
    let v = parse_required(label, s, 0.0)?;
    if v.fract() != 0.0 || v > max as f64 { return Err(format!("{label} must be a whole number of minutes from 0 to {max}")); }
    Ok(v as i64)
}

/// Trade size for a strategy config: must be a whole number of units (the API rejects fractions); never truncated.
pub fn trade_size_str(s: &str) -> Result<Option<String>, String> {
    let t = s.trim().replace(',', "");
    if t.is_empty() { return Ok(None); }
    let v: f64 = t.parse().map_err(|_| format!("Trade size '{s}' is not a number"))?;
    if !v.is_finite() || v <= 0.0 { return Err("Trade size must be positive".into()); }
    if v.fract() != 0.0 { return Err(format!("Trade size must be a whole number of units (got {s})")); }
    Ok(Some(format!("{}", v as i64)))
}

/// Seconds since the Unix epoch for an ISO timestamp ("…Z", "…+00:00", "…-05:00", "YYYY-MM-DD").
pub fn iso_epoch(s: &str) -> f64 {
    let b = s.as_bytes();
    let num = |a: usize, n: usize| -> i64 { s.get(a..a + n).and_then(|x| x.parse::<i64>().ok()).unwrap_or(0) };
    if b.len() < 10 { return 0.0; }
    let (y, m, d) = (num(0, 4), num(5, 2), num(8, 2));
    let (hh, mm, ss) = if b.len() >= 19 { (num(11, 2), num(14, 2), num(17, 2)) } else { (0, 0, 0) };
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = (if y2 >= 0 { y2 } else { y2 - 399 }) / 400;
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    let mut t = (days * 86400 + hh * 3600 + mm * 60 + ss) as f64;
    if let Some(pos) = s.get(19..).and_then(|r| r.find(|c| c == '+' || c == '-')) {
        let off = &s[19 + pos..];
        let sign = if off.starts_with('-') { -1.0 } else { 1.0 };
        let oh = off.get(1..3).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
        let om = off.get(4..6).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
        t -= sign * (oh * 3600.0 + om * 60.0);
    }
    t
}

/// Browser builds: std::time panics on wasm32 ("unreachable"), so the app copies the browser clock here every second.
static WEB_NOW_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub fn set_web_now_ms(ms: f64) {
    if ms.is_finite() && ms > 0.0 { WEB_NOW_MS.store(ms as u64, std::sync::atomic::Ordering::Relaxed); }
}

/// Wall clock in Unix seconds (in the browser: the last value copied from the page; None until the first copy).
pub fn now_secs() -> Option<f64> {
    #[cfg(not(target_arch = "wasm32"))]
    { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs_f64()) }
    #[cfg(target_arch = "wasm32")]
    { let ms = WEB_NOW_MS.load(std::sync::atomic::Ordering::Relaxed); (ms > 0).then(|| ms as f64 / 1000.0) }
}

/// A quote is stale when its last update is older than `max_age` seconds relative to `now`
/// (or, without a clock, relative to the freshest quote in the list).
pub fn quote_stale(quote_time: Option<&str>, now: Option<f64>, freshest: Option<f64>, max_age: f64) -> bool {
    let Some(t) = quote_time.filter(|s| s.len() >= 10).map(iso_epoch) else { return true };
    match now.or(freshest) { Some(n) => n - t > max_age, None => false }
}

/// Value of cookie `name` in a Cookie header ("a=1; qos_op=xyz") — None when missing or empty
pub fn cookie_value(header: &str, name: &str) -> Option<String> {
    header.split(';').find_map(|c| {
        let (k, v) = c.trim().split_once('=')?;
        (k == name && !v.is_empty()).then(|| v.to_string())
    })
}

/// Set-Cookie for the operator token: HttpOnly (page scripts cannot read it), Secure, SameSite=Strict (never sent by
/// other sites), 12 h like the API's longest session; an empty token deletes the cookie
pub fn op_cookie(name: &str, token: &str) -> String {
    let age = if token.is_empty() { 0 } else { 12 * 3600 };
    format!("{name}={token}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age={age}")
}

/// "http://127.0.0.1:8000" → "ws://127.0.0.1:8000/ws" ("https" → "wss")
pub fn ws_url_from(base: &str) -> String {
    let b = base.trim().trim_end_matches('/');
    let b = if let Some(r) = b.strip_prefix("https://") { format!("wss://{r}") } else if let Some(r) = b.strip_prefix("http://") { format!("ws://{r}") } else { format!("ws://{b}") };
    format!("{b}/ws")
}

/// Up/down/total for a heatmap when the API did not send breadth (cached responses).
pub fn breadth_of(changes: impl IntoIterator<Item = f64>) -> (usize, usize, usize) {
    let (mut up, mut down, mut n) = (0, 0, 0);
    for c in changes { n += 1; if c > 0.0 { up += 1 } else if c < 0.0 { down += 1 } }
    (up, down, n)
}

/// Terminal zoom steps (Ctrl/Cmd + = / − / 0): 50 %–200 % in 10 % steps
pub const UI_ZOOM_MIN: f64 = 0.5;
pub const UI_ZOOM_MAX: f64 = 2.0;

pub fn next_ui_zoom(cur: f64, action: &str) -> f64 {
    let cur = if cur.is_finite() && cur > 0.0 { cur } else { 1.0 };
    let z = match action {
        "ui_zoom_in" => cur + 0.1,
        "ui_zoom_out" => cur - 0.1,
        _ => 1.0,
    };
    ((z * 10.0).round() / 10.0).clamp(UI_ZOOM_MIN, UI_ZOOM_MAX)
}

/// Saved zoom from localStorage; None when missing or invalid
pub fn parse_ui_zoom(s: &str) -> Option<f64> {
    let z: f64 = s.trim().parse().ok()?;
    if z.is_finite() && (UI_ZOOM_MIN..=UI_ZOOM_MAX).contains(&z) { Some(z) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn money_rounds_cents_into_units() {
        assert_eq!(money(2.999), "3.00");
        assert_eq!(money(-149.996), "-150.00");
        assert_eq!(money(999.995), "1,000.00");
        assert_eq!(money(1234567.891), "1,234,567.89");
        assert_eq!(money(-1234.5), "-1,234.50");
    }
    #[test] fn money_no_negative_zero_and_neutral_colour() {
        assert_eq!(money(-0.004), "0.00");
        assert_eq!(pnl_class(-0.004), "text-neutral");
        assert_eq!(pnl_class(-0.01), "text-down");
        assert_eq!(pnl_class(0.01), "text-up");
    }
    #[test] fn money_non_finite_and_ib_unset() {
        assert_eq!(money(f64::NAN), "—");
        assert_eq!(money(f64::INFINITY), "—");
        assert_eq!(money(1.7976931348623157e308), "—");
        assert_eq!(finite_opt(Some(1.7976931348623157e308)), None);
        assert_eq!(finite_opt(Some(-12.5)), Some(-12.5));
    }
    #[test] fn qty_and_px() {
        assert_eq!(qty_fmt(0.1 + 0.2), "0.3");
        assert_eq!(qty_fmt(25000.0), "25,000");
        assert_eq!(qty_fmt(-100.0), "-100");
        assert_eq!(qty_fmt(0.00012345), "0.00012345");
        assert_eq!(px(f64::NAN, 5), "—");
        assert_eq!(px(150.1234, 3), "150.123");
    }
    #[test] fn local_currency_values() {
        assert_eq!(money_in(12345.0, false, "JPY"), "12,345.00 JPY");
        assert_eq!(money_in(-5.5, true, "JPY"), "-5.50");
        assert_eq!(usd_total([(10.0, true), (12345.0, false), (-2.5, true)]), (7.5, 1));
    }
    #[test] fn change_pct_without_prev_close() {
        assert_eq!(chg_pct(0.0, 0.0), "—");
        assert_eq!(chg_pct(1.234, 100.0), "+1.23%");
        assert_eq!(frac_pct(None, 1), "—");
        assert_eq!(frac_pct(Some(0.5), 1), "50.0%");
        assert_eq!(pct_of(10.0, 0.0), "");
    }
    #[test] fn decimals_rule() {
        assert_eq!(decimals_for_key("FX:USDJPY"), 3);
        assert_eq!(decimals_for_key("FX:GBPUSD"), 5);
        assert_eq!(decimals_for_key("CRYPTO:BTC"), 0);
        assert_eq!(decimals_for_key("STK:SPY"), 2);
        assert_eq!(round_to_dec(1.096645000000001, 5), 1.09665);
    }
    fn rates() -> Rates { rates_from_prices([("EURUSD", 1.085), ("USDJPY", 160.0), ("GBPUSD", 1.29), ("AUDUSD", 0.65)]) }
    #[test] fn ticket_usd_quoted() {
        let m = ticket_math("FX", "EURUSD", 20000.0, false, 1.0850, Some(1.0830), Some(1.0890), &rates());
        assert!((m.notional_usd.unwrap() - 21700.0).abs() < 1e-6);
        assert!((m.risk_at_sl_usd.unwrap() - 40.0).abs() < 1e-6);
        assert!((m.rr.unwrap() - 2.0).abs() < 1e-9);
    }
    #[test] fn ticket_usd_base() {
        let m = ticket_math("FX", "USDJPY", 20000.0, false, 150.0, Some(149.5), None, &rates());
        assert!((m.notional_usd.unwrap() - 20000.0).abs() < 1e-6);
        assert!((m.risk_at_sl_usd.unwrap() - 10000.0 / 149.5).abs() < 1e-6);   // 10,000 JPY converted at the SL price
    }
    #[test] fn size_by_risk_exact_lot_and_fractional_qty() {
        let k = ticket_math("FX", "USDJPY", 1.0, false, 150.0, None, None, &rates()).usd_per_price_unit;
        assert_eq!(size_by_risk("FX", 100.0, 0.50, k), Some(30000.0));
        assert_eq!(qty_fmt(1234.5), "1,234.5");
        assert_eq!(qty_fmt(-1234567.25), "-1,234,567.25");
    }
    #[test] fn ticket_crosses() {
        let m = ticket_math("FX", "EURJPY", 20000.0, false, 160.0, Some(159.5), None, &rates());
        assert!((m.risk_at_sl_usd.unwrap() - 62.5).abs() < 1e-6);           // 10,000 JPY at USDJPY 160
        assert!((m.notional_usd.unwrap() - 21700.0).abs() < 1e-6);          // 20k EUR
        let g = ticket_math("FX", "EURGBP", 100000.0, false, 0.84, Some(0.8380), None, &rates());
        assert!((g.notional_usd.unwrap() - 108500.0).abs() < 1e-6);
        assert!((g.risk_at_sl_usd.unwrap() - 258.0).abs() < 1e-6);          // 200 GBP * 1.29
        assert_eq!(size_by_risk("FX", 260.0, 0.002, g.usd_per_price_unit), Some(100000.0));
    }
    #[test] fn ticket_missing_rate_is_none() {
        let r = rates_from_prices([("EURUSD", 1.085)]);
        let m = ticket_math("FX", "EURGBP", 100000.0, false, 0.84, Some(0.838), None, &r);
        assert_eq!(m.risk_at_sl_usd, None);
        assert_eq!(m.usd_per_price_unit, None);
        assert!((m.notional_usd.unwrap() - 108500.0).abs() < 1e-6);         // base EUR is known
        assert_eq!(size_by_risk("FX", 100.0, 0.002, m.usd_per_price_unit), None);
        let n = ticket_math("FX", "GBPJPY", 1000.0, false, 200.0, None, None, &r);
        assert_eq!(n.notional_usd, None);
    }
    #[test] fn ticket_stock_crypto() {
        let m = ticket_math("STK", "AAPL", 10.0, false, 230.0, Some(225.0), None, &Rates::new());
        assert_eq!(m.notional_usd, Some(2300.0));
        assert_eq!(m.risk_at_sl_usd, Some(50.0));
        let c = ticket_math("CRYPTO", "BTC", 500.0, true, 100000.0, None, None, &Rates::new());
        assert_eq!(c.notional_usd, Some(500.0));
    }
    #[test] fn required_fields() {
        assert!(parse_required("Max daily loss", "", 0.0).is_err());
        assert!(parse_required("Max daily loss", "5k", 0.0).is_err());
        assert_eq!(parse_required("Max daily loss", "1,000", 0.0), Ok(1000.0));
        assert_eq!(parse_required("Max daily loss", "0", 0.0), Ok(0.0));
    }
    #[test] fn news_pause_minutes() {
        assert!(parse_minutes("News pause", "", 240).is_err());
        assert!(parse_minutes("News pause", "abc", 240).is_err());
        assert!(parse_minutes("News pause", "2.5", 240).is_err());
        assert!(parse_minutes("News pause", "300", 240).is_err());
        assert_eq!(parse_minutes("News pause", "0", 240), Ok(0));
        assert_eq!(parse_minutes("News pause", " 30 ", 240), Ok(30));
    }
    #[test] fn trade_size_not_truncated() {
        assert_eq!(trade_size_str("20000"), Ok(Some("20000".into())));
        assert!(trade_size_str("0.05").is_err());
        assert_eq!(trade_size_str(""), Ok(None));
    }
    #[test] fn time_and_staleness() {
        assert_eq!(iso_epoch("2026-10-27T05:20:06Z"), 1793078406.0);
        assert_eq!(iso_epoch("2026-10-27T00:20:06.123-05:00"), 1793078406.0);
        assert!(quote_stale(Some("2026-10-27T05:20:06+00:00"), Some(1793078406.0 + 600.0), None, 120.0));
        assert!(!quote_stale(Some("2026-10-27T05:20:06+00:00"), Some(1793078406.0 + 5.0), None, 120.0));
        assert!(quote_stale(None, Some(0.0), None, 120.0));
        assert_eq!(breadth_of([1.0, -0.5, 0.0]), (1, 1, 3));
        assert_eq!(ws_url_from("http://10.0.0.5:9000/"), "ws://10.0.0.5:9000/ws");
        assert_eq!(ws_url_from("https://bot.local"), "wss://bot.local/ws");
        assert_eq!(cookie_value("a=1; qos_op=tok123; b=2", "qos_op").as_deref(), Some("tok123"));
        assert_eq!(cookie_value("xqos_op=no; qos_op=", "qos_op"), None);
        assert!(op_cookie("qos_op", "t").contains("HttpOnly; Secure; SameSite=Strict; Max-Age=43200"));
        assert!(op_cookie("qos_op", "").ends_with("Max-Age=0"));
    }

    #[test]
    fn ui_zoom_steps_and_bounds() {
        assert_eq!(next_ui_zoom(1.0, "ui_zoom_in"), 1.1);
        assert_eq!(next_ui_zoom(1.0, "ui_zoom_out"), 0.9);
        assert_eq!(next_ui_zoom(1.3, "ui_zoom_reset"), 1.0);
        assert_eq!(next_ui_zoom(2.0, "ui_zoom_in"), 2.0);
        assert_eq!(next_ui_zoom(0.5, "ui_zoom_out"), 0.5);
        assert_eq!(next_ui_zoom(f64::NAN, "ui_zoom_in"), 1.1);
        let mut z = 1.0; for _ in 0..3 { z = next_ui_zoom(z, "ui_zoom_in"); } assert_eq!(z, 1.3);
        assert_eq!(parse_ui_zoom(" 1.2 "), Some(1.2));
        assert_eq!(parse_ui_zoom("abc"), None);
        assert_eq!(parse_ui_zoom("5"), None);
        assert_eq!(parse_ui_zoom(""), None);
    }
}
