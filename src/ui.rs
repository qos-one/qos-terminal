//! Panels of the trading screen
use std::collections::HashMap;

use dioxus::prelude::*;

use crate::api::*;

pub const LOGO: Asset = asset!("/assets/qos-logo.png");

// ---------------- shared state ----------------
#[derive(Clone, Copy)]
pub struct State {
    pub quotes: Signal<Vec<Quote>>,
    pub prev_mid: Signal<HashMap<String, f64>>,
    pub snapshot: Signal<Option<Snapshot>>,
    pub api_error: Signal<Option<String>>,
    pub selected: Signal<String>,          // active instrument (ticket, watchlist highlight)
    pub pane_key: [Signal<String>; 4],     // instrument of each pane
    pub pane_tf: [Signal<String>; 4],
    pub pane_bars: [Signal<Vec<Bar>>; 4],
    pub pane_loading: [Signal<bool>; 4],
    pub layout: Signal<usize>,             // 1 | 2 | 4
    pub active_pane: Signal<usize>,
    pub toast: Signal<Option<(bool, String)>>,
    pub refresh_tick: Signal<u32>,
    pub market: Signal<Vec<MarketItem>>,
    pub markets: Signal<Vec<Market>>,
    pub watchlist: Signal<Vec<Instrument>>,
    pub wl_market: Signal<String>,         // watchlist market filter ("ALL" or id)
    pub wl_filter: Signal<String>,         // filter text
    pub browser_open: Signal<bool>,
    pub mode: Signal<String>,              // "manual" | "agent"
    pub heat_group: Signal<Option<String>>, // group whose heatmap is open
    pub chart_full: Signal<Option<usize>>,  // fullscreen pane
    pub risk: Signal<Option<RiskInfo>>,
    pub indicators: Signal<IndCfg>,
    pub ticket_prefill: Signal<Option<Prefill>>,
    pub ctx_menu: Signal<Option<CtxMenu>>,
    pub crosshair: Signal<Option<Crosshair>>,
    pub bottom_h: Signal<f64>,                 // bottom panel height (px)
    pub bottom_drag: Signal<Option<(f64, f64)>>, // (drag start y, start height)
    // round 6
    pub bottom_tab: Signal<usize>,
    pub conn: Signal<Option<ConnInfo>>,
    pub ws_open: Signal<bool>,
    pub notifs: Signal<Vec<Notif>>,
    pub notif_open: Signal<bool>,
    pub help_open: Signal<bool>,
    pub wl_w: Signal<f64>,
    pub ticket_w: Signal<f64>,
    pub side_drag: Signal<Option<(u8, f64, f64)>>, // (0=watchlist 1=ticket, start x, start width)
    pub alerts: Signal<Vec<Alert>>,
    pub alert_dialog: Signal<Option<f64>>,        // open the set-alert dialog at this price
    pub sound: Signal<bool>,
    pub preset: Signal<String>,
    pub ui_loaded: Signal<bool>,
    pub submit_req: Signal<u32>,                   // Enter hotkey → ticket submit
    pub news_cache: Signal<Option<(String, Vec<NewsItem>)>>,
    pub session: Signal<Option<SetupStatus>>,    // None = status not known yet
    pub login_busy: Signal<bool>,
    pub op: Signal<Option<OpStatus>>,            // operator (who) + permissions
    pub hist_scope: Signal<String>,              // "" = selected account, "ALL" = all accounts
    pub node_events: Signal<Vec<serde_json::Value>>, // live events from Nautilus nodes (newest first, ≤ 300)
    pub node_live: Signal<HashMap<String, serde_json::Value>>, // latest heartbeat per node
    pub bt_open: Signal<Option<(String, String)>>,
    pub node_detail: Signal<Option<String>>,
    pub strat_pkg: Signal<Option<String>>,          // package to open in the Strategies tab (link from the agent detail page)        // node whose detail page is open
    pub perf_full: Signal<bool>,                     // Performance tab expanded to fullscreen  // open this report in the Backtests tab (kind, file)
    pub confirm: Signal<Option<ConfirmReq>>,         // pending confirmation for a destructive / order action
    pub err_src: Signal<std::collections::BTreeMap<&'static str, String>>, // API errors by source (banner shows them until that source succeeds)
    pub ws_quote_seq: Signal<u32>,                   // counts websocket quote pushes (polling resumes when it stops moving)
    pub ws_snap_seq: Signal<u32>,                    // counts websocket snapshot pushes
    pub acct_missing: Signal<bool>,
    pub bar_dec: Signal<HashMap<String, usize>>,
    pub tz: Signal<String>,                           // viewer's IANA time zone, sent with history requests ("" = UTC)      // display decimals per instrument as reported by /bars                  // last snapshot had no account data (header shows last known values as stale / "—")
}

/// Merge a snapshot into the previous one: fields the update lacks (websocket push without reconcile / running_nodes /
/// account) keep their last known values. Returns the merged snapshot and whether the update carried account data.
pub fn merge_snapshot(prev: Option<&Snapshot>, mut s: Snapshot, has_reconcile: bool, has_nodes: bool) -> (Snapshot, bool) {
    let acct_ok = !s.account.account_id.is_empty();
    if let Some(p) = prev {
        if !has_reconcile { s.reconcile = p.reconcile.clone(); }
        if !has_nodes { s.running_nodes = p.running_nodes.clone(); }
        if !acct_ok && !p.account.account_id.is_empty() { s.account = p.account.clone(); }
    }
    (s, acct_ok)
}

/// Action that runs only after the user confirms it in ConfirmDialog
#[derive(Clone, Debug, PartialEq)]
pub enum ConfirmAction { Kill, ClosePos(String, bool), Reverse(String), NodeStop(String), MoveOrder { id: i64, from: f64, to: f64, pane: usize }, CancelOrder(i64), ModifyOrder { id: i64, qty: Option<f64>, price: Option<f64> }, ConfirmBaseline(Option<f64>) }

#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmReq { pub title: String, pub body: String, pub ok_label: String, pub action: ConfirmAction }

impl State {
    pub fn role(&self) -> String { self.op.peek().as_ref().and_then(|o| o.operator.as_ref()).map(|o| o.role.clone()).unwrap_or_else(|| "admin".into()) }
    pub fn can_trade(&self) -> bool { matches!(self.role().as_str(), "admin" | "trader") }
    pub fn is_admin(&self) -> bool { self.role() == "admin" }
    /// roles the API lets activate the kill switch (operators.py WRITE_RULES: /risk/kill)
    pub fn can_kill(&self) -> bool { matches!(self.role().as_str(), "admin" | "risk" | "trader") }
    /// roles that may change risk state: resume after a halt, confirm the daily-loss baseline (/risk)
    pub fn can_risk(&self) -> bool { matches!(self.role().as_str(), "admin" | "risk") }
    /// Record (Some) or clear (None) the API error of one polling source; the banner shows any remaining error.
    pub fn set_err(&mut self, src: &'static str, e: Option<String>) {
        let changed = { let m = self.err_src.peek(); match &e { Some(x) => m.get(src) != Some(x), None => m.contains_key(src) } };
        if changed {
            let mut m = self.err_src.write();
            match e { Some(x) => { m.insert(src, x); } None => { m.remove(src); } }
        }
        let first = self.err_src.peek().values().next().cloned();
        if *self.api_error.peek() != first { self.api_error.set(first); }
    }
    /// Store a snapshot. A websocket push may lack `reconcile` / `running_nodes` / account data: keep the last known
    /// values then (and flag the account as missing) instead of blanking the reconciliation strip and node counts.
    pub fn apply_snapshot(&mut self, s: Snapshot, has_reconcile: bool, has_nodes: bool) {
        let (s, acct_ok) = merge_snapshot(self.snapshot.peek().as_ref(), s, has_reconcile, has_nodes);
        if *self.acct_missing.peek() == acct_ok { self.acct_missing.set(!acct_ok); }
        if !s.mode.is_empty() && *self.mode.peek() != s.mode { self.mode.set(s.mode.clone()); }
        self.snapshot.set(Some(s));
    }
    /// Watchlist rows as shown (market filter + text filter) — also used by the ↑/↓ hotkeys
    pub fn visible_watchlist(&self) -> Vec<Instrument> {
        let wl_market = self.wl_market.read().clone();
        let filter = self.wl_filter.read().to_lowercase();
        self.watchlist.read().iter()
            .filter(|i| wl_market == "ALL" || i.market == wl_market)
            .filter(|i| filter.is_empty() || i.symbol.to_lowercase().contains(&filter) || i.name.to_lowercase().contains(&filter))
            .cloned().collect()
    }
    /// Display decimals for an instrument: quote → /bars → server rule (FX 5, JPY 3, BTC 0, else 2)
    pub fn dec_for(&self, key: &str) -> usize {
        if let Some(d) = self.quotes.read().iter().find(|q| q.key == key).map(|q| q.decimals).filter(|d| *d > 0) { return d; }
        if let Some(d) = self.bar_dec.read().get(key) { return *d; }
        decimals_for_key(key)
    }
    pub fn ask(&mut self, title: impl Into<String>, body: impl Into<String>, ok_label: impl Into<String>, action: ConfirmAction) {
        // a dialog that is still open is cancelled first (a pending "Move order?" puts its line back) — unless the
        // new request moves the SAME order again: its line already sits at the new drop price
        let same_order = matches!((revert_on_cancel(self.confirm.peek().as_ref()), &action), (Some((_, a, _)), ConfirmAction::MoveOrder { id, .. }) if a == *id);
        if !same_order { cancel_confirm(self); }
        self.confirm.set(Some(ConfirmReq { title: title.into(), body: body.into(), ok_label: ok_label.into(), action }));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notif { pub ts: String, pub kind: String, pub text: String, pub read: bool }

pub const SIDE_MIN: f64 = 200.0;
pub const SIDE_MAX: f64 = 560.0;

pub fn now_hms() -> String {
    let secs = crate::calc::now_secs().map(|s| s as u64).unwrap_or(0) % 86400;     // never std::time: it panics in the browser
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

impl State {
    /// Add a notification to the notification center (+ sound)
    pub fn notify(&mut self, kind: &str, text: impl Into<String>) {
        let text = text.into();
        let mut v = self.notifs.write();
        v.insert(0, Notif { ts: now_hms(), kind: kind.to_string(), text: text.clone(), read: false });
        v.truncate(200);
        drop(v);
        if *self.sound.peek() {
            let snd = match kind { "fill" => "fill", "alert" => "alert", "error" | "disconnect" => "error", _ => "info" };
            let _ = document::eval(&format!("if (window.qxSound) window.qxSound.play('{snd}');"));
        }
    }
    pub fn apply_preset(&mut self, name: &str) {
        match name {
            "Research" => { self.wl_w.set(260.0); self.ticket_w.set(SIDE_MIN); self.bottom_h.set(BOTTOM_BIG); self.layout.set(2); self.bottom_tab.set(5); }
            "Monitoring" => { self.wl_w.set(340.0); self.ticket_w.set(300.0); self.bottom_h.set(300.0); self.layout.set(4); self.bottom_tab.set(0); }
            _ => { self.wl_w.set(300.0); self.ticket_w.set(280.0); self.bottom_h.set(BOTTOM_DEFAULT); self.layout.set(1); self.bottom_tab.set(0); }
        }
        self.preset.set(name.to_string());
    }
    pub fn ui_settings(&self) -> UiSettings {
        UiSettings {
            wl_w: *self.wl_w.peek(), ticket_w: *self.ticket_w.peek(), bottom_h: *self.bottom_h.peek(), layout: *self.layout.peek(),
            pane_key: self.pane_key.iter().map(|k| k.peek().clone()).collect(),
            pane_tf: self.pane_tf.iter().map(|k| k.peek().clone()).collect(),
            sound: *self.sound.peek(), preset: self.preset.peek().clone(),
        }
    }
    pub fn apply_ui(&mut self, u: &UiSettings) {
        self.wl_w.set(u.wl_w.clamp(SIDE_MIN, SIDE_MAX)); self.ticket_w.set(u.ticket_w.clamp(SIDE_MIN, SIDE_MAX));
        self.bottom_h.set(u.bottom_h.clamp(BOTTOM_MIN, 900.0));
        if matches!(u.layout, 1 | 2 | 4) { self.layout.set(u.layout); }
        for (i, k) in u.pane_key.iter().enumerate().take(4) { if !k.is_empty() { self.pane_key[i].set(k.clone()); } }
        for (i, t) in u.pane_tf.iter().enumerate().take(4) { if !t.is_empty() { self.pane_tf[i].set(t.clone()); } }
        self.sound.set(u.sound);
        if !u.preset.is_empty() { self.preset.set(u.preset.clone()); }
        if let Some(k) = u.pane_key.first() { if !k.is_empty() { self.selected.set(k.clone()); } }
    }
}

pub const BOTTOM_MIN: f64 = 36.0;
pub const BOTTOM_DEFAULT: f64 = 190.0;
pub const BOTTOM_BIG: f64 = 520.0;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IndCfg {
    pub ema: Vec<u32>,
    pub sma: Vec<u32>,
    pub bb: Option<BbCfg>,
    pub vwap: bool,
    pub volume: bool,
    pub sub: String, // none | rsi | macd
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BbCfg { pub period: u32, pub k: f64 }
impl Default for IndCfg {
    fn default() -> Self { IndCfg { ema: vec![20], sma: vec![], bb: None, vwap: false, volume: true, sub: "none".into() } }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prefill { pub side: String, pub otype: String, pub price: f64 }

#[derive(Clone, Debug, PartialEq)]
pub struct CtxMenu { pub x: f64, pub y: f64, pub price: f64, pub line_kind: String, pub line_id: String, pub line_title: String, pub pane: usize }

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
pub struct Crosshair {
    #[serde(default)] pub pane: usize,
    #[serde(default)] pub visible: bool,
    #[serde(default)] pub time: f64,
    #[serde(default)] pub o: f64, #[serde(default)] pub h: f64, #[serde(default)] pub l: f64, #[serde(default)] pub c: f64,
    #[serde(default)] pub v: f64,
    #[serde(default)] pub ind: serde_json::Map<String, serde_json::Value>,
}

pub fn tf_seconds(id: &str) -> u64 {
    match id { "1m" => 60, "5m" => 300, "15m" => 900, "30m" => 1800, "1h" => 3600, "4h" => 14400, "1D" => 86400, "1W" => 604800, _ => 60 }
}

/// Call a chart bridge function (waits for the script to load if it is not ready yet)
pub fn js_call(pane: usize, code: &str, need_ready: bool) {
    let cond = if need_ready { format!("window.qxChart && window.qxChart.ready({pane})") } else { "window.qxChart".to_string() };
    let wrapped = format!(
        "(function(){{ const run = () => {{ const I = window.qxChart.get({pane}); {code} }}; if ({cond}) {{ run(); }} else {{ (window.__qxOnReady = window.__qxOnReady || []).push({{ p: {pane}, n: {need_ready}, run }}); }} }})();"
    );
    let _ = document::eval(&wrapped);
}

pub const TIMEFRAMES: [(&str, &str, &str, &str); 8] = [
    // id, label, IBKR bar size, IBKR duration
    ("1m", "1m", "1 min", "1 D"),
    ("5m", "5m", "5 mins", "3 D"),
    ("15m", "15m", "15 mins", "1 W"),
    ("30m", "30m", "30 mins", "2 W"),
    ("1h", "1h", "1 hour", "1 M"),
    ("4h", "4h", "4 hours", "3 M"),
    ("1D", "1D", "1 day", "1 Y"),
    ("1W", "1W", "1 week", "5 Y"),
];

pub fn tf_params(id: &str) -> (String, String) {
    TIMEFRAMES.iter().find(|t| t.0 == id).map(|t| (t.2.to_string(), t.3.to_string()))
        .unwrap_or(("1 min".into(), "1 D".into()))
}

pub fn app_state() -> State { use_context::<State>() }

impl State {
    /// Select an instrument: put it in the active pane and make it the current instrument
    pub fn select(&mut self, key: String) {
        let p = (*self.active_pane.read()).min(self.pane_key.len() - 1);
        self.pane_key[p].set(key.clone());
        self.selected.set(key);
    }
}

// ---------------- formatting ----------------
// implementations (unit-tested) live in calc.rs
pub use crate::calc::{chg_pct, decimals_for_key, finite_opt, frac_pct, iso_epoch, money, money_in, pct_of, pnl_class, px, qty_fmt};

fn short_time(iso: &Option<String>) -> String {
    iso.as_deref().map(|s| s.replace('T', " ").chars().skip(11).take(8).collect()).unwrap_or_default()
}

fn market_tag(m: &str) -> &'static str {
    match m { "FX" => "tag fx", "STK" => "tag stock", "CRYPTO" => "tag crypto", "IDX" => "tag index", _ => "tag" }
}

pub fn source_label(src: &str) -> String {
    if let Some(n) = src.strip_prefix("node:") { return n.to_string(); }
    if let Some(a) = src.strip_prefix("agent:") { return a.to_string(); }
    match src {
        "" | "manual" => "Manual".into(),
        "nautilus" => "Nautilus".into(),
        "mixed" => "Mixed".into(),
        "untracked" => "Untracked".into(),
        other => other.to_string(),
    }
}

fn source_class(src: &str) -> &'static str {
    if src.starts_with("node:") || src == "nautilus" { "tag xs src-node" }
    else if src.starts_with("agent:") { "tag xs src-agent" }
    else if src == "mixed" { "tag xs src-mixed" }
    else if src == "untracked" { "tag xs src-warn" }
    else { "src-manual" }
}

/// Order placed by hand (not by a node / agent)
pub fn order_is_manual(o: &Order) -> bool {
    let src = if o.source.is_empty() { legacy_source(&o.order_ref) } else { o.source.clone() };
    src.is_empty() || src == "manual" || src.starts_with("manual:")
}

fn legacy_source(order_ref: &str) -> String {
    if order_ref.starts_with("agent:") { order_ref.to_string() } else { "manual".to_string() }
}

#[component]
fn SourceTag(source: String, #[props(default)] strategy: String, #[props(default)] title: String) -> Element {
    let lbl = source_label(&source);
    let tip = if !title.is_empty() { title } else if !strategy.is_empty() { format!("{source} · {strategy}") } else { source.clone() };
    rsx! { span { class: "{source_class(&source)}", title: "{tip}", "{lbl}" } }
}

fn market_short(m: &str) -> &'static str {
    match m { "FX" => "FX", "STK" => "US", "CRYPTO" => "CRYPTO", "IDX" => "IDX", _ => "" }
}

// ---------------- header ----------------
#[component]
pub fn Header() -> Element {
    let mut st = app_state();
    let snap = st.snapshot.read();
    let (ib_ok, net, upnl, rpnl, acct, ccy) = match snap.as_ref() {
        Some(s) => (s.ib_connected, s.account.net_liquidation, s.account.unrealized_pnl, s.account.realized_pnl, s.account.account_id.clone(), s.account.currency.clone()),
        None => (false, 0.0, 0.0, 0.0, String::new(), String::new()),
    };
    // account values are in the account's base currency — label it when that is not USD
    let ccy_lbl = if ccy.is_empty() || ccy == "USD" { String::new() } else { format!(" ({ccy})") };
    drop(snap);
    let api_ok = st.api_error.read().is_none();
    let last_q = st.quotes.read().iter().filter_map(|q| q.time.clone()).max();
    // no account data at all → "—"; account data missing from the latest update → last known values, marked stale
    let known = !acct.is_empty();
    let stale = *st.acct_missing.read() || !api_ok || !ib_ok;
    let (net_s, upnl_s, rpnl_s) = if known { (money(net), money(upnl), money(rpnl)) } else { ("—".to_string(), "—".to_string(), "—".to_string()) };
    let stale_cls = if known && stale { " stale" } else { "" };
    let stale_tip = if known && stale { "Last known value — account data is not updating (API / IBKR disconnected or account not available)" } else { "" };

    rsx! {
        header { class: "hdr",
            // left: shrinks and is clipped when the window is narrow (never slides under the controls on the right)
            div { class: "hdr-left",
                div { class: "brand", title: "QOS — Quant Operating System",
                    img { class: "logo-img", src: LOGO, alt: "QOS" }
                }
                div { class: "hdr-stats",
                    div { class: "stat", title: "{stale_tip}", span { class: "lbl", "Net Liq{ccy_lbl}" } span { class: "val{stale_cls}", "{net_s}" } }
                    div { class: "stat stat-2", title: "{stale_tip}", span { class: "lbl", "Unrealized" } span { class: "val {pnl_class(upnl)}{stale_cls}", "{upnl_s}" } }
                    div { class: "stat stat-2", title: "{stale_tip}", span { class: "lbl", "Realized" } span { class: "val {pnl_class(rpnl)}{stale_cls}", "{rpnl_s}" } }
                    if known && stale { span { class: "pill warn", title: "{stale_tip}", "STALE" } }
                }
                DailyRisk {}
            }
            div { class: "hdr-right",
                ModeSwitch {}
                KillSwitch {}
                FullscreenButton {}
                AccountMenu {}
                CloudMenu {}
                AssistantChip {}
                OperatorChip {}
                PresetSwitch {}
                NotifBell {}
                button { class: "tool", title: "Keyboard shortcuts (?)", onclick: move |_| { let v = !*st.help_open.peek(); st.help_open.set(v); }, "?" }
                ConnPill { ib_ok, api_ok, last: short_time(&last_q) }
            }
        }
    }
}

// ---------------- watchlist ----------------
#[component]
pub fn Watchlist() -> Element {
    let mut st = app_state();
    let wl_w = *st.wl_w.read();
    let quotes = st.quotes.read().clone();
    let prev = st.prev_mid.read().clone();
    let selected = st.selected.read().clone();
    let markets = st.markets.read().clone();
    let wl_market = st.wl_market.read().clone();
    let freshest = quotes.iter().filter_map(|q| q.time.as_deref()).map(iso_epoch).fold(None, |a: Option<f64>, t| Some(a.map_or(t, |x| x.max(t))));
    let now = crate::calc::now_secs();
    // the whole feed is stale when the API or IBKR is down (values below are the last ones received)
    let feed_down = st.api_error.read().is_some() || st.conn.read().as_ref().map(|c| c.state != "connected").unwrap_or(false);
    let by_key: HashMap<String, Quote> = quotes.into_iter().map(|q| (q.key.clone(), q)).collect();

    let rows: Vec<Instrument> = st.visible_watchlist();

    rsx! {
        aside { class: "panel watch", style: "width: {wl_w}px",
            div { class: "panel-title",
                "Watchlist"
                button { class: "mini accent", onclick: move |_| st.browser_open.set(true), "+ Add" }
            }
            div { class: "wl-tools",
                Dropdown { class: "wl-market", value: "{wl_market}", onchange: move |v| st.wl_market.set(v),
                    options: std::iter::once(("ALL".to_string(), "All markets".to_string())).chain(markets.iter().map(|m| (m.id.clone(), m.name.clone()))).collect::<DdOpts>() }
                input { class: "wl-search", placeholder: "Filter...", value: "{st.wl_filter}", oninput: move |e| st.wl_filter.set(e.value()) }
            }
            table { class: "wl",
                thead { tr { th { "Symbol" } th { class: "r", "Bid" } th { class: "r", "Ask" } th { class: "r", "Chg%" } th { "" } } }
                tbody {
                    if rows.is_empty() { tr { td { colspan: "5", class: "empty small", "No symbols. Click + Add to add one." } } }
                    for inst in rows {
                        {
                            let key = inst.key.clone();
                            let q = by_key.get(&key).cloned().unwrap_or_default();
                            let dir = prev.get(&key).map(|p| if q.mid > *p { 1 } else if q.mid < *p { -1 } else { 0 }).unwrap_or(0);
                            let stale = feed_down || crate::calc::quote_stale(q.time.as_deref(), now, freshest, 120.0) && (q.bid > 0.0 || q.last > 0.0);
                            let row_cls = format!("{}{}", if key == selected { "sel" } else { "" }, if stale { " stale" } else { "" });
                            let row_tip = if stale { format!("Stale: last update {}", q.time.clone().unwrap_or_else(|| "never".into()).replace('T', " ").chars().take(19).collect::<String>()) } else { String::new() };
                            let tick_cls = if dir > 0 { "tick-up" } else if dir < 0 { "tick-down" } else { "" };
                            let dec = if q.decimals > 0 { q.decimals } else { inst.decimals };
                            let k_sel = key.clone();
                            let k_rm = key.clone();
                            let (bid, ask) = if q.bid > 0.0 { (q.bid, q.ask) } else { (q.last, q.last) };
                            rsx! {
                                tr { class: "{row_cls}", title: "{row_tip}",
                                    onclick: move |_| st.select(k_sel.clone()),
                                    td { class: "sym",
                                        span { class: "{market_tag(&inst.market)} xs", "{market_short(&inst.market)}" }
                                        "{inst.symbol}"
                                        if inst.market == "FX" { span { class: "spread", "{q.spread_pip:.1}p" } }
                                    }
                                    td { class: "r num {tick_cls}", "{px(bid, dec)}" }
                                    td { class: "r num {tick_cls}", "{px(ask, dec)}" }
                                    td { class: "r num {pnl_class(q.change_pct)}", "{chg_pct(q.change_pct, q.prev_close)}" }
                                    td { class: "r",
                                        button { class: "rm", title: "Remove from watchlist",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                let k = k_rm.clone();
                                                spawn(async move {
                                                    match remove_watch(k).await {
                                                        Ok(items) => st.watchlist.set(items),
                                                        Err(er) => st.toast.set(Some((false, format!("Remove failed: {er}")))),
                                                    }
                                                });
                                            }, "×" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------- instrument browser (modal) ----------------
#[component]
pub fn InstrumentBrowser() -> Element {
    let mut st = app_state();
    let mut bg_down = use_signal(|| false);
    let markets = st.markets.read().clone();
    let mut market = use_signal(|| "STK".to_string());
    let mut q = use_signal(String::new);
    let mut results = use_signal(Vec::<Instrument>::new);
    let mut loading = use_signal(|| false);
    let in_wl: Vec<String> = st.watchlist.read().iter().map(|i| i.key.clone()).collect();

    // load the list when the window opens or the market/search term changes
    use_effect(move || {
        let open = *st.browser_open.read();
        let m = market.read().clone();
        let qq = q.read().clone();
        if !open { return; }
        spawn(async move {
            loading.set(true);
            match get_instruments(m, qq).await {
                Ok(r) => results.set(r),
                Err(e) => st.toast.set(Some((false, format!("Could not load instruments: {e}")))),
            }
            loading.set(false);
        });
    });

    if !*st.browser_open.read() { return rsx! {}; }

    rsx! {
        div { class: "modal-bg", onmousedown: move |_| bg_down.set(true), onclick: move |_| { if *bg_down.peek() { st.browser_open.set(false); } bg_down.set(false); },
            div { class: "modal resizable", onmousedown: move |e| { e.stop_propagation(); bg_down.set(false); }, onclick: move |e| e.stop_propagation(),
                div { class: "modal-head",
                    span { class: "modal-title", "Add Instrument" }
                    span { class: "muted small", "Instruments tradable at IBKR · type to search the broker for more" }
                    button { class: "mini", onclick: move |_| st.browser_open.set(false), "Close" }
                }
                div { class: "modal-tabs",
                    for m in markets {
                        button { class: if *market.read() == m.id { "tab active" } else { "tab" },
                            onclick: move |_| market.set(m.id.clone()),
                            "{m.name}"
                            if !m.tradable { span { class: "muted xs", " (view only)" } }
                        }
                    }
                }
                input { class: "modal-search", placeholder: "Search symbol or name, e.g. AAPL, Apple, EURUSD", value: "{q}", autofocus: true,
                    oninput: move |e| q.set(e.value()) }
                div { class: "modal-body",
                    if *loading.read() { p { class: "empty small", "Searching..." } }
                    table { class: "grid-tbl",
                        thead { tr { th { "Symbol" } th { "Name" } th { "Exchange" } th { "" } } }
                        tbody {
                            for inst in results.read().clone() {
                                {
                                    let key = inst.key.clone();
                                    let added = in_wl.contains(&key);
                                    rsx! {
                                        tr {
                                            td { class: "sym", span { class: "{market_tag(&inst.market)} xs", "{market_short(&inst.market)}" } "{inst.symbol}" }
                                            td { class: "muted", "{inst.name}" }
                                            td { class: "muted num", "{inst.exchange}" }
                                            td { class: "r",
                                                if added { span { class: "muted small", "In watchlist" } } else {
                                                    button { class: "mini accent", onclick: move |_| {
                                                        let k = key.clone();
                                                        spawn(async move {
                                                            match add_watch(k.clone()).await {
                                                                Ok(items) => { st.watchlist.set(items); st.toast.set(Some((true, format!("Added {k}")))); }
                                                                Err(e) => st.toast.set(Some((false, format!("Add failed: {e}")))),
                                                            }
                                                        });
                                                    }, "+ Add" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------- chart (Lightweight Charts bridge, one per pane) ----------------
#[component]
pub fn ChartGrid() -> Element {
    let mut st = app_state();
    let layout = *st.layout.read();
    let full = *st.chart_full.read();
    let n = match layout { 2 => 2, 4 => 4, _ => 1 };
    rsx! {
        div { class: "charts-col",
            div { class: "layout-bar",
                span { class: "muted xs", "Layout" }
                button { class: if layout == 1 { "lay active" } else { "lay" }, title: "Single chart", onclick: move |_| { st.layout.set(1); st.active_pane.set(0); }, "▭" }
                button { class: if layout == 2 { "lay active" } else { "lay" }, title: "Two charts", onclick: move |_| st.layout.set(2), "▯▯" }
                button { class: if layout == 4 { "lay active" } else { "lay" }, title: "Four charts", onclick: move |_| st.layout.set(4), "▦" }
                span { class: "muted xs hint", "click a chart to make it active · watchlist loads into the active chart" }
            }
            div { class: "charts layout-{n}",
                for i in 0..n {
                    Chart { pane: i, hidden: full.map(|f| f != i).unwrap_or(false) }
                }
            }
        }
    }
}

#[component]
pub fn Chart(pane: usize, hidden: bool) -> Element {
    let mut st = app_state();
    let key = st.pane_key[pane].read().clone();
    let tf = st.pane_tf[pane].read().clone();
    let full = *st.chart_full.read() == Some(pane);
    let active = *st.active_pane.read() == pane;
    let loading = *st.pane_loading[pane].read();
    let n_bars = st.pane_bars[pane].read().len();
    let quote = st.quotes.read().iter().find(|q| q.key == key).cloned().unwrap_or_default();
    let dec = st.dec_for(&key);
    let symbol = if quote.symbol.is_empty() { key.split(':').nth(1).unwrap_or(&key).to_string() } else { quote.symbol.clone() };
    let name = quote.name.clone();
    let cur = if quote.mid > 0.0 { quote.mid } else { quote.last };
    let mut show_ind = use_signal(|| false);
    let el_id = format!("qx-chart-{pane}");
    let sub_id = format!("qx-sub-{pane}");

    // init (create the chart instance for this pane)
    {
        let el_id = el_id.clone(); let sub_id = sub_id.clone();
        use_effect(move || { js_call(pane, &format!("I.init('{el_id}', '{sub_id}');"), false); });
    }
    // load bars when key/timeframe changes, and refresh every 60 s
    use_effect(move || {
        let key = st.pane_key[pane].read().clone();
        let (bs, dur) = tf_params(&st.pane_tf[pane].read());
        spawn(async move {
            st.pane_loading[pane].set(true);
            match get_bars(key.clone(), bs, dur).await {
                Ok(b) => { if let Some(d) = b.decimals { st.bar_dec.write().insert(key, d); } st.pane_bars[pane].set(b.bars) }
                Err(e) => { st.pane_bars[pane].set(Vec::new()); st.toast.set(Some((false, format!("Chart: {e}")))); }
            }
            st.pane_loading[pane].set(false);
        });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop {
            crate::sleep_ms(60000).await;
            let key = st.pane_key[pane].read().clone();
            let (bs, dur) = tf_params(&st.pane_tf[pane].read());
            if let Ok(b) = get_bars(key, bs, dur).await { st.pane_bars[pane].set(b.bars); }
        }
    });
    // bars → chart
    use_effect(move || {
        let bars = st.pane_bars[pane].read().clone();
        let key = st.pane_key[pane].read().clone();
        let tf = st.pane_tf[pane].read().clone();
        let dec = st.dec_for(&key);
        let payload = serde_json::json!({ "key": key, "decimals": dec, "tfSec": tf_seconds(&tf), "bars": bars });
        js_call(pane, &format!("I.setData({payload});"), true);
    });
    // live tick — stamped with the quote's own time, so an old quote (market closed) never opens new candles
    use_effect(move || {
        let key = st.pane_key[pane].read().clone();
        if let Some(q) = st.quotes.read().iter().find(|q| q.key == key) {
            let p = if q.mid > 0.0 { q.mid } else { q.last };
            let ts = q.time.as_deref().map(iso_epoch).unwrap_or(0.0).floor();
            if p > 0.0 && p.is_finite() && ts > 0.0 { js_call(pane, &format!("I.tick({p}, {ts});"), true); }
        }
    });
    // indicators (shared by all panes)
    use_effect(move || {
        let cfg = st.indicators.read().clone();
        js_call(pane, &format!("I.setIndicators({});", serde_json::to_string(&cfg).unwrap_or("{}".into())), true);
    });
    // fills / position / orders
    use_effect(move || {
        let key = st.pane_key[pane].read().clone();
        let manual = *st.mode.read() == "manual";
        let can_trade = st.can_trade();
        if let Some(snap) = st.snapshot.read().as_ref() {
            let markers: Vec<serde_json::Value> = snap.fills.iter().filter(|f| f.key == key && f.time.is_some() && finite_opt(f.price).is_some())
                .map(|f| serde_json::json!({ "t": f.time, "side": if f.side == "BOT" { "BUY" } else { "SELL" }, "price": f.price, "qty": qty_fmt(f.qty) })).collect();
            let pos = snap.positions.iter().find(|p| p.key == key)
                .filter(|p| !p.unpriced)
                .map(|p| serde_json::json!({ "price": p.avg_cost, "qty": p.position, "pnl": p.unrealized_pnl, "ccy": if p.usd { String::new() } else { p.currency.clone() } }))
                .unwrap_or(serde_json::Value::Null);
            // TRAIL orders carry a trail DISTANCE in stop_price, not a price level → no line for them
            let orders: Vec<serde_json::Value> = snap.orders.iter().filter(|o| o.key == key && o.order_type != "TRAIL").filter_map(|o| {
                let price = finite_opt(o.limit_price.or(o.stop_price))?;
                let kind = if o.parent_id.is_some() { if o.order_type == "LMT" { "TP" } else { "SL" } } else { o.order_type.as_str() };
                // only manual orders can be dragged, and only in Manual mode by a role that may trade
                let draggable = manual && can_trade && order_is_manual(o);
                Some(serde_json::json!({ "id": o.order_id, "price": price, "side": o.side, "qty": qty_fmt(o.qty), "kind": kind, "draggable": draggable }))
            }).collect();
            js_call(pane, &format!("I.setMarkers({}); I.setLines({});",
                serde_json::Value::Array(markers), serde_json::json!({ "position": pos, "orders": orders })), true);
        }
    });
    // size changed (layout / fullscreen) → resize
    use_effect(move || {
        let _ = st.layout.read(); let _ = st.chart_full.read();
        js_call(pane, "setTimeout(() => I.resize(), 50);", true);
    });

    let ch = st.crosshair.read().clone().filter(|c| c.pane == pane);
    let ind = st.indicators.read().clone();
    let cls = format!("panel chart{}{}{}", if full { " fullscreen" } else { "" }, if active { " active" } else { "" }, if hidden { " hidden-pane" } else { "" });

    rsx! {
        section { class: "{cls}", onmousedown: move |_| { st.active_pane.set(pane); let k = st.pane_key[pane].read().clone(); st.selected.set(k); },
            div { class: "chart-head",
                div { class: "chart-id",
                    span { class: "pane-no", "{pane + 1}" }
                    span { class: "{market_tag(&quote.market)}", "{market_short(&quote.market)}" }
                    span { class: "chart-sym", "{symbol}" }
                    span { class: "chart-name muted", "{name}" }
                    span { class: "chart-px num", "{px(cur, dec)}" }
                    span { class: "num {pnl_class(quote.change_pct)}", "{chg_pct(quote.change_pct, quote.prev_close)}" }
                }
                div { class: "chart-tools",
                    div { class: "tfs",
                        for (id, label, _, _) in TIMEFRAMES {
                            button { class: if tf == id { "tf active" } else { "tf" }, onclick: move |_| st.pane_tf[pane].set(id.to_string()), "{label}" }
                        }
                    }
                    button { class: if *show_ind.read() { "tool wide active" } else { "tool wide" }, onclick: move |_| show_ind.toggle(), "Indicators" }
                    div { class: "zooms",
                        button { class: "tool", title: "Zoom out", onclick: move |_| js_call(pane, "I.zoom(1.4);", true), "−" }
                        button { class: "tool", title: "Zoom in", onclick: move |_| js_call(pane, "I.zoom(1.0/1.4);", true), "+" }
                        button { class: "tool", title: "Reset view", onclick: move |_| js_call(pane, "I.fit();", true), "⟲" }
                        button { class: "tool", title: if full { "Exit fullscreen" } else { "Fullscreen" },
                            onclick: move |_| { let cur = *st.chart_full.peek(); st.chart_full.set(if cur == Some(pane) { None } else { Some(pane) }); },
                            if full { "✕" } else { "⛶" } }
                    }
                }
            }
            div { class: "ohlc num",
                match ch {
                    Some(c) => {
                        let f = |v: f64| format!("{:.*}", dec, v);
                        let (o, h, l, cl, vol) = (f(c.o), f(c.h), f(c.l), f(c.c), qty_fmt(c.v));
                        let up = c.c >= c.o;
                        let ind_items: Vec<(String, String)> = c.ind.iter()
                            .map(|(k, v)| (k.clone(), v.as_f64().map(|x| format!("{:.*}", dec, x)).unwrap_or_default())).collect();
                        rsx! {
                            span { class: "muted", "O" } span { class: if up { "text-up" } else { "text-down" }, "{o}" }
                            span { class: "muted", "H" } span { "{h}" }
                            span { class: "muted", "L" } span { "{l}" }
                            span { class: "muted", "C" } span { class: if up { "text-up" } else { "text-down" }, "{cl}" }
                            span { class: "muted", "V" } span { "{vol}" }
                            for (k, v) in ind_items {
                                span { class: "muted", "{k}" } span { "{v}" }
                            }
                        }
                    },
                    None => rsx! { span { class: "muted", if loading { "Loading {n_bars} bars..." } else { "{tf} · {n_bars} bars · hover for OHLC · right-click to trade at a price · drag order lines to move them" } } },
                }
            }
            if *show_ind.read() { IndicatorMenu {} }
            div { class: "chart-area",
                div { id: "{el_id}", class: "lwc-main" }
                div { id: "{sub_id}", class: "lwc-sub", style: if ind.sub == "none" { "display:none" } else { "" } }
            }
        }
    }
}

/// Receive events from the JS of every pane (rendered once in App)
#[component]
pub fn ChartEvents() -> Element {
    let mut st = app_state();
    use_future(move || async move {
        let mut ev = document::eval("window.__qxSend = (m) => dioxus.send(m); (window.__qxQueue || []).forEach((m) => dioxus.send(m)); window.__qxQueue = []; await new Promise(() => {});");
        loop {
            match ev.recv::<serde_json::Value>().await {
                Ok(v) => {
                    let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
                    let pane = v.get("pane").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                    match t {
                        "crosshair" => {
                            let mut ch: Crosshair = serde_json::from_value(v.clone()).unwrap_or_default();
                            ch.pane = pane;
                            st.crosshair.set(if ch.visible { Some(ch) } else { None });
                        }
                        "ctx" => {
                            let g = |k: &str| v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0);
                            let line = v.get("line").cloned().unwrap_or(serde_json::Value::Null);
                            if pane < st.pane_key.len() {
                                st.active_pane.set(pane);
                                let k = st.pane_key[pane].read().clone();
                                st.selected.set(k);
                            }
                            st.ctx_menu.set(Some(CtxMenu {
                                x: g("x"), y: g("y"), price: g("price"),
                                line_kind: line.get("kind").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                line_id: line.get("id").map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).unwrap_or_default(),
                                line_title: line.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                pane,
                            }));
                        }
                        "ws_state" => {
                            let open = v.get("open").and_then(|x| x.as_bool()).unwrap_or(false);
                            st.ws_open.set(open);
                        }
                        "ws" => {
                            let m = v.get("msg").cloned().unwrap_or(serde_json::Value::Null);
                            handle_ws(&mut st, m);
                        }
                        "hotkey" => {
                            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                            handle_hotkey(&mut st, &name);
                        }
                        "order_move" => {
                            // a dragged order line: re-check ownership/mode, round to the instrument grid, then ask before modifying
                            let id = v.get("order_id").and_then(|x| x.as_i64()).unwrap_or(0);
                            let price = v.get("price").and_then(|x| x.as_f64()).unwrap_or(0.0);
                            let mut from = v.get("from").and_then(|x| x.as_f64()).unwrap_or(0.0);
                            // dragged again while its "Move order?" is still open → cancelling must restore the ORIGINAL price
                            if let Some((_, pid, pfrom)) = revert_on_cancel(st.confirm.peek().as_ref()) { if pid == id { from = pfrom; } }
                            let order = st.snapshot.peek().as_ref().and_then(|s| s.orders.iter().find(|o| o.order_id == id).cloned());
                            let allowed = *st.mode.peek() == "manual" && st.can_trade();
                            match order {
                                Some(o) if allowed && order_is_manual(&o) && o.order_type != "TRAIL" && price > 0.0 && price.is_finite() => {
                                    let dec = st.dec_for(&o.key);
                                    let to = crate::calc::round_to_dec(price, dec);
                                    st.ask("Move order?", format!("{} {} {} {} #{id}\nfrom {} to {}", o.side, qty_fmt(o.qty), o.symbol, o.order_type, px(from, dec), px(to, dec)),
                                           "Move order", ConfirmAction::MoveOrder { id, from, to, pane });
                                }
                                _ => {
                                    if from > 0.0 { js_call(pane, &format!("I.moveLine({id}, {from});"), true); }
                                    st.toast.set(Some((false, "This order cannot be moved from the chart (Manual mode, manual orders only)".into())));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => break,
            }
        }
    });
    rsx! { ChartContextMenu {} HelpModal {} AlertDialog {} NotifCenter {} ConfirmDialog {} }
}

/// Realtime messages from the API (/ws) — polling keeps running as a fallback
fn handle_ws(st: &mut State, m: serde_json::Value) {
    let t = m.get("type").and_then(|x| x.as_str()).unwrap_or("");
    match t {
        "quotes" => {
            if let Ok(qs) = serde_json::from_value::<Vec<Quote>>(m.get("quotes").cloned().unwrap_or_default()) {
                let prev: HashMap<String, f64> = st.quotes.peek().iter().map(|q| (q.key.clone(), q.mid)).collect();
                st.prev_mid.set(prev);
                st.quotes.set(qs);
                st.set_err("quotes", None);
                st.ws_quote_seq += 1;
            }
        }
        "snapshot" => {
            let has_rec = m.get("reconcile").map(|v| !v.is_null()).unwrap_or(false);
            let has_nodes = m.get("running_nodes").is_some();
            let acct_flag_bad = m.get("account_ok").and_then(|x| x.as_bool()) == Some(false);
            if let Ok(mut s) = serde_json::from_value::<Snapshot>(m.clone()) {
                if acct_flag_bad { s.account = Account::default(); }
                st.apply_snapshot(s, has_rec, has_nodes);
                st.ws_snap_seq += 1;
            }
        }
        "fill" => {
            let g = |k: &str| m.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
            let qty = m.get("qty").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let px = m.get("price").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let side = if g("side") == "BOT" { "BUY" } else { "SELL" };
            st.notify("fill", format!("Filled {side} {} {} @ {} · {}", qty_fmt(qty), g("key"), px, g("ref")));
            st.refresh_tick += 1;
        }
        "order_status" => {
            let status = m.get("status").and_then(|x| x.as_str()).unwrap_or("");
            if matches!(status, "Cancelled" | "Inactive" | "ApiCancelled") {
                let id = m.get("order_id").and_then(|x| x.as_i64()).unwrap_or(0);
                st.notify("info", format!("Order #{id} {status}"));
            }
            st.refresh_tick += 1;
        }
        "alert" => {
            let text = m.get("text").and_then(|x| x.as_str()).unwrap_or("Price alert").to_string();
            st.notify("alert", text.clone());
            st.toast.set(Some((true, text)));
            let mut st = *st;
            spawn(async move { if let Ok(a) = get_alerts().await { st.alerts.set(a); } });
        }
        "node_event" => {
            let ev = m.get("event").cloned().unwrap_or(serde_json::Value::Null);
            let t = ev.get("type").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let node = ev.get("node").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if t == "heartbeat" {
                st.node_live.write().insert(node, ev);
                return;
            }
            let d = ev.get("data").cloned().unwrap_or(serde_json::Value::Null);
            let g = |k: &str| d.get(k).map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).unwrap_or_default();
            let inst = ev.get("instrument").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if t == "fill" {
                let qty = d.get("qty").and_then(|x| x.as_f64()).unwrap_or(0.0);
                st.notify("fill", format!("Node {node}: filled {} {} {inst} @ {}", g("side"), qty_fmt(qty), g("price")));
            } else if t == "order" && matches!(g("event").as_str(), "Rejected" | "Denied") {
                st.notify("error", format!("Node {node}: order {} {inst} — {}", g("event").to_lowercase(), g("reason")));
            } else if t == "risk" && matches!(g("event").as_str(), "kill" | "halt") {
                st.notify("error", format!("Node {node}: {} — {}", g("event"), g("reason")));
            } else if t == "node" && g("event") == "stopping" {
                st.node_live.write().remove(&node);
            }
            let mut w = st.node_events.write();
            w.insert(0, ev);
            w.truncate(300);
        }
        "assistant_action" => {
            let text = m.get("text").and_then(|x| x.as_str()).unwrap_or("AI assistant action").to_string();
            st.notify("alert", text);
            st.refresh_tick += 1;
        }
        "conn" => {
            if let Ok(c) = serde_json::from_value::<ConnInfo>(m.clone()) {
                let prev_state = st.conn.peek().as_ref().map(|c| c.state.clone()).unwrap_or_default();
                if prev_state != c.state && !prev_state.is_empty() {
                    if c.state == "connected" { st.notify("info", format!("IB Gateway reconnected (#{})", c.reconnects)); st.refresh_tick += 1; }
                    else { st.notify("disconnect", "IB Gateway connection lost — reconnecting…"); }
                }
                st.conn.set(Some(c));
            }
        }
        _ => {}
    }
}

/// Hotkeys (captured in JS and forwarded here)
fn handle_hotkey(st: &mut State, name: &str) {
    let manual = *st.mode.peek() == "manual";
    match name {
        "escape" => { cancel_confirm(st); st.perf_full.set(false); st.ctx_menu.set(None); st.help_open.set(false); st.alert_dialog.set(None); st.notif_open.set(false); st.browser_open.set(false); st.heat_group.set(None); st.node_detail.set(None); if st.chart_full.peek().is_some() { st.chart_full.set(None); } }
        "help" => { let v = !*st.help_open.peek(); st.help_open.set(v); }
        "buy" | "sell" => {
            if manual && st.can_trade() {
                let side = if name == "buy" { "BUY" } else { "SELL" };
                st.ticket_prefill.set(Some(Prefill { side: side.into(), otype: "MKT".into(), price: 0.0 }));
            } else { st.toast.set(Some((false, "Switch to Manual mode to use F1/F2".into()))); }
        }
        "enter" => { if st.can_trade() && st.confirm.peek().is_none() { st.submit_req += 1; } }
        "close_all" => { st.bottom_tab.set(0); let h = (*st.bottom_h.peek()).max(BOTTOM_DEFAULT); st.bottom_h.set(h); st.toast.set(Some((false, "Close All: confirm with the button in the Positions tab".into()))); }
        "kill" => {
            // same two-step confirmation as the header button
            if st.can_kill() {
                st.ask("Activate the KILL SWITCH?", "Stops all nodes, cancels all orders, closes every position at market and halts trading on ALL accounts.", KILL_CONFIRM_LABEL, ConfirmAction::Kill);
            } else { st.toast.set(Some((false, format!("Your role ({}) cannot use the kill switch", st.role())))); }
        }
        "fullscreen" => {
            let cur = *st.chart_full.peek();
            st.chart_full.set(if cur.is_some() { None } else { Some(*st.active_pane.peek()) });
        }
        "layout1" => { st.layout.set(1); st.active_pane.set(0); }
        "layout2" => st.layout.set(2),
        "layout4" => st.layout.set(4),
        "zoom_in" | "zoom_out" => { let f = if name == "zoom_in" { 0.7 } else { 1.4 }; js_call(*st.active_pane.peek(), &format!("I.zoom({f});"), true); }
        "ui_zoom_in" | "ui_zoom_out" | "ui_zoom_reset" => {
            let z = crate::calc::next_ui_zoom(ui_zoom_get(), name);
            ui_zoom_set(z);
            st.toast.set(Some((true, format!("Zoom {:.0}%", z * 100.0))));
        }
        "wl_up" | "wl_down" => {
            let keys: Vec<String> = st.visible_watchlist().into_iter().map(|i| i.key).collect();
            if keys.is_empty() { return; }
            let cur = st.selected.peek().clone();
            let i = keys.iter().position(|k| *k == cur).unwrap_or(0) as i64;
            let j = (i + if name == "wl_up" { -1 } else { 1 }).rem_euclid(keys.len() as i64) as usize;
            st.select(keys[j].clone());
        }
        "alert" => {
            let key = st.selected.peek().clone();
            let px = st.quotes.peek().iter().find(|q| q.key == key).map(|q| q.mid).unwrap_or(0.0);
            if px > 0.0 { st.alert_dialog.set(Some(px)); }
        }
        _ => {}
    }
}

// ---------------- round 6 components ----------------
#[component]
fn ConnPill(ib_ok: bool, api_ok: bool, last: String) -> Element {
    let st = app_state();
    let conn = st.conn.read().clone();
    let ws = *st.ws_open.read();
    let state = conn.as_ref().map(|c| c.state.clone()).unwrap_or_else(|| if ib_ok { "connected".into() } else { "disconnected".into() });
    // one status pill: the engine API first (nothing works without it), then the IBKR connection
    let (cls, label) = if !api_ok { ("pill bad", "API offline".to_string()) } else { match state.as_str() {
        "connected" => ("pill ok", "IBKR".to_string()),
        "reconnecting" => ("pill warn", "IBKR …".to_string()),
        _ => ("pill bad", "IBKR".to_string()),
    } };
    let rec = conn.as_ref().map(|c| c.reconnects).unwrap_or(0);
    let title = format!("API {} · IBKR {} · reconnects: {} · realtime: {} · last quote {} UTC", if api_ok { "ok" } else { "offline" }, state, rec, if ws { "websocket" } else { "polling" }, last);
    rsx! {
        span { class: "{cls}", title: "{title}", "{label}" }
        span { class: if ws { "dot live" } else { "dot" }, title: if ws { "Realtime stream connected" } else { "Polling fallback (websocket offline)" } }
    }
}

#[component]
fn PresetSwitch() -> Element {
    let mut st = app_state();
    let mut open = use_signal(|| false);
    let cur = st.preset.read().clone();
    rsx! {
        div { class: "preset-dd",
            button { class: if *open.read() { "preset-btn open" } else { "preset-btn" }, title: "Layout preset",
                onclick: move |_| { let v = !*open.peek(); open.set(v); },
                span { class: "preset-ic", "▦" } span { class: "preset-name", "{cur}" } span { class: "caret", "▾" }
            }
            if *open.read() {
                div { class: "ctx-bg", onclick: move |_| open.set(false) }
                div { class: "preset-menu",
                    for (name, desc) in [("Trading", "1 chart · ticket · positions"), ("Research", "2 charts · performance"), ("Monitoring", "4 charts · wide watchlist")] {
                        button { class: if cur == name { "preset-item active" } else { "preset-item" },
                            onclick: move |_| { st.apply_preset(name); open.set(false); },
                            span { class: "chk-mark", if cur == name { "✓" } else { "" } }
                            span { class: "pi-text", span { class: "pi-name", "{name}" } span { class: "pi-desc", "{desc}" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NotifBell() -> Element {
    let mut st = app_state();
    let unread = st.notifs.read().iter().filter(|n| !n.read).count();
    let snd = *st.sound.read();
    rsx! {
        button { class: if unread > 0 { "bell has" } else { "bell" }, title: "Notification center",
            onclick: move |_| { let v = !*st.notif_open.peek(); st.notif_open.set(v); if v { for n in st.notifs.write().iter_mut() { n.read = true; } } },
            "🔔" if unread > 0 { span { class: "badge", "{unread}" } }
        }
        button { class: "tool", title: if snd { "Sounds on — click to mute" } else { "Sounds muted" }, onclick: move |_| { let v = !*st.sound.peek(); st.sound.set(v); }, if snd { "🔊" } else { "🔇" } }
    }
}

#[component]
fn NotifCenter() -> Element {
    let mut st = app_state();
    if !*st.notif_open.read() { return rsx! {}; }
    let items = st.notifs.read().clone();
    rsx! {
        div { class: "ctx-bg", onclick: move |_| st.notif_open.set(false) }
        div { class: "notif-center",
            div { class: "panel-title", "Notifications" span { class: "muted", "{items.len()}" }
                button { class: "mini", style: "margin-left:auto", onclick: move |_| st.notifs.set(vec![]), "Clear" } }
            if items.is_empty() { p { class: "empty", "No notifications yet" } }
            for n in items {
                div { class: "notif {n.kind}", span { class: "muted num", "{n.ts}" } span { "{n.text}" } }
            }
        }
    }
}

#[component]
fn HelpModal() -> Element {
    let mut st = app_state();
    if !*st.help_open.read() { return rsx! {}; }
    let rows: [(&str, &str); 15] = [
        ("F1 / B", "Buy — prefill ticket (Manual mode)"), ("F2 / S", "Sell — prefill ticket (Manual mode)"), ("Enter", "Confirm / submit the order ticket"),
        ("Esc", "Close menus, dialogs and fullscreen"), ("1 · 2 · 4", "Chart layout: single / two / four panes"), ("↑ / ↓", "Previous / next instrument in the watchlist"),
        ("+ / −", "Zoom active chart in / out"), ("A", "Price alert at the current price"), ("Ctrl/⌘ + Shift + F", "Toggle chart fullscreen"),
        ("Ctrl/⌘ + Shift + X", "Jump to Positions tab for Close All"), ("Ctrl/⌘ + Shift + K", "Kill switch — halt all trading (asks to confirm)"), ("Right-click chart", "Trade / alert / horizontal line at price"),
        ("Drag price line", "Move a working manual order (asks to confirm)"), ("Ctrl/⌘ + = / − / 0", "Zoom the whole Terminal in / out / reset"), ("?", "This help"),
    ];
    rsx! {
        div { class: "modal-bg", onclick: move |_| st.help_open.set(false),
            div { class: "modal help", onclick: move |e| e.stop_propagation(),
                div { class: "panel-title", "Keyboard shortcuts" button { class: "mini", style: "margin-left:auto", onclick: move |_| st.help_open.set(false), "Close" } }
                table { class: "grid-tbl help-tbl", tbody {
                    for (k, d) in rows { tr { td { kbd { "{k}" } } td { "{d}" } } }
                } }
            }
        }
    }
}

static UI_ZOOM_BITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0x3FF0_0000_0000_0000);

/// After sign-in: reconnect the live stream with the new session (desktop: the address carries the token; web: the
/// browser sends the session cookie)
async fn reconnect_live_stream() {
    let u = get_ws_url().await.unwrap_or_default();
    let js = serde_json::to_string(&u).unwrap_or_default();
    let _ = document::eval(&format!("window.qxWs && window.qxWs.reconnect({js})"));
}

fn ui_zoom_get() -> f64 { f64::from_bits(UI_ZOOM_BITS.load(std::sync::atomic::Ordering::Relaxed)) }

/// Zoom the whole Terminal with native page zoom (desktop) and remember it for next start.
/// The web build leaves zoom to the browser: CSS zoom scales vw/vh and canvas coordinates, which pushed the layout off screen.
fn ui_zoom_set(z: f64) {
    UI_ZOOM_BITS.store(z.to_bits(), std::sync::atomic::Ordering::Relaxed);
    #[cfg(feature = "desktop")]
    {
        let _ = dioxus::desktop::window().webview.zoom(z);
        let _ = document::eval(&format!("try {{ localStorage.setItem('qx_ui_zoom', '{z}'); }} catch (e) {{}}"));
    }
}

/// Restore the zoom saved by ui_zoom_set (called once at start)
pub fn ui_zoom_restore() {
    #[cfg(not(feature = "desktop"))]
    {
        let _ = document::eval("window.QX_BROWSER_ZOOM = true; document.documentElement.style.zoom = ''; try { localStorage.removeItem('qx_ui_zoom'); } catch (e) {}");
        return;
    }
    #[allow(unreachable_code)]
    spawn(async move {
        let r = document::eval("let v = null; try { v = localStorage.getItem('qx_ui_zoom'); } catch (e) {} return v || '1';").await;
        if let Ok(v) = r {
            if let Some(z) = v.as_str().and_then(crate::calc::parse_ui_zoom) { if (z - 1.0).abs() > 1e-9 { ui_zoom_set(z); } }
        }
    });
}

// ---------------- dropdown (Terminal-styled replacement for the native <select>) ----------------
/// Options as (value, label) pairs
pub type DdOpts = Vec<(String, String)>;

pub fn dd_opts<V: ToString, L: ToString>(items: impl IntoIterator<Item = (V, L)>) -> DdOpts {
    items.into_iter().map(|(v, l)| (v.to_string(), l.to_string())).collect()
}

/// Label shown on the closed dropdown: the option's label, else the raw value
pub fn dd_label(options: &DdOpts, value: &str) -> String {
    options.iter().find(|(v, _)| v == value).map(|(_, l)| l.clone()).unwrap_or_else(|| value.to_string())
}

/// Where the open list goes: below the button unless there is clearly more room above.
/// Returns (open_up, max_height) for a list of `n` rows in a viewport `vh` px high.
pub fn dd_place(btn_top: f64, btn_bottom: f64, vh: f64, n: usize) -> (bool, f64) {
    let want = (n as f64 * 26.0 + 8.0).min(320.0);
    let below = (vh - btn_bottom - 8.0).max(0.0);
    let above = (btn_top - 8.0).max(0.0);
    if below >= want || below >= above { (false, below.min(320.0).max(60.0)) } else { (true, above.min(320.0).max(60.0)) }
}

#[component]
pub fn Dropdown(
    value: String,
    options: DdOpts,
    onchange: EventHandler<String>,
    #[props(default)] class: String,
    #[props(default)] disabled: bool,
    #[props(default)] title: String,
) -> Element {
    let mut open = use_signal(|| false);
    let mut hi = use_signal(|| 0usize);
    let mut el = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut place = use_signal(String::new);
    let label = dd_label(&options, &value);
    let n = options.len();
    let cur_idx = options.iter().position(|(v, _)| *v == value).unwrap_or(0);
    let mut show = move |start: usize| {
        if disabled || n == 0 { return; }
        hi.set(start);
        spawn(async move {
            let Some(m) = el.peek().clone() else { return };
            let Ok(r) = m.get_client_rect().await else { return };
            let vh = document::eval("return window.innerHeight;").await.ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
            let (x, y, w, h) = (r.origin.x, r.origin.y, r.size.width, r.size.height);
            let (up, maxh) = dd_place(y, y + h, vh, n);
            let vert = if up { format!("bottom: {:.0}px;", vh - y + 2.0) } else { format!("top: {:.0}px;", y + h + 2.0) };
            place.set(format!("left: {x:.0}px; min-width: {w:.0}px; max-height: {maxh:.0}px; {vert}"));
            open.set(true);
        });
    };
    let opts2 = options.clone();
    rsx! {
        div { class: "dd {class}", title: "{title}",
            button { class: if *open.read() { "dd-btn open" } else { "dd-btn" }, r#type: "button", disabled,
                onmounted: move |e| el.set(Some(e.data())),
                onclick: move |_| { if *open.peek() { open.set(false); } else { show(cur_idx); } },
                onkeydown: move |e| {
                    let k = e.key();
                    let is_open = *open.peek();
                    match k {
                        Key::ArrowDown | Key::ArrowUp => {
                            e.prevent_default();
                            if !is_open { show(cur_idx); return; }
                            let i = *hi.peek();
                            hi.set(if k == Key::ArrowDown { (i + 1).min(n.saturating_sub(1)) } else { i.saturating_sub(1) });
                        }
                        Key::Enter => {
                            e.prevent_default();
                            if is_open {
                                if let Some((v, _)) = opts2.get(*hi.peek()) { onchange.call(v.clone()); }
                                open.set(false);
                            } else { show(cur_idx); }
                        }
                        Key::Escape | Key::Tab => { if is_open { open.set(false); } }
                        _ => {}
                    }
                },
                span { class: "dd-val", "{label}" }
                span { class: "dd-caret", "▾" }
            }
            if *open.read() {
                div { class: "dd-bg", onclick: move |e| { e.prevent_default(); e.stop_propagation(); open.set(false); } }
                div { class: "dd-menu", style: "{place}", role: "listbox",
                    for (i, (v, l)) in options.iter().cloned().enumerate() {
                        div {
                            key: "{v}",
                            class: if v == value { if i == *hi.read() { "dd-opt sel hi" } else { "dd-opt sel" } } else if i == *hi.read() { "dd-opt hi" } else { "dd-opt" },
                            onmouseenter: move |_| hi.set(i),
                            // inside a <label> the click would also "activate" the label → its button → the list reopens
                            onclick: move |e| { e.prevent_default(); e.stop_propagation(); onchange.call(v.clone()); open.set(false); },
                            span { class: "dd-check", if v == value { "✓" } }
                            span { "{l}" }
                        }
                    }
                }
            }
        }
    }
}

/// Close the confirmation dialog without running its action (a dragged order line goes back to its price)
pub fn cancel_confirm(st: &mut State) {
    if let Some((pane, id, from)) = revert_on_cancel(st.confirm.peek().as_ref()) {
        js_call(pane, &format!("I.moveLine({id}, {from});"), true);
    }
    if st.confirm.peek().is_some() { st.confirm.set(None); }
}

/// Order line to put back when a confirmation is cancelled or replaced: (pane, order id, original price)
pub fn revert_on_cancel(req: Option<&ConfirmReq>) -> Option<(usize, i64, f64)> {
    match req { Some(ConfirmReq { action: ConfirmAction::MoveOrder { id, from, pane, .. }, .. }) => Some((*pane, *id, *from)), _ => None }
}

/// Tooltip of the PARTIAL tag on "All accounts" equity / stats
pub const PARTIAL_TIP: &str = "Some accounts have no data for part of this range; their first/last value is held flat.";

/// The kill switch halts every account (server default since round 2) — the confirm button says so
pub const KILL_CONFIRM_LABEL: &str = "CONFIRM KILL — ALL accounts";

/// Runs a confirmed action (kill switch, close / reverse position, stop node, move / cancel order)
async fn run_confirmed(mut st: State, action: ConfirmAction) {
    match action {
        ConfirmAction::Kill => match kill_switch().await {
            Ok(_) => { st.mode.set("manual".into()); st.notify("error", "KILL SWITCH engaged"); st.toast.set(Some((true, "KILL SWITCH: agents stopped, orders cancelled, positions closed, trading halted".into()))); }
            Err(e) => st.toast.set(Some((false, format!("Kill switch failed: {e}")))),
        },
        ConfirmAction::ClosePos(k, force) => match close_position(k.clone(), force).await {
            Ok(r) => st.toast.set(Some((true, format!("Closed {k}: {} @ {}", r.status, px(r.avg_fill_price, st.dec_for(&k)))))),
            // the API refuses (409) when a running node holds the position and force was not sent → offer "Close anyway"
            Err(e) if !force && e.to_string().contains("409") => {
                st.ask("Position held by a running node", format!("{e}\n\nThe node keeps running and may re-open or double-close it. Stopping the node first is safer (it closes its own positions)."),
                       "Close anyway", ConfirmAction::ClosePos(k, true));
            }
            Err(e) => st.toast.set(Some((false, format!("Close failed: {e}")))),
        },
        ConfirmAction::Reverse(k) => match reverse_position(k.clone()).await {
            Ok(r) => st.toast.set(Some((true, format!("Reversed {k}: {}", r.status)))),
            Err(e) => st.toast.set(Some((false, format!("Reverse failed: {e}")))),
        },
        ConfirmAction::NodeStop(id) => match node_action(id.clone(), "stop".into()).await {
            Ok(s) => st.toast.set(Some((true, format!("{id}: stop → {s}")))),
            Err(e) => st.toast.set(Some((false, format!("{id}: stop failed — {e}")))),
        },
        ConfirmAction::MoveOrder { id, from, to, pane } => match modify_order(id, None, Some(to)).await {
            Ok(r) => { js_call(pane, &format!("I.releaseLine({id});"), true); st.toast.set(Some((true, format!("Order {id} moved to {}: {}", to, r.status)))) }
            Err(e) => { js_call(pane, &format!("I.moveLine({id}, {from});"), true); st.toast.set(Some((false, format!("Move failed: {e}")))) }
        },
        ConfirmAction::ModifyOrder { id, qty, price } => match modify_order(id, qty, price).await {
            Ok(r) => st.toast.set(Some((true, format!("Order {id} updated: {}", r.status)))),
            Err(e) => st.toast.set(Some((false, format!("Modify failed: {e}")))),
        },
        ConfirmAction::ConfirmBaseline(n) => match confirm_daily_baseline(n).await {
            Ok(d) => st.toast.set(Some((true, format!("Daily-loss baseline confirmed: start NetLiq {} ({})", money(d.start_netliq), d.date)))),
            Err(e) => st.toast.set(Some((false, format!("Baseline not confirmed: {e}")))),
        },
        ConfirmAction::CancelOrder(id) => match cancel_order(id).await {
            Ok(_) => st.toast.set(Some((true, format!("Cancelled order {id}")))),
            Err(e) => st.toast.set(Some((false, format!("Cancel failed: {e}")))),
        },
    }
    if let Ok(r) = get_risk().await { st.risk.set(Some(r)); }
    st.refresh_tick += 1;
}

#[component]
fn ConfirmDialog() -> Element {
    let mut st = app_state();
    let Some(req) = st.confirm.read().clone() else { return rsx! {} };
    let action = req.action.clone();
    rsx! {
        div { class: "modal-bg", onclick: move |_| cancel_confirm(&mut st),
            div { class: "modal confirm-dlg", onclick: move |e| e.stop_propagation(),
                div { class: "panel-title", "{req.title}" }
                div { class: "confirm-body", "{req.body}" }
                div { class: "btn-row",
                    button { class: "mini", onclick: move |_| cancel_confirm(&mut st), "Cancel" }
                    button { class: "mini danger", onclick: move |_| {
                        let a = action.clone();
                        st.confirm.set(None);
                        spawn(run_confirmed(st, a));
                    }, "{req.ok_label}" }
                }
            }
        }
    }
}

#[component]
fn AlertDialog() -> Element {
    let mut st = app_state();
    let mut level = use_signal(String::new);
    let mut kind = use_signal(|| "above".to_string());
    let mut note = use_signal(String::new);
    let mut repeat = use_signal(|| false);
    // fill in defaults when the dialog opens
    use_effect(move || {
        let base = *st.alert_dialog.read();
        if let Some(b) = base {
            let key = st.selected.peek().clone();
            let (mid, dec) = st.quotes.peek().iter().find(|q| q.key == key).map(|q| (q.mid, if q.decimals > 0 { q.decimals } else { 2 })).unwrap_or((0.0, 5));
            level.set(px(b, dec));
            kind.set(if b >= mid { "above".to_string() } else { "below".to_string() });
            note.set(String::new());
        }
    });
    let open = st.alert_dialog.read().is_some();
    let key = st.selected.read().clone();
    let key_disp = key.clone();
    let close = move |_| st.alert_dialog.set(None);
    let save = move |_| {
        let lv: f64 = level.read().replace(',', "").parse().unwrap_or(0.0);
        if lv <= 0.0 { return; }
        let (k, kd, n, r) = (key.clone(), kind.read().clone(), note.read().clone(), *repeat.read());
        spawn(async move {
            match add_alert(k.clone(), kd.clone(), lv, n, r).await {
                Ok(_) => { st.toast.set(Some((true, format!("Alert set: {k} {kd} {lv}")))); if let Ok(a) = get_alerts().await { st.alerts.set(a); } }
                Err(e) => st.toast.set(Some((false, format!("Alert failed: {e}")))),
            }
            st.alert_dialog.set(None);
        });
    };
    if !open { return rsx! {}; }
    rsx! {
        div { class: "modal-bg", onclick: close,
            div { class: "modal alert-dlg", onclick: move |e| e.stop_propagation(),
                div { class: "panel-title", "Price alert" span { class: "muted", "{key_disp}" } }
                div { class: "fld-row",
                    label { class: "fld", span { "Condition" }
                        Dropdown { value: "{kind}", onchange: move |v| kind.set(v),
                            options: dd_opts([("above", "Price above"), ("below", "Price below"), ("change_pct", "Day change % beyond")]) } }
                    label { class: "fld", span { "Level" } input { class: "num", value: "{level}", oninput: move |e| level.set(e.value()) } }
                }
                label { class: "fld", span { "Note" } input { value: "{note}", placeholder: "optional", oninput: move |e| note.set(e.value()) } }
                label { class: "chk", input { r#type: "checkbox", checked: *repeat.read(), oninput: move |e| repeat.set(e.checked()) } span { "Re-arm after firing" } }
                div { class: "btn-row", button { class: "mini", onclick: close, "Cancel" } button { class: "mini primary", onclick: save, "Set alert" } }
            }
        }
    }
}

#[component]
fn AlertsTab() -> Element {
    let mut st = app_state();
    use_future(move || async move { if let Ok(a) = get_alerts().await { st.alerts.set(a); } });
    let items = st.alerts.read().clone();
    let quotes = st.quotes.read().clone();
    rsx! {
        div { class: "alerts-tab",
            div { class: "btn-row", style: "padding: 6px 12px",
                button { class: "mini primary", onclick: move |_| {
                    let key = st.selected.peek().clone();
                    let pxv = st.quotes.peek().iter().find(|q| q.key == key).map(|q| q.mid).unwrap_or(0.0);
                    st.alert_dialog.set(Some(if pxv > 0.0 { pxv } else { 1.0 }));
                }, "+ New alert (A)" }
                span { class: "muted small", "Alerts fire from the API every second: toast + sound here, Telegram if enabled." }
            }
            if items.is_empty() { p { class: "empty", "No price alerts. Right-click a chart or press A." } } else {
                table { class: "grid-tbl",
                    thead { tr { th { "Instrument" } th { "Condition" } th { class: "r", "Level" } th { class: "r", "Last" } th { "Status" } th { "Note" } th { "" } } }
                    tbody {
                        for a in items {
                            {
                                let last = quotes.iter().find(|q| q.key == a.key).map(|q| q.mid).unwrap_or(0.0);
                                let dec = quotes.iter().find(|q| q.key == a.key).map(|q| q.decimals).filter(|d| *d > 0).unwrap_or_else(|| decimals_for_key(&a.key));
                                let level_txt = if a.kind == "change_pct" { format!("{:+.2}%", a.level) } else { px(a.level, dec) };
                                let status = if a.armed { "armed".to_string() } else { format!("fired {}", a.fired_at.clone().unwrap_or_default().chars().take(16).collect::<String>()) };
                                let id = a.id.clone();
                                rsx! { tr {
                                    td { class: "sym", "{a.key}" } td { "{a.kind}" } td { class: "r num", "{level_txt}" } td { class: "r num", "{px(last, dec)}" }
                                    td { span { class: if a.armed { "tag mint" } else { "tag" }, "{status}" } if a.repeat { span { class: "tag lav xs", "repeat" } } }
                                    td { class: "muted", "{a.note}" }
                                    td { button { class: "mini danger", onclick: move |_| { let id = id.clone(); spawn(async move { let _ = delete_alert(id).await; if let Ok(a) = get_alerts().await { st.alerts.set(a); } }); }, "Delete" } }
                                } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NewsTab() -> Element {
    let mut st = app_state();
    let key = st.selected.read().clone();
    let mut cal = use_signal(|| None::<Calendar>);
    let mut article = use_signal(|| None::<(String, String)>);
    let mut pause_in = use_signal(String::new);
    let mut err = use_signal(String::new);
    use_future(move || async move {
        match get_calendar().await { Ok(c) => { pause_in.set(c.news_pause_minutes.to_string()); cal.set(Some(c)); } Err(e) => err.set(e.to_string()) }
    });
    // news per instrument (loaded when the selection changes)
    use_effect(move || {
        let k = st.selected.read().clone();
        let cached = st.news_cache.peek().as_ref().map(|c| c.0 == k).unwrap_or(false);
        if !cached {
            spawn(async move { match get_news(k.clone()).await { Ok(items) => st.news_cache.set(Some((k, items))), Err(e) => st.news_cache.set(Some((k, vec![NewsItem { headline: format!("No news available: {e}"), ..Default::default() }]))) } });
        }
    });
    let news = st.news_cache.read().clone().map(|c| c.1).unwrap_or_default();
    let cal_items = cal.read().clone().map(|c| c.items).unwrap_or_default();
    let cal_stale = cal.read().as_ref().map(|c| c.calendar_stale).unwrap_or(false);
    let art = article.read().clone();
    rsx! {
        div { class: "news-grid",
            div { class: "news-col",
                div { class: "panel-title", "Headlines" span { class: "muted", "{key}" } }
                if news.is_empty() { p { class: "empty", "Loading…" } }
                for n in news {
                    {
                        let (prov, aid, head) = (n.provider.clone(), n.article_id.clone(), n.headline.clone());
                        rsx! { div { class: "news-item", onclick: move |_| {
                            let (p, a, h) = (prov.clone(), aid.clone(), head.clone());
                            if a.is_empty() { return; }
                            spawn(async move { match get_article(p, a).await { Ok(t) => article.set(Some((h, t))), Err(e) => article.set(Some((h, format!("Could not load article: {e}")))) } });
                        },
                            span { class: "muted num small", "{n.time}" } span { class: "tag sky xs", "{n.provider}" } span { "{n.headline}" } } }
                    }
                }
                match art { Some((h, t)) => rsx! { div { class: "article", div { class: "panel-title", "{h}" button { class: "mini", style: "margin-left:auto", onclick: move |_| article.set(None), "Close" } } div { class: "article-body", "{t}" } } }, None => rsx! {} }
            }
            div { class: "news-col",
                div { class: "panel-title", "Economic calendar (this week)"
                    span { class: "muted small", style: "margin-left:auto", "Pause agents around high-impact news (min):" }
                    if cal_stale { span { class: "tag peach xs", title: "The economic calendar has not been refreshed for over 6 hours — the pause may miss events", "calendar stale" } }
                    input { class: "num short", value: "{pause_in}", oninput: move |e| pause_in.set(e.value()) }
                    button { class: "mini", onclick: move |_| {
                        // a blank or mistyped value must not silently switch the news pause off (0)
                        let m = match crate::calc::parse_minutes("News pause", &pause_in.peek(), 240) { Ok(v) => v, Err(e) => { st.toast.set(Some((false, format!("Not saved: {e}")))); return; } };
                        spawn(async move { match set_news_pause(m).await { Ok(v) => st.toast.set(Some((true, format!("News pause set to {v} min")))), Err(e) => st.toast.set(Some((false, format!("Failed: {e}")))) } }); }, "Save" }
                }
                if !err.read().is_empty() { p { class: "empty", "{err}" } }
                if cal_stale { div { class: "notice small", "The economic calendar is stale (not refreshed for over 6 hours). The news pause for agents works on this old list until it refreshes." } }
                table { class: "grid-tbl",
                    thead { tr { th { "Time (UTC)" } th { "Cur" } th { "Impact" } th { "Event" } th { class: "r", "Actual" } th { class: "r", "Forecast" } th { class: "r", "Previous" } } }
                    tbody {
                        for c in cal_items {
                            {
                                let cls = match c.impact.as_str() { "high" => "tag rose xs", "medium" => "tag peach xs", _ => "tag xs" };
                                let d: String = c.date.chars().take(16).collect::<String>().replace('T', " ");
                                rsx! { tr { td { class: "num", "{d}" } td { "{c.country}" } td { span { class: "{cls}", "{c.impact}" } } td { "{c.title}" } td { class: "r num", "{c.actual}" } td { class: "r num", "{c.forecast}" } td { class: "r num", "{c.previous}" } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn IndicatorMenu() -> Element {
    let mut st = app_state();
    let cfg = st.indicators.read().clone();
    let has_ema = |n: u32| cfg.ema.contains(&n);
    let has_sma = |n: u32| cfg.sma.contains(&n);
    let mut toggle_ema = move |n: u32| { let mut c = st.indicators.write(); if c.ema.contains(&n) { c.ema.retain(|x| *x != n); } else { c.ema.push(n); } };
    let mut toggle_sma = move |n: u32| { let mut c = st.indicators.write(); if c.sma.contains(&n) { c.sma.retain(|x| *x != n); } else { c.sma.push(n); } };
    rsx! {
        div { class: "ind-menu",
            div { class: "ind-group", span { class: "lbl", "Moving averages" }
                for n in [9u32, 20, 50, 200] {
                    label { class: "chk", input { r#type: "checkbox", checked: has_ema(n), oninput: move |_| toggle_ema(n) } span { "EMA {n}" } }
                }
                for n in [50u32, 200] {
                    label { class: "chk", input { r#type: "checkbox", checked: has_sma(n), oninput: move |_| toggle_sma(n) } span { "SMA {n}" } }
                }
            }
            div { class: "ind-group", span { class: "lbl", "Overlays" }
                label { class: "chk", input { r#type: "checkbox", checked: cfg.bb.is_some(), oninput: move |e| { st.indicators.write().bb = if e.checked() { Some(BbCfg { period: 20, k: 2.0 }) } else { None }; } } span { "Bollinger (20, 2)" } }
                label { class: "chk", input { r#type: "checkbox", checked: cfg.vwap, oninput: move |e| st.indicators.write().vwap = e.checked() } span { "VWAP" } }
                label { class: "chk", input { r#type: "checkbox", checked: cfg.volume, oninput: move |e| st.indicators.write().volume = e.checked() } span { "Volume" } }
            }
            div { class: "ind-group", span { class: "lbl", "Lower pane" }
                for (id, lbl) in [("none", "None"), ("rsi", "RSI 14"), ("macd", "MACD 12/26/9")] {
                    label { class: "chk", input { r#type: "radio", name: "subpane", checked: cfg.sub == id, oninput: move |_| st.indicators.write().sub = id.to_string() } span { "{lbl}" } }
                }
            }
        }
    }
}

#[component]
fn ChartContextMenu() -> Element {
    let mut st = app_state();
    let Some(m) = st.ctx_menu.read().clone() else { return rsx! {} };
    let key = st.selected.read().clone();
    let quote = st.quotes.read().iter().find(|q| q.key == key).cloned().unwrap_or_default();
    let dec = st.dec_for(&key);
    let mid = if quote.mid > 0.0 { quote.mid } else { quote.last };
    let price = m.price;
    let has_price = price > 0.0 && price.is_finite();
    let p_lab = if has_price { format!("{:.*}", dec, price) } else { "no price here".to_string() };
    let manual = *st.mode.read() == "manual";
    let can_trade = st.can_trade();
    // the order behind the line: cancelling from the chart is for manual orders, in Manual mode, with confirmation
    let line_order = st.snapshot.read().as_ref().and_then(|s| s.orders.iter().find(|o| o.order_id.to_string() == m.line_id).cloned());
    let can_cancel = manual && can_trade && line_order.as_ref().map(order_is_manual).unwrap_or(false);
    let mut prefill = move |side: &'static str, otype: &'static str| {
        st.ticket_prefill.set(Some(Prefill { side: side.into(), otype: otype.into(), price }));
        st.ctx_menu.set(None);
    };
    // order type that makes sense for the price relative to the market
    let below = mid > 0.0 && price < mid;
    let has_mid = mid > 0.0;
    let win_h = 720.0; let top = if m.y + 260.0 > win_h { (m.y - 260.0).max(0.0) } else { m.y };
    let line_kind = m.line_kind.clone();
    let line_title = m.line_title.clone();
    let line_id_txt = m.line_id.clone();
    let order_id: i64 = m.line_id.parse().unwrap_or(0);
    let hline_id = m.line_id.clone();
    let ctx_pane = m.pane;
    let (mx, my) = (m.x, top);
    rsx! {
        div { class: "ctx-bg", onclick: move |_| st.ctx_menu.set(None), oncontextmenu: move |e| { e.prevent_default(); st.ctx_menu.set(None); } }
        div { class: "ctx", style: "left: {mx}px; top: {my}px;",
            div { class: "ctx-title num", "{p_lab}" }
            if line_kind == "order" {
                if can_cancel {
                    div { class: "ctx-item danger", onclick: move |_| {
                        let id = order_id;
                        st.ctx_menu.set(None);
                        st.ask("Cancel order?", format!("{line_title} #{id}"), "Cancel order", ConfirmAction::CancelOrder(id));
                    }, "Cancel order {line_title} #{line_id_txt}" }
                } else {
                    div { class: "ctx-item muted", "Order {line_title} #{line_id_txt} — manage it in the Orders tab (Manual mode, manual orders only)" }
                }
            } else if line_kind == "hline" {
                div { class: "ctx-item", onclick: move |_| { js_call(ctx_pane, &format!("I.removeHLine('{}');", hline_id), true); st.ctx_menu.set(None); }, "Remove horizontal line" }
            } else {
                if !has_price {
                    div { class: "ctx-item muted", "No price at this point — right-click inside the price area" }
                } else if !can_trade {
                    div { class: "ctx-item muted", "Your role ({st.role()}) cannot place orders" }
                } else if !has_mid {
                    div { class: "ctx-item muted", "No market price for this instrument — cannot tell limit from stop" }
                } else if manual {
                    if below {
                        div { class: "ctx-item up", onclick: move |_| prefill("BUY", "LMT"), "Buy Limit @ {p_lab}" }
                        div { class: "ctx-item down", onclick: move |_| prefill("SELL", "STP"), "Sell Stop @ {p_lab}" }
                    } else {
                        div { class: "ctx-item down", onclick: move |_| prefill("SELL", "LMT"), "Sell Limit @ {p_lab}" }
                        div { class: "ctx-item up", onclick: move |_| prefill("BUY", "STP"), "Buy Stop @ {p_lab}" }
                    }
                } else {
                    div { class: "ctx-item muted", "Switch to Manual mode to trade from the chart" }
                }
                if has_price {
                    div { class: "ctx-sep" }
                    div { class: "ctx-item", onclick: move |_| { st.ctx_menu.set(None); st.alert_dialog.set(Some(price)); }, "Alert at {p_lab}" }
                    div { class: "ctx-item", onclick: move |_| { js_call(ctx_pane, &format!("I.addHLine({price});"), true); st.ctx_menu.set(None); }, "Add horizontal line here" }
                }
            }
        }
    }
}

// ---------------- order ticket ----------------
fn pip_size(market: &str, symbol: &str) -> f64 {
    match market { "FX" => if symbol.ends_with("JPY") { 0.01 } else { 0.0001 }, _ => 0.01 }
}

#[component]
pub fn OrderTicket() -> Element {
    let mut st = app_state();
    let key = st.selected.read().clone();
    let quote = st.quotes.read().iter().find(|q| q.key == key).cloned().unwrap_or_default();
    let dec = st.dec_for(&key);
    let symbol = if quote.symbol.is_empty() { key.split(':').nth(1).unwrap_or(&key).to_string() } else { quote.symbol.clone() };
    let market = if quote.market.is_empty() { key.split(':').next().unwrap_or("").to_string() } else { quote.market.clone() };
    let tradable = quote.tradable || quote.key.is_empty() && market != "IDX";
    let can_trade = st.can_trade();
    let unit = if quote.unit.is_empty() { "units".to_string() } else { quote.unit.clone() };
    let mut qty_mode = use_signal(|| "coins".to_string());   // crypto: "coins" | "usd"
    let presets: [f64; 4] = match market.as_str() {
        "STK" => [10.0, 50.0, 100.0, 500.0],
        "CRYPTO" => if *qty_mode.read() == "usd" { [100.0, 500.0, 1000.0, 5000.0] } else { [0.01, 0.05, 0.1, 0.5] },
        _ => [10000.0, 20000.0, 50000.0, 100000.0],
    };
    let step = match market.as_str() { "STK" => "1", "CRYPTO" => if *qty_mode.read() == "usd" { "10" } else { "0.001" }, _ => "1000" };
    let pip = pip_size(&market, &symbol);

    let mut side = use_signal(|| "BUY".to_string());
    let mut otype = use_signal(|| "MKT".to_string());
    let mut qty = use_signal(|| "20000".to_string());
    let mut price = use_signal(String::new);
    let mut tif = use_signal(|| "DAY".to_string());
    let mut trail = use_signal(String::new);
    let mut bracket = use_signal(|| false);
    let mut tp = use_signal(String::new);
    let mut sl = use_signal(String::new);
    let mut risk_usd = use_signal(|| "100".to_string());
    let mut confirming = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let tk_w = *st.ticket_w.read();
    let n_running_nodes = st.snapshot.read().as_ref().map(|s| s.running_nodes.len()).unwrap_or(0);
    let is_fx = market == "FX";
    let fx_lot_hint = {
        let q: f64 = qty.read().replace(',', "").parse().unwrap_or(0.0);
        if q > 0.0 && q < 1000.0 { format!("⚠ {q} units is too small — enter units of {}: 1,000 = 0.01 lot · 10,000 = 0.1 lot · 100,000 = 1 lot", &symbol[..3.min(symbol.len())]) }
        else { format!("= {:.2} lot (1 lot = 100,000 {})", q / 100000.0, &symbol[..3.min(symbol.len())]) }
    };

    use_effect(move || {
        let pf = st.ticket_prefill.read().clone();
        if let Some(p) = pf {
            side.set(p.side.clone());
            otype.set(p.otype.clone());
            if p.price > 0.0 { price.set(format!("{:.*}", dec, p.price)); }
            confirming.set(false);
            st.ticket_prefill.set(None);
        }
    });

    // another instrument: prices typed for the previous one (limit / stop / trail / SL / TP) no longer apply
    let mut last_key = use_signal(String::new);
    let mut last_market = use_signal(String::new);
    if *last_key.peek() != key {
        let first = last_key.peek().is_empty();
        last_key.set(key.clone());
        if !first {
            price.set(String::new()); trail.set(String::new()); tp.set(String::new()); sl.set(String::new());
        }
        confirming.set(false);
    }
    if *last_market.peek() != market {
        last_market.set(market.clone());
        qty.set(format!("{}", presets[1]));
        tif.set("DAY".into());
        if market == "CRYPTO" {
            bracket.set(false);                                   // no bracket orders for crypto at IBKR
            if matches!(otype.peek().as_str(), "STP" | "TRAIL") { otype.set("MKT".into()); }
        }
        confirming.set(false);
    }

    let qty_v: f64 = qty.read().replace(',', "").parse().unwrap_or(0.0);
    let price_v: Option<f64> = price.read().parse().ok().filter(|v: &f64| v.is_finite() && *v > 0.0);
    let sl_v: Option<f64> = sl.read().parse().ok().filter(|v: &f64| v.is_finite() && *v > 0.0);
    let tp_v: Option<f64> = tp.read().parse().ok().filter(|v: &f64| v.is_finite() && *v > 0.0);
    let is_buy = *side.read() == "BUY";
    let ref_px = if is_buy { quote.ask } else { quote.bid };
    let ref_px = if ref_px > 0.0 { ref_px } else { quote.last };
    let entry = match otype.read().as_str() { "LMT" | "STP" => price_v.unwrap_or(ref_px), _ => ref_px };
    let usd_mode = market == "CRYPTO" && *qty_mode.read() == "usd";
    let coins_from_usd = if usd_mode && ref_px > 0.0 { qty_v / ref_px } else { 0.0 };
    let needs_price = matches!(otype.read().as_str(), "LMT" | "STP");
    let is_trail = *otype.read() == "TRAIL";
    let is_crypto = market == "CRYPTO";
    let crypto_cash_buy = is_crypto && *otype.read() == "MKT" && *side.read() == "BUY";   // IBKR: Market crypto buys must be specified in USD
    let use_bracket = *bracket.read() && !is_trail && !is_crypto;
    let crypto_usd: f64 = if usd_mode { qty_v } else { qty_v * quote.mid };
    // USD figures: quote currency → USD from the FX quotes the Terminal already streams (None = rate not available → "—")
    let rates = crate::calc::rates_from_prices(st.quotes.read().iter().filter(|q| q.market == "FX").map(|q| (q.symbol.as_str(), if q.mid > 0.0 { q.mid } else { q.last })).collect::<Vec<_>>());
    let tm = crate::calc::ticket_math(&market, &symbol, qty_v, usd_mode, entry, if use_bracket { sl_v } else { None }, if use_bracket { tp_v } else { None }, &rates);
    let usd_per_price_unit = tm.usd_per_price_unit;
    let opt_money = |v: Option<f64>| v.map(|x| format!("{} USD", money(x))).unwrap_or_else(|| "— (no USD rate)".into());
    let buy_px = px(quote.ask.max(quote.last), dec);
    let sell_px = px(if quote.bid > 0.0 { quote.bid } else { quote.last }, dec);

    // ---- the order exactly as it would be sent, built from the signals at call time (button, Enter hotkey and validation use the same path) ----
    let build_req = move || -> Result<OrderRequest, String> {
        let key = st.selected.read().clone();
        let q = st.quotes.read().iter().find(|q| q.key == key).cloned().unwrap_or_default();
        let market = key.split(':').next().unwrap_or("").to_string();
        if !st.can_trade() { return Err(format!("Your role ({}) cannot place orders", st.role())); }
        if *st.mode.read() != "manual" { return Err("Switch to Manual mode to trade".into()); }
        if !(q.tradable || q.key.is_empty() && market != "IDX") { return Err(format!("{key} is view-only")); }
        let is_crypto = market == "CRYPTO";
        let ot = otype.read().clone();
        if is_crypto && matches!(ot.as_str(), "STP" | "TRAIL") { return Err("Crypto supports Market and Limit orders only".into()); }
        let sd = side.read().clone();
        let um = is_crypto && *qty_mode.read() == "usd";
        let qv: f64 = qty.read().replace(',', "").parse().unwrap_or(0.0);
        if !(qv.is_finite() && qv > 0.0) { return Err("Enter a positive quantity".into()); }
        let rp = { let p = if sd == "BUY" { q.ask } else { q.bid }; if p > 0.0 { p } else { q.last } };
        let units = if um { if rp > 0.0 { qv / rp } else { return Err("No price to convert the USD amount to coins".into()) } } else { qv };
        // prices are sent on the instrument's grid (its display decimals), like a dragged order line
        let dec = st.dec_for(&key);
        let grid = |v: f64| crate::calc::round_to_dec(v, dec);
        let lp: Option<f64> = if matches!(ot.as_str(), "LMT" | "STP") {
            match price.read().parse::<f64>() { Ok(v) if v.is_finite() && v > 0.0 => Some(grid(v)), _ => return Err("Enter a price for the Limit / Stop order".into()) }
        } else { None };
        let tr: Option<f64> = if ot == "TRAIL" {
            match trail.read().parse::<f64>() { Ok(v) if v.is_finite() && grid(v) > 0.0 => Some(grid(v)), _ => return Err("Enter the trail amount (at least one price step)".into()) }
        } else { None };
        let br = *bracket.read() && ot != "TRAIL" && !is_crypto;
        let pos = |s: &Signal<String>| s.read().parse::<f64>().ok().filter(|v| v.is_finite() && *v > 0.0).map(grid);
        let (tpv, slv) = if br { (pos(&tp), pos(&sl)) } else { (None, None) };
        if br && tpv.is_none() && slv.is_none() { return Err("Enter a take profit and/or a stop loss, or untick the bracket".into()); }
        let entry = lp.unwrap_or(rp);
        if entry > 0.0 {
            if sd == "BUY" {
                if tpv.map(|t| t <= entry).unwrap_or(false) { return Err("Take profit must be above the entry for a BUY".into()); }
                if slv.map(|s| s >= entry).unwrap_or(false) { return Err("Stop loss must be below the entry for a BUY".into()); }
            } else {
                if tpv.map(|t| t >= entry).unwrap_or(false) { return Err("Take profit must be below the entry for a SELL".into()); }
                if slv.map(|s| s <= entry).unwrap_or(false) { return Err("Stop loss must be above the entry for a SELL".into()); }
            }
        }
        // crypto: the API sets IOC for Market orders itself and accepts only DAY / GTC
        let tf = if is_crypto { "DAY".to_string() } else { tif.read().clone() };
        Ok(OrderRequest { key, side: sd, qty: units, cash_qty: if um { Some(qv) } else { None }, order_type: ot, price: lp, tif: tf,
                          take_profit: tpv, stop_loss: slv, trail_amount: tr })
    };
    let check = build_req();
    let valid = check.is_ok();
    let invalid_reason = check.as_ref().err().cloned().unwrap_or_default();

    // ---- risk-based sizing: qty = risk / (stop distance × USD per price unit); disabled without a USD rate ----
    let size_by_risk = move |_: Event<MouseData>| {
        let r: f64 = risk_usd.read().parse().unwrap_or(0.0);
        let slv: Option<f64> = sl.read().parse().ok();
        if let Some(slv) = slv {
            match crate::calc::size_by_risk(&market, r, (entry - slv).abs(), usd_per_price_unit) {
                Some(n) => { qty.set(format!("{}", n)); confirming.set(false); }
                None => st.toast.set(Some((false, "Cannot size: needs risk > 0, a stop distance and a USD rate for this pair".into()))),
            }
        }
    };
    // helper buttons to set SL/TP as a pip distance from the entry price
    let mut set_sl_pips = move |pips: f64| {
        let v = if is_buy { entry - pips * pip } else { entry + pips * pip };
        sl.set(format!("{:.*}", dec, v)); confirming.set(false);
    };
    let mut set_tp_pips = move |pips: f64| {
        let v = if is_buy { entry + pips * pip } else { entry - pips * pip };
        tp.set(format!("{:.*}", dec, v)); confirming.set(false);
    };

    let do_submit = move || {
        let req = match build_req() { Ok(r) => r, Err(e) => { confirming.set(false); st.toast.set(Some((false, e))); return; } };
        if !*confirming.read() { confirming.set(true); return; }
        confirming.set(false);
        busy.set(true);
        spawn(async move {
            match place_order(req.clone()).await {
                Ok(r) => {
                    let bad = matches!(r.status.as_str(), "Cancelled" | "Inactive" | "ApiCancelled");
                    let reason = if bad && !r.message.is_empty() { format!(" — {}", r.message) } else { String::new() };
                    let qty_txt = match req.cash_qty { Some(c) => format!("{} USD of", money(c)), None => qty_fmt(req.qty) };
                    st.toast.set(Some((!bad, format!("{} {} {} → {} {}{}{}", req.side, qty_txt, req.key, r.status,
                        if r.avg_fill_price > 0.0 { format!("@ {} ", px(r.avg_fill_price, st.dec_for(&req.key))) } else { String::new() },
                        if r.child_order_ids.is_empty() { String::new() } else { format!("(+{} bracket orders)", r.child_order_ids.len()) }, reason))));
                }
                Err(e) => st.toast.set(Some((false, format!("Order rejected: {e}")))),
            }
            busy.set(false);
            st.refresh_tick += 1;
        });
    };
    let mut do_submit2 = do_submit;
    let submit = move |_: Event<MouseData>| do_submit2();
    let mut last_req = use_signal(|| 0u32);
    use_effect(move || {
        let r = *st.submit_req.read();
        if r != *last_req.peek() {
            last_req.set(r);
            if *st.mode.peek() == "manual" && !*busy.peek() { let mut f = do_submit; f(); }
        }
    });
    let confirm_label = {
        let what = if usd_mode { format!("{} USD of {symbol}", qty_fmt(qty_v)) } else { format!("{} {symbol}", qty_fmt(qty_v)) };
        let at = match (otype.read().as_str(), price_v) { ("LMT", Some(p)) => format!(" LMT {}", px(p, dec)), ("STP", Some(p)) => format!(" STP {}", px(p, dec)), ("TRAIL", _) => " TRAIL".to_string(), _ => " MKT".to_string() };
        format!("Confirm {} {what}{at}", side.read())
    };

    rsx! {
        aside { class: "panel ticket", style: "width: {tk_w}px",
            div { class: "panel-title", "Order Ticket" span { class: "muted", "{symbol}" } }

            if n_running_nodes > 0 {
                div { class: "notice agents-running",
                    span { class: "dot live" }
                    " {n_running_nodes} node(s) trading — "
                    a { href: "#", onclick: move |e| { e.prevent_default(); st.bottom_tab.set(6); let h = (*st.bottom_h.peek()).max(BOTTOM_DEFAULT); st.bottom_h.set(h); }, "manage / stop" }
                }
            }
            if !can_trade {
                div { class: "notice", "Signed in as {st.role()} — this role cannot place orders." }
            }
            if !tradable && is_crypto {
                div { class: "notice", b { "Crypto: view-only" } " — not permitted on this IBKR account (Regulatory Restrictions)." }
            } else if !tradable {
                div { class: "notice", "{symbol} is a view-only instrument and cannot be traded. Select another instrument from the watchlist." }
            }

            div { class: "side-row",
                button { class: if is_buy { "side buy active" } else { "side buy" }, onclick: move |_| { side.set("BUY".into()); confirming.set(false); },
                    div { class: "side-lbl", "BUY" } div { class: "side-px num", "{buy_px}" } }
                button { class: if !is_buy { "side sell active" } else { "side sell" }, onclick: move |_| { side.set("SELL".into()); confirming.set(false); },
                    div { class: "side-lbl", "SELL" } div { class: "side-px num", "{sell_px}" } }
            }

            div { class: "fld-row",
                label { class: "fld", span { "Type" }
                    Dropdown { value: "{otype}", onchange: move |v| { otype.set(v); confirming.set(false); },
                        options: if is_crypto { dd_opts([("MKT", "Market"), ("LMT", "Limit")]) } else { dd_opts([("MKT", "Market"), ("LMT", "Limit"), ("STP", "Stop"), ("TRAIL", "Trailing stop")]) } }
                }
                label { class: "fld", span { "TIF" }
                    if crypto_cash_buy {
                        Dropdown { value: "IOC", disabled: true, onchange: move |_| {}, options: dd_opts([("IOC", "IOC (set by IBKR)")]) }
                    } else if is_crypto {
                        Dropdown { value: "DAY", disabled: true, onchange: move |_| {}, options: dd_opts([("DAY", "DAY")]) }
                    } else {
                        Dropdown { value: "{tif}", onchange: move |v| { tif.set(v); confirming.set(false); },
                            options: dd_opts([("DAY", "DAY"), ("GTC", "GTC")]) }
                    }
                }
            }
            if needs_price {
                label { class: "fld", span { "Price" }
                    input { r#type: "number", step: "any", placeholder: "{px(quote.mid, dec)}", value: "{price}",
                        oninput: move |e| { price.set(e.value()); confirming.set(false); } }
                }
            }
            if is_trail {
                label { class: "fld", span { "Trail amount (price distance)" }
                    input { r#type: "number", step: "any", placeholder: "e.g. {pip * 20.0:.5}", value: "{trail}",
                        oninput: move |e| { trail.set(e.value()); confirming.set(false); } }
                    span { class: "muted xs", "Stop follows the price at this distance" }
                }
            }
            label { class: "fld",
                span { if usd_mode { "Amount (USD)" } else { "Quantity ({unit})" }
                    if is_crypto {
                        span { class: "qty-mode",
                            button { class: if !usd_mode { "qm active" } else { "qm" }, onclick: move |e| { e.prevent_default(); if *qty_mode.peek() != "coins" { qty_mode.set("coins".into()); qty.set("0.01".into()); confirming.set(false); } }, "Coins" }
                            button { class: if usd_mode { "qm active" } else { "qm" }, onclick: move |e| { e.prevent_default(); if *qty_mode.peek() != "usd" { qty_mode.set("usd".into()); qty.set("500".into()); confirming.set(false); } }, "USD" }
                        }
                    }
                }
                input { r#type: "number", step: "{step}", min: "0", value: "{qty}",
                    oninput: move |e| { qty.set(e.value()); confirming.set(false); } }
                if is_fx { span { class: "muted xs", "{fx_lot_hint}" } }
                if usd_mode { span { class: "muted xs", "≈ {coins_from_usd:.6} {symbol} at the current price" if crypto_cash_buy { " · sent to IBKR as cash quantity (IOC)" } else { " · sent as coins" } } }
                else if crypto_cash_buy { span { class: "muted xs", "IBKR fills market buys as a USD amount: ≈ {money(crypto_usd)} USD at the current price (coins received may differ slightly)" } }
                else if is_crypto { span { class: "muted xs", "= {money(crypto_usd)} USD · min 0.0001 BTC-equivalent" } }
            }
            div { class: "presets",
                for p in presets {
                    button { class: "preset", onclick: move |_| { qty.set(qty_fmt(p).replace(',', "")); confirming.set(false); }, "{qty_fmt(p)}" }
                }
            }

            // ---- bracket ----
            if is_crypto {
                p { class: "muted xs", "IBKR crypto (Paxos): Market and Limit orders only, no Stop / Trailing / bracket — use price alerts to manage exits." }
            }
            if !is_trail && !is_crypto {
                div { class: "bracket",
                    label { class: "chk",
                        input { r#type: "checkbox", checked: *bracket.read(), oninput: move |e| { bracket.set(e.checked()); confirming.set(false); } }
                        span { "Attach take profit / stop loss (bracket)" }
                    }
                    if use_bracket {
                        div { class: "fld-row",
                            label { class: "fld", span { class: "text-down", "Stop loss" }
                                input { r#type: "number", step: "any", value: "{sl}", placeholder: "price", oninput: move |e| { sl.set(e.value()); confirming.set(false); } }
                            }
                            label { class: "fld", span { class: "text-up", "Take profit" }
                                input { r#type: "number", step: "any", value: "{tp}", placeholder: "price", oninput: move |e| { tp.set(e.value()); confirming.set(false); } }
                            }
                        }
                        div { class: "presets",
                            span { class: "muted xs", "SL" }
                            for p in [10.0, 20.0, 50.0] { button { class: "preset", onclick: move |_| set_sl_pips(p), "{p:.0}p" } }
                            span { class: "muted xs", "TP" }
                            for p in [20.0, 40.0, 100.0] { button { class: "preset", onclick: move |_| set_tp_pips(p), "{p:.0}p" } }
                        }
                        div { class: "fld-row risk",
                            label { class: "fld", span { "Risk (USD)" }
                                input { r#type: "number", step: "10", value: "{risk_usd}", oninput: move |e| risk_usd.set(e.value()) }
                            }
                            button { class: "mini accent sizebtn", disabled: sl_v.is_none() || usd_per_price_unit.is_none(),
                                title: if usd_per_price_unit.is_none() { "No USD rate for this pair's quote currency — add the USD pair to the watchlist" } else { "" },
                                onclick: size_by_risk, "Size by risk" }
                        }
                        div { class: "summary",
                            div { span { class: "muted", "Risk at SL" } span { class: "num text-down", "{opt_money(tm.risk_at_sl_usd)}" } }
                            div { span { class: "muted", "Reward at TP" } span { class: "num text-up", "{opt_money(tm.reward_at_tp_usd)}" } }
                            div { span { class: "muted", "R : R" } span { class: "num", match tm.rr { Some(rr) => format!("1 : {rr:.2}"), None => "—".to_string() } } }
                        }
                    }
                }
            }

            div { class: "summary",
                div { span { class: "muted", "Est. notional" } span { class: "num", "{opt_money(tm.notional_usd)}" } }
                div { span { class: "muted", "Limits" } span { class: "muted num", "see Risk tab" } }
            }

            button {
                class: if !valid || *busy.read() { "submit disabled" } else if *confirming.read() { "submit confirm" } else if is_buy { "submit buy" } else { "submit sell" },
                disabled: !valid || *busy.read(),
                onclick: submit,
                if *busy.read() { "Sending..." }
                else if *confirming.read() { "{confirm_label}" }
                else { "{side} {symbol}" }
            }
            if *confirming.read() {
                button { class: "cancel-confirm", onclick: move |_| confirming.set(false), "Cancel" }
            }
            if !valid && can_trade && tradable { p { class: "muted xs", "{invalid_reason}" } }
            p { class: "muted small", "Orders pass API risk checks before reaching IBKR" }
        }
    }
}

// ---------------- bottom panel ----------------
#[component]
pub fn BottomPanel() -> Element {
    let mut st = app_state();
    let mut tab = st.bottom_tab;
    let mut confirm_close_all = use_signal(|| false);
    let snap = st.snapshot.read().clone().unwrap_or_default();
    let n_pos = snap.positions.len();
    let n_ord = snap.orders.len();
    let n_fill = snap.fills.len();
    let n_alert = st.alerts.read().iter().filter(|a| a.armed).count();
    let manual = *st.mode.read() == "manual";

    let h = *st.bottom_h.read();
    let collapsed = h <= BOTTOM_MIN + 1.0;
    rsx! {
        section { class: "panel bottom",
            div { class: "resize-handle", title: "Drag to resize · double-click to toggle",
                onmousedown: move |e| {
                    e.prevent_default();
                    let y0 = e.client_coordinates().y;
                    st.bottom_drag.set(Some((y0, *st.bottom_h.peek())));
                    spawn(async move {   // start from the height actually shown (the CSS caps it on short windows)
                        let r = document::eval("return document.querySelector('.panel.bottom')?.getBoundingClientRect().height || 0;").await;
                        if let Some(h) = r.ok().and_then(|v| v.as_f64()).filter(|h| *h > 0.0) {
                            let still = matches!(*st.bottom_drag.peek(), Some((y, _)) if y == y0);
                            if still { st.bottom_drag.set(Some((y0, h))); }
                        }
                    });
                },
                ondoubleclick: move |_| { let cur = *st.bottom_h.peek(); st.bottom_h.set(if cur < BOTTOM_BIG - 10.0 { BOTTOM_BIG } else { BOTTOM_DEFAULT }); },
            }
            div { class: "tabs", onclick: move |_| { if *st.bottom_h.peek() <= BOTTOM_MIN + 1.0 { st.bottom_h.set(BOTTOM_DEFAULT); } },
                button { class: if *tab.read() == 0 { "tab active" } else { "tab" }, onclick: move |_| tab.set(0), "Positions " span { class: "cnt", "{n_pos}" } }
                button { class: if *tab.read() == 1 { "tab active" } else { "tab" }, onclick: move |_| tab.set(1), "Orders " span { class: "cnt", "{n_ord}" } }
                button { class: if *tab.read() == 2 { "tab active" } else { "tab" }, onclick: move |_| tab.set(2), "Fills " span { class: "cnt", "{n_fill}" } }
                button { class: if *tab.read() == 3 { "tab active" } else { "tab" }, onclick: move |_| tab.set(3), "Risk" }
                button { class: if *tab.read() == 4 { "tab active" } else { "tab" }, onclick: move |_| tab.set(4), "Audit" }
                button { class: if *tab.read() == 5 { "tab active" } else { "tab" }, onclick: move |_| tab.set(5), "Performance" }
                button { class: if *tab.read() == 6 || *tab.read() == 10 { "tab active" } else { "tab" }, title: "Automated strategies (Nautilus nodes)", onclick: move |_| tab.set(6), "Agents" }
                button { class: if *tab.read() == 7 { "tab active" } else { "tab" }, onclick: move |_| tab.set(7), "Alerts " span { class: "cnt", "{n_alert}" } }
                button { class: if *tab.read() == 8 { "tab active" } else { "tab" }, onclick: move |_| tab.set(8), "News" }
                button { class: if *tab.read() == 9 { "tab active" } else { "tab" }, onclick: move |_| tab.set(9), "Backtests" }
                button { class: if *tab.read() == 11 { "tab active" } else { "tab" }, onclick: move |_| tab.set(11), "Strategies" }
                div { class: "tabs-right",
                    button { class: "tool", title: "Collapse", onclick: move |_| st.bottom_h.set(BOTTOM_MIN), "⤓" }
                    button { class: "tool", title: "Default size", onclick: move |_| st.bottom_h.set(BOTTOM_DEFAULT), "▭" }
                    button { class: "tool", title: "Expand", onclick: move |_| st.bottom_h.set(BOTTOM_BIG), "⤒" }
                    if (n_pos > 0 || n_ord > 0) && st.can_trade() {
                        if *confirm_close_all.read() {
                            span { class: "muted small", "Cancel all orders and close every position?" }
                            button { class: "mini danger", onclick: move |_| {
                                confirm_close_all.set(false);
                                spawn(async move {
                                    match close_all_positions().await {
                                        Ok(list) => {
                                            // rows the API could not close (no price, contract error) are reported, never hidden
                                            let skipped: Vec<String> = list.iter().filter(|c| c.status.starts_with("skipped")).map(|c| format!("{} — {}", c.key, c.status)).collect();
                                            let sent = list.len() - skipped.len();
                                            if skipped.is_empty() { st.toast.set(Some((true, format!("Close all: {sent} closing order(s) sent")))); }
                                            else {
                                                for x in &skipped { st.notify("error", format!("Close all: NOT closed {x}")); }
                                                st.toast.set(Some((false, format!("Close all: {sent} order(s) sent, {} NOT closed: {}", skipped.len(), skipped.join(" · ")))));
                                            }
                                        }
                                        Err(e) => st.toast.set(Some((false, format!("Close all failed: {e}")))),
                                    }
                                    st.refresh_tick += 1;
                                });
                            }, "Yes, close all" }
                            button { class: "mini", onclick: move |_| confirm_close_all.set(false), "No" }
                        } else {
                            button { class: "mini danger", onclick: move |_| confirm_close_all.set(true), "Close All" }
                        }
                    }
                }
            }
            div { class: if collapsed { "tab-body hidden-pane" } else { "tab-body" },
                match *tab.read() {
                    0 => rsx! { PositionsTable { positions: snap.positions.clone(), manual, recon: snap.reconcile.clone() } },
                    1 => rsx! { OrdersTable { orders: snap.orders.clone() } },
                    2 => rsx! { FillsTable { fills: snap.fills.clone() } },
                    3 => rsx! { RiskPanel {} },
                    4 => rsx! { AuditPanel {} },
                    5 => rsx! { PerformancePanel {} },
                    6 | 10 => rsx! { NodesTab {} },
                    7 => rsx! { AlertsTab {} },
                    9 => rsx! { BacktestsTab {} },
                    11 => rsx! { StrategiesTab {} },
                    _ => rsx! { NewsTab {} },
                }
            }
        }
    }
}

#[component]
fn PositionsTable(positions: Vec<Position>, manual: bool, recon: Reconcile) -> Element {
    let mut st = app_state();
    let can_trade = st.can_trade();
    let running: Vec<String> = st.snapshot.read().as_ref().map(|s| s.running_nodes.clone()).unwrap_or_default();
    // USD totals only include rows the API could convert to USD
    let (tot_u, skip_u) = crate::calc::usd_total(positions.iter().filter(|p| !p.unpriced).map(|p| (p.unrealized_pnl, p.usd)));
    let (tot_mv, _) = crate::calc::usd_total(positions.iter().filter(|p| !p.unpriced).map(|p| (p.market_value, p.usd)));
    let n_unpriced = positions.iter().filter(|p| p.unpriced).count();
    let tot_note = match (skip_u, n_unpriced) { (0, 0) => String::new(), (a, b) => format!("excludes {} row(s) in another currency{}", a, if b > 0 { format!(" and {b} unpriced cash balance(s)") } else { String::new() }) };
    rsx! {
        ReconStrip { recon }
        if positions.is_empty() { p { class: "empty", "No open positions" } } else {
            table { class: "grid-tbl",
                thead { tr { th { "Symbol" } th { "Source" } th { class: "r", "Qty" } th { class: "r", "Avg cost" } th { class: "r", "Mkt price" } th { class: "r", "Mkt value" } th { class: "r", "Unrealized" } th { class: "r", "Realized" } th { "" } } }
                tbody {
                    for p in positions {
                        {
                            let key = p.key.clone();
                            let key2 = p.key.clone();
                            let dec = if p.decimals > 0 { p.decimals } else { st.dec_for(&p.key) };
                            // values are USD when p.usd; otherwise the API could not convert them → shown with their currency code
                            let ccy_tip = if p.usd { String::new() } else { format!("In {} — no USD rate available; left out of USD totals", p.currency) };
                            let running_owners: Vec<String> = std::iter::once(p.source.clone()).chain(p.sources.iter().map(|x| x.source.clone()))
                                .filter_map(|s| s.strip_prefix("node:").map(String::from)).filter(|n| running.contains(n)).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
                            let src = if p.source.is_empty() { "manual".to_string() } else { p.source.clone() };
                            let breakdown = p.sources.iter().map(|x| format!("{} {}", source_label(&x.source), qty_fmt(x.qty))).collect::<Vec<_>>().join(" · ");
                            let show_dot = matches!(p.recon.as_str(), "ok" | "mismatch" | "orphan" | "stale");
                            let dot_cls = format!("recon-dot {}", p.recon);
                            let dot_tip = p.recon_note.clone();
                            let node_owned = p.source.starts_with("node:") || p.source == "nautilus" || p.source.starts_with("agent:");
                            let side_txt = if p.position > 0.0 { "SELL" } else { "BUY" };
                            let close_txt = format!("{side_txt} {} {} at market (position {}){}", qty_fmt(p.position.abs()), p.symbol, qty_fmt(p.position),
                                if !running_owners.is_empty() { format!("\nNode {} still runs and holds this position — it may re-open or double-close it. Stopping the node first is safer (it closes its own positions).", running_owners.join(", ")) }
                                else if node_owned { format!("\nThis position belongs to {}.", source_label(&p.source)) } else { String::new() });
                            let force = !running_owners.is_empty();
                            let close_lbl = if force { "Close anyway" } else { "Close at market" };
                            let unpriced = p.unpriced;
                            let cash_txt = format!("cash {} {}", qty_fmt(p.cash_balance), p.cash_currency);
                            let rev_txt = format!("{side_txt} {} {} at market (2 × the position {}) — flips the position to the other side", qty_fmt(p.position.abs() * 2.0), p.symbol, qty_fmt(p.position));
                            rsx! {
                                tr {
                                    td { class: "sym", span { class: "{market_tag(&p.market)} xs", "{market_short(&p.market)}" } "{p.symbol}"
                                        if let Some(w) = p.cash_warn.clone().filter(|w| !w.trim().is_empty()) { span { class: "tag peach xs", style: "margin-left:6px", title: "{w}", "CASH?" } }
                                    }
                                    td { class: "src-cell",
                                        SourceTag { source: src, strategy: p.strategy.clone(), title: if p.sources.len() > 1 { breakdown } else { String::new() } }
                                        if show_dot { span { class: "{dot_cls}", title: "{dot_tip}" } }
                                    }
                                    if unpriced {
                                        td { class: "r num muted", title: "No price for this pair — the position cannot be sized (Close All reports it as skipped)", "{cash_txt}" }
                                    } else {
                                        td { class: "r num {pnl_class(p.position)}", "{qty_fmt(p.position)}" }
                                    }
                                    td { class: "r num", "{px(p.avg_cost, dec)}" }
                                    td { class: "r num", "{px(p.market_price, dec)}" }
                                    td { class: "r num", title: "{ccy_tip}", if unpriced { "—" } else { "{money_in(p.market_value, p.usd, &p.currency)}" } }
                                    td { class: "r num {pnl_class(p.unrealized_pnl)}", title: "{ccy_tip}", if unpriced { "—" } else { "{money_in(p.unrealized_pnl, p.usd, &p.currency)}" } }
                                    td { class: "r num {pnl_class(p.realized_pnl)}", title: "{ccy_tip}", if unpriced { "—" } else { "{money_in(p.realized_pnl, p.usd, &p.currency)}" } }
                                    td { class: "r actions",
                                        if manual && can_trade && !node_owned && !unpriced && p.position != 0.0 {
                                            button { class: "mini", title: "Flip to the opposite side with the same size", onclick: move |_| {
                                                st.ask("Reverse position?", format!("{rev_txt}"), "Reverse", ConfirmAction::Reverse(key2.clone()));
                                            }, "Reverse" }
                                        }
                                        if can_trade && !unpriced {
                                            button { class: "mini danger", onclick: move |_| {
                                                st.ask(if force { "Close a node's position?" } else { "Close position?" }, format!("{close_txt}"), close_lbl, ConfirmAction::ClosePos(key.clone(), force));
                                            }, "Close" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                tfoot { tr { class: "total-row",
                    td { colspan: "5", class: "muted small", "Total (USD) {tot_note}" }
                    td { class: "r num", "{money(tot_mv)}" }
                    td { class: "r num {pnl_class(tot_u)}", "{money(tot_u)}" }
                    td {} td {}
                } }
            }
        }
    }
}

#[component]
fn ReconStrip(recon: Reconcile) -> Element {
    if !recon_visible(&recon) { return rsx! {}; }
    let bad = recon_issue_rows(&recon);
    let checked = recon.ts.replace('T', " ").chars().skip(11).take(8).collect::<String>();
    let (cls, msg) = match recon.status.as_str() {
        "mismatch" => ("recon-strip bad", format!("{} instrument(s) don't match IBKR", recon.issues)),
        "warn" => {
            let n_drift = recon.rows.iter().filter(|r| r.drift).count();
            let n_stale = recon.rows.iter().filter(|r| r.status == "stale").count();
            let mut parts = vec![];
            if n_stale > 0 { parts.push(format!("{n_stale} node heartbeat(s) stale, using fill history")); }
            if n_drift > 0 { parts.push(format!("{n_drift} instrument(s): manual/agent history holds a position but IBKR is flat")); }
            let n_warn = recon.rows.iter().filter(|r| r.status == "warn").count();
            if n_warn > 0 { parts.push(format!("{n_warn} instrument(s) need a look (see notes)")); }
            ("recon-strip warn", if parts.is_empty() { "Check reconciliation".to_string() } else { parts.join(" · ") })
        }
        _ => ("recon-strip ok", format!("Node positions match IBKR ({} instrument(s))", recon.node_keys)),
    };
    rsx! {
        div { class: "{cls}",
            span { class: "recon-title", "Reconciliation" }
            span { "{msg}" }
            span { class: "muted small", "checked {checked} UTC" }
            for (cls, tip, text) in bad {
                span { class: "{cls}", title: "{tip}", "{text}" }
            }
        }
    }
}

/// Rows to list under the reconciliation strip: (css class, tooltip, text). "warn" rows (also on a manual-only book) are amber.
pub fn recon_issue_rows(recon: &Reconcile) -> Vec<(String, String, String)> {
    recon.rows.iter().filter(|r| r.drift || matches!(r.status.as_str(), "mismatch" | "orphan" | "stale" | "warn"))
        .map(|r| {
            let cls = match r.status.as_str() { "mismatch" | "orphan" => r.status.as_str(), "stale" => "stale", _ => "warn" };
            (format!("recon-item {cls}"), format!("expected {} · IBKR {}", qty_fmt(r.expected), qty_fmt(r.ib_qty)),
             format!("{}: {}", r.key.split(':').last().unwrap_or_default(), r.note))
        }).collect()
}

/// The strip shows whenever nodes are involved or any row needs attention — not only for node books
pub fn recon_visible(recon: &Reconcile) -> bool {
    recon.node_keys > 0 || recon.issues > 0 || matches!(recon.status.as_str(), "mismatch" | "warn") || !recon_issue_rows(recon).is_empty()
}

#[component]
fn OrdersTable(orders: Vec<Order>) -> Element {
    rsx! {
        if orders.is_empty() { p { class: "empty", "No working orders" } } else {
            table { class: "grid-tbl",
                thead { tr { th { "ID" } th { "Symbol" } th { "Source" } th { "Side" } th { "Type" } th { class: "r", "Qty" } th { class: "r", "Price" } th { "TIF" } th { "Status" } th { class: "r", "Filled" } th { "" } } }
                tbody {
                    for o in orders { OrderRow { order: o } }
                }
            }
        }
    }
}

#[component]
fn OrderRow(order: Order) -> Element {
    let mut st = app_state();
    let mut editing = use_signal(|| false);
    let mut e_qty = use_signal(String::new);
    let mut e_px = use_signal(String::new);
    let id = order.order_id;
    let price_v = finite_opt(order.limit_price.or(order.stop_price));
    let dec = st.dec_for(&order.key);
    // TRAIL: stop_price is the trail distance, not a price level
    let pr = match price_v {
        Some(v) if order.order_type == "TRAIL" => format!("trail {v}"),
        Some(v) => px(v, dec),
        None if order.order_type == "MKT" => "MKT".into(),
        None => format!("{} (price n/a)", order.order_type),      // e.g. STP LMT: the API does not report its prices
    };
    let can_trade = st.can_trade();
    let manual_mode = *st.mode.read() == "manual";
    let is_manual = order_is_manual(&order);
    // same rule as the chart: edits only for manual orders, in Manual mode (the API refuses every edit in Agent mode)
    let editable = can_trade && manual_mode && is_manual && (price_v.is_some() || order.order_type == "MKT");
    let desc = format!("{} {} {} {} #{id}", order.side, qty_fmt(order.qty), order.symbol, order.order_type);
    let cancel_body = if is_manual { desc.clone() } else {
        format!("{desc}\nThis order belongs to {} — if it is a node's take-profit / stop-loss, the node's position stays open WITHOUT that protection.", source_label(&order.source))
    };
    let key_o = order.key.clone();
    let otype_o = order.order_type.clone();
    let kind = if order.parent_id.is_some() {
        if order.order_type == "LMT" { "TP".to_string() } else if order.order_type == "STP" || order.order_type == "STP LMT" { "SL".to_string() } else { order.order_type.clone() }
    } else { order.order_type.clone() };
    let src = if order.source.is_empty() { legacy_source(&order.order_ref) } else { order.source.clone() };

    let start_edit = move |_: Event<MouseData>| {
        e_qty.set(qty_fmt(order.qty).replace(',', ""));
        e_px.set(price_v.map(|v| if order.order_type == "TRAIL" { format!("{v}") } else { format!("{:.*}", dec, v) }).unwrap_or_default());
        editing.set(true);
    };
    let save = move |_: Event<MouseData>| {
        let q: Option<f64> = e_qty.read().replace(',', "").parse().ok().filter(|v: &f64| v.is_finite() && *v > 0.0);
        // prices go out on the instrument's grid (TRAIL amounts are distances and stay as typed)
        let p: Option<f64> = e_px.read().parse().ok().filter(|v: &f64| v.is_finite() && *v > 0.0)
            .map(|v| if otype_o == "TRAIL" { v } else { crate::calc::round_to_dec(v, st.dec_for(&key_o)) });
        if q.is_none() && p.is_none() { st.toast.set(Some((false, "Enter a positive quantity or price".into()))); return; }
        editing.set(false);
        let what = [q.map(|x| format!("qty {}", qty_fmt(x))), p.map(|x| format!("price {x}"))].into_iter().flatten().collect::<Vec<_>>().join(", ");
        st.ask("Modify order?", format!("{desc}\nnew {what}"), "Modify order", ConfirmAction::ModifyOrder { id, qty: q, price: p });
    };

    rsx! {
        tr { class: if order.parent_id.is_some() { "child" } else { "" },
            td { class: "num muted", "{id}" }
            td { class: "sym",
                if order.parent_id.is_some() { span { class: "muted", "└ " } }
                "{order.symbol}"
            }
            td { class: "src-cell", SourceTag { source: src, strategy: order.strategy.clone() } }
            td { class: if order.side == "BUY" { "text-up" } else { "text-down" }, "{order.side}" }
            td { span { class: "status", "{kind}" } }
            td { class: "r num",
                if *editing.read() { input { class: "inline", r#type: "number", value: "{e_qty}", oninput: move |e| e_qty.set(e.value()) } }
                else { "{qty_fmt(order.qty)}" }
            }
            td { class: "r num",
                if *editing.read() && price_v.is_some() { input { class: "inline", r#type: "number", step: "any", value: "{e_px}", oninput: move |e| e_px.set(e.value()) } }
                else { "{pr}" }
            }
            td { class: "muted", "{order.tif}" }
            td { span { class: "status", "{order.status}" } }
            td { class: "r num", "{qty_fmt(order.filled)} / {qty_fmt(order.qty)}" }
            td { class: "r actions",
                if *editing.read() {
                    button { class: "mini accent", onclick: save, "Save" }
                    button { class: "mini", onclick: move |_| editing.set(false), "Cancel" }
                } else {
                    if editable { button { class: "mini", onclick: start_edit, "Edit" } }
                    if can_trade {
                        button { class: "mini danger", onclick: move |_| {
                            st.ask(if is_manual { "Cancel order?" } else { "Cancel a node's order?" }, cancel_body.clone(),
                                   if is_manual { "Cancel order" } else { "Cancel anyway" }, ConfirmAction::CancelOrder(id));
                        }, "Cancel" }
                    }
                }
            }
        }
    }
}

#[component]
fn FillsTable(fills: Vec<Fill>) -> Element {
    let st = app_state();
    rsx! {
        if fills.is_empty() { p { class: "empty", "No fills this session" } } else {
            table { class: "grid-tbl",
                thead { tr { th { "Time (UTC)" } th { "Symbol" } th { "Source" } th { "Side" } th { class: "r", "Qty" } th { class: "r", "Price" } th { class: "r", "Comm." } th { class: "r", "Realized" } } }
                tbody {
                    for f in fills {
                        tr {
                            td { class: "num muted", "{short_time(&f.time)}" }
                            td { class: "sym", "{f.symbol}" }
                            td { class: "src-cell", SourceTag { source: if f.source.is_empty() { "manual".to_string() } else { f.source.clone() }, strategy: f.strategy.clone() } }
                            td { class: if f.side == "BOT" { "text-up" } else { "text-down" }, if f.side == "BOT" { "BUY" } else { "SELL" } }
                            td { class: "r num", "{qty_fmt(f.qty)}" }
                            td { class: "r num", "{finite_opt(f.price).map(|v| px(v, st.dec_for(&f.key))).unwrap_or_else(|| \"—\".into())}" }
                            td { class: "r num muted", "{finite_opt(f.commission).map(money).unwrap_or_default()}" }
                            // IB sends 1.797e308 ("unset") as realized P&L on opening fills → blank, not a number
                            td { class: "r num {pnl_class(finite_opt(f.realized_pnl).unwrap_or(0.0))}", "{finite_opt(f.realized_pnl).map(money).unwrap_or_default()}" }
                        }
                    }
                }
            }
        }
    }
}

// ---------------- toast ----------------
#[component]
pub fn Toast() -> Element {
    let mut st = app_state();
    let t = st.toast.read().clone();
    match t {
        Some((ok, msg)) => rsx! {
            div { class: if ok { "toast ok" } else { "toast bad" },
                span { "{msg}" }
                button { onclick: move |_| st.toast.set(None), "✕" }
            }
        },
        None => rsx! {},
    }
}

// ---------------- market overview ----------------
#[component]
pub fn MarketOverview() -> Element {
    let mut st = app_state();
    let items = st.market.read().clone();
    rsx! {
        section { class: "market",
            div { class: "market-head",
                span { class: "panel-title-inline", "Market Overview" }
                span { class: "muted small", "click a market to open its heatmap · delayed data" }
            }
            if items.is_empty() {
                p { class: "empty small", "Loading market overview..." }
            } else {
                div { class: "market-row",
                    for it in items {
                        {
                            let last = if it.last <= 0.0 { "—".to_string() } else { format!("{:.*}", it.decimals, it.last) };
                            let dir = if it.prev_close <= 0.0 { "flat" } else if it.change_pct > 0.0 { "up" } else if it.change_pct < 0.0 { "down" } else { "flat" };
                            let gid = it.id.clone();
                            rsx! {
                                button { class: "mcard {dir}", onclick: move |_| st.heat_group.set(Some(gid.clone())),
                                    div { class: "mcard-top",
                                        span { class: "mcard-name", "{it.name}" }
                                        span { class: "mcard-sym muted", "{it.symbol}" }
                                    }
                                    div { class: "mcard-px num", "{last}" }
                                    div { class: "mcard-foot",
                                        span { class: "mcard-chg num", "{chg_pct(it.change_pct, it.prev_close)}" }
                                        span { class: "muted xs", "{it.members} symbols ›" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------- heatmap modal ----------------
fn heat_color(chg: f64) -> String {
    // -3% dark red … 0 gray … +3% dark green
    let t = (chg / 3.0).clamp(-1.0, 1.0);
    let (h, sat, light) = if t >= 0.0 { (152.0, 55.0 + 25.0 * t, 22.0 + 14.0 * t) } else { (352.0, 55.0 + 25.0 * (-t), 22.0 + 14.0 * (-t)) };
    let sat = if t.abs() < 0.05 { 8.0 } else { sat };
    format!("hsl({h} {sat}% {light}%)")
}

fn tile_span(weight: f64, max_w: f64) -> (u8, u8) {
    let r = if max_w > 0.0 { weight / max_w } else { 0.0 };
    if r >= 0.6 { (4, 3) } else if r >= 0.35 { (3, 2) } else if r >= 0.18 { (2, 2) } else if r >= 0.09 { (2, 1) } else { (1, 1) }
}

#[component]
pub fn HeatmapModal() -> Element {
    let mut st = app_state();
    let mut bg_down = use_signal(|| false);
    let group = st.heat_group.read().clone();
    let mut data = use_signal(|| None::<Heatmap>);
    let mut loading = use_signal(|| false);
    let mut err = use_signal(|| None::<String>);

    use_effect(move || {
        let g = st.heat_group.read().clone();
        data.set(None);
        err.set(None);
        if let Some(g) = g {
            spawn(async move {
                loading.set(true);
                match get_heatmap(g).await {
                    Ok(h) => data.set(Some(h)),
                    Err(e) => err.set(Some(e.to_string())),
                }
                loading.set(false);
            });
        }
    });

    let Some(gid) = group else { return rsx! {} };
    let d = data.read().clone();
    let title = d.as_ref().map(|h| format!("{} heatmap", h.name)).unwrap_or(format!("{gid} heatmap"));
    // cached API responses carry no breadth → count it from the tiles
    let breadth: Option<Breadth> = d.as_ref().map(|h| h.breadth.clone().unwrap_or_else(|| { let (up, down, total) = crate::calc::breadth_of(h.items.iter().map(|i| i.change_pct)); Breadth { up, down, total } }));
    let items: Vec<HeatItem> = d.as_ref().map(|h| h.items.clone()).unwrap_or_default();
    let has_data = d.is_some();
    let err_msg = err.read().clone();
    let max_w = items.iter().map(|i| i.weight).fold(0.0, f64::max);

    rsx! {
        div { class: "modal-bg", onmousedown: move |_| bg_down.set(true), onclick: move |_| { if *bg_down.peek() { st.heat_group.set(None); } bg_down.set(false); },
            div { class: "modal heat resizable", onmousedown: move |e| { e.stop_propagation(); bg_down.set(false); }, onclick: move |e| e.stop_propagation(),
                div { class: "modal-head",
                    span { class: "modal-title", "{title}" }
                    {
                        match breadth {
                            Some(b) => rsx! {
                                span { class: "breadth",
                                    span { class: "text-up num", "▲ {b.up}" }
                                    span { class: "text-down num", "▼ {b.down}" }
                                    span { class: "muted num", "of {b.total}" }
                                }
                            },
                            None => rsx! {},
                        }
                    }
                    span { class: "legend",
                        span { class: "legend-bar" }
                        span { class: "muted xs", "-3%" } span { class: "muted xs", "0" } span { class: "muted xs", "+3%" }
                    }
                    button { class: "mini", onclick: move |_| st.heat_group.set(None), "Close" }
                }
                div { class: "modal-body heat-body",
                    if *loading.read() { p { class: "empty", "Loading snapshot from IBKR..." } }
                    {
                        match err_msg {
                            Some(e) => rsx! { p { class: "empty text-down", "{e}" } },
                            None => rsx! {},
                        }
                    }
                    if has_data {
                        div { class: "heat-grid",
                            for it in items {
                                {
                                    let (cs, rs) = tile_span(it.weight, max_w);
                                    let bg = heat_color(it.change_pct);
                                    let last = if it.last <= 0.0 { "—".to_string() } else { format!("{:.*}", it.decimals, it.last) };
                                    let big = cs >= 2 && rs >= 2;        // big: show the price, large font
                                    let tall = rs >= 2;                   // at least 2 rows tall: room for 3 lines
                                    let k = it.key.clone();
                                    let tip = format!("{}  {}  {:+.2}%", it.name, last, it.change_pct);
                                    rsx! {
                                        div { class: if big { "tile big" } else if tall { "tile tall" } else { "tile" },
                                            style: "grid-column: span {cs}; grid-row: span {rs}; background: {bg};",
                                            title: "{tip}",
                                            onclick: move |_| { st.select(k.clone()); st.heat_group.set(None); },
                                            div { class: "tile-sym", "{it.symbol}" }
                                            div { class: "tile-chg num", "{it.change_pct:+.2}%" }
                                            if tall { div { class: "tile-px num", "{last}" } }
                                        }
                                    }
                                }
                            }
                        }
                        p { class: "muted xs heat-note", "Tile size reflects approximate index weight · click a tile to load it in the chart · snapshot cached 60 s" }
                    }
                }
            }
        }
    }
}

// ---------------- mode switch ----------------
#[component]
pub fn ModeSwitch() -> Element {
    let mut st = app_state();
    let mode = st.mode.read().clone();
    let n_active = st.snapshot.read().as_ref().map(|s| s.running_nodes.len()).unwrap_or(0);
    let can_switch = st.can_trade();
    let switch_tip = if can_switch { String::new() } else { format!("Your role ({}) cannot change the trading mode", st.role()) };
    let switch = move |m: &'static str| {
        spawn(async move {
            match set_mode(m.to_string()).await {
                Ok(new_mode) => {
                    st.mode.set(new_mode.clone());
                    let running = st.snapshot.peek().as_ref().map(|s| s.running_nodes.len()).unwrap_or(0);
                    let msg = if new_mode == "agent" { "Agent mode: manual ticket disabled, nodes are on the right".to_string() }
                        else if running > 0 { format!("Manual mode: {running} node(s) keep trading until you press Stop") }
                        else { "Manual mode".to_string() };
                    st.toast.set(Some((true, msg)));
                }
                Err(e) => st.toast.set(Some((false, format!("Mode change failed: {e}")))),
            }
        });
    };
    rsx! {
        div { class: "mode",
            button { class: if mode == "manual" { "mode-btn active" } else { "mode-btn" }, disabled: !can_switch, title: "{switch_tip}", onclick: move |_| switch("manual"), "Manual" }
            button { class: if mode == "agent" { "mode-btn active agent" } else { "mode-btn" }, disabled: !can_switch, title: "{switch_tip}", onclick: move |_| switch("agent"),
                "Agent"
                if n_active > 0 { span { class: "cnt live", "{n_active}" } }
            }
        }
    }
}

// ---------------- nodes side panel (Agent mode) ----------------
fn node_dot(state: &str) -> (&'static str, &'static str) {
    match state {
        "running" => ("nd-dot run", "Running"),
        "starting" => ("nd-dot run pulse", "Starting"),
        "degraded" => ("nd-dot warn", "Disconnected"),
        "backoff" => ("nd-dot warn pulse", "Restarting"),
        "crashed" => ("nd-dot bad", "Crashed"),
        _ => ("nd-dot", "Stopped"),
    }
}

#[component]
pub fn NodesSidePanel() -> Element {
    let mut st = app_state();
    let mut data = use_signal(NodesResp::default);
    let mut busy = use_signal(String::new);
    let mut filt = use_signal(|| "all");
    let mut tick = use_signal(|| 0u32);
    use_effect(move || {
        let _ = tick.read();
        spawn(async move { if let Ok(d) = get_nodes().await { data.set(d); } });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move { loop { crate::sleep_ms(5000).await; tick += 1; } });
    let w = *st.ticket_w.read();
    let d = data.read().clone();
    let busy_id = busy.read().clone();
    let live_map = st.node_live.read().clone();
    let n_run = d.nodes.iter().filter(|n| n.running).count();
    let f = *filt.read();
    let n_stop = d.nodes.len() - n_run;
    let shown: Vec<NodeInfo> = d.nodes.iter().filter(|n| f == "all" || (f == "running") == n.running).cloned().collect();
    rsx! {
        aside { class: "panel ticket agents", style: "width: {w}px",
            div { class: "panel-title",
                "Agents"
                span { class: "muted", "{n_run}/{d.nodes.len()} running" }
                button { class: "mini", title: "Open the Agents tab (all nodes, soak, logs)", onclick: move |_| { st.bottom_tab.set(6); let h = (*st.bottom_h.peek()).max(BOTTOM_DEFAULT); st.bottom_h.set(h); }, "Manage" }
            }
            div { class: "nd-filter",
                button { class: if f == "all" { "nd-chip active" } else { "nd-chip" }, onclick: move |_| filt.set("all"), "All {d.nodes.len()}" }
                button { class: if f == "running" { "nd-chip active" } else { "nd-chip" }, onclick: move |_| filt.set("running"), "Running {n_run}" }
                button { class: if f == "stopped" { "nd-chip active" } else { "nd-chip" }, onclick: move |_| filt.set("stopped"), "Stopped {n_stop}" }
            }
            if d.nodes.is_empty() { p { class: "empty small", "No nodes yet. Deploy a strategy from the Strategies tab." } }
            else if shown.is_empty() { p { class: "empty small", if f == "running" { "No running nodes." } else { "No stopped nodes." } } }
            div { class: "nd-list",
                for n in shown {
                    {
                        let (dcls, dlbl) = node_dot(&n.state);
                        let c = n.config.clone();
                        let pair = js(&c, &["instrument"]).split('.').next().unwrap_or_default().to_string();
                        let what = format!("{pair} {}m · {}", js(&c, &["tf_minutes"]), n.summary);
                        let hb = live_map.get(&n.id).and_then(|e| e.get("data")).cloned().unwrap_or_else(|| n.live.clone());
                        let pos: Vec<String> = if n.running { hb.get("positions").and_then(|p| p.as_array()).cloned().unwrap_or_default().iter()
                            .map(|p| format!("{} {}", if js(p, &["side"]) == "LONG" { "L" } else { "S" }, qty_fmt(jf(p, &["qty"])))).collect() } else { vec![] };
                        let pos_txt = if !n.running { "—".to_string() } else if pos.is_empty() { "flat".to_string() } else { pos.join(" · ") };
                        let pos_cls = if pos.is_empty() { "nd-pos muted" } else { "nd-pos" };
                        let realized = hb.get("realized_pnl").and_then(|x| x.as_f64()).filter(|_| n.running);
                        let id = n.id.clone();
                        let running = n.running;
                        let is_busy = busy_id == n.id;
                        let tip = format!("{} · {dlbl} · click for details", n.id);
                        let open_id = n.id.clone();
                        rsx! {
                            div { class: if running { "nd-row" } else { "nd-row off" }, title: "{tip}", onclick: move |_| st.node_detail.set(Some(open_id.clone())),
                                span { class: "{dcls}", title: "{dlbl}" }
                                div { class: "nd-main",
                                    div { class: "nd-line",
                                        span { class: "nd-name", "{n.name}" }
                                        span { class: "{pos_cls}", "{pos_txt}" }
                                    }
                                    div { class: "nd-line sub",
                                        span { class: "nd-what", "{what}" }
                                        if let Some(r) = realized { span { class: "num {pnl_class(r)}", "{money(r)}" } }
                                    }
                                }
                                button { class: if running { "nd-btn stop" } else { "nd-btn start" }, disabled: is_busy || !st.can_trade(),
                                    title: if running { "Stop node" } else { "Start node" },
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        let id = id.clone();
                                        if running {
                                            st.ask("Stop node?", format!("Stop {id}? The node cancels its orders and closes its positions when it stops."), "Stop node", ConfirmAction::NodeStop(id));
                                            return;
                                        }
                                        busy.set(id.clone());
                                        spawn(async move {
                                            let action = if running { "stop" } else { "start" };
                                            match node_action(id.clone(), action.to_string()).await {
                                                Ok(s) => st.toast.set(Some((true, format!("{id}: {action} → {s}")))),
                                                Err(e) => st.toast.set(Some((false, format!("{id}: {action} failed — {e}")))),
                                            }
                                            busy.set(String::new());
                                            tick += 1;
                                        });
                                    },
                                    if is_busy { "…" } else if running { "■" } else { "▶" }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "muted small nd-foot", "Nodes trade under the shared account risk limits · edit them in the Strategies tab" }
        }
    }
}

// ---------------- agent detail: equity & returns ----------------
fn short_date(t: &str) -> String {
    t.replace('T', " ").chars().take(10).collect()
}

fn axis_money(v: f64) -> String {
    if v.abs() >= 10_000.0 { format!("{:.1}k", v / 1000.0) } else { format!("{:.0}", v) }
}


#[component]
fn EquityDdChart(curve: Vec<(String, f64)>, base: f64, #[props(default)] overlay: Vec<(String, f64)>) -> Element {
    const W: f64 = 900.0; const H: f64 = 240.0; const PL: f64 = 6.0; const PR: f64 = 64.0; const PT: f64 = 10.0; const PB: f64 = 70.0; const DDH: f64 = 40.0;
    if curve.len() < 2 && overlay.len() < 2 {
        return rsx! { div { class: "chart-empty small", "The equity curve appears after the first closed trades" } };
    }
    let mut pts: Vec<(f64, f64)> = Vec::new();
    if let Some(f) = curve.first() { pts.push((iso_epoch(&f.0) - 1.0, 0.0)); }
    pts.extend(curve.iter().map(|p| (iso_epoch(&p.0), p.1)));
    // backtest over the same period: starts at the live value at the start time so the slopes compare directly
    let ov_start = overlay.first().map(|p| iso_epoch(&p.0));
    let offset = ov_start.map(|t0| pts.iter().filter(|p| p.0 <= t0).last().map(|p| p.1).unwrap_or(0.0)).unwrap_or(0.0);
    let mut ov: Vec<(f64, f64)> = Vec::new();
    if let Some(t0) = ov_start { ov.push((t0 - 1.0, offset)); }
    ov.extend(overlay.iter().map(|p| (iso_epoch(&p.0), p.1 + offset)));
    let all = pts.iter().chain(ov.iter());
    let (t0, t1, lo, hi) = all.fold((f64::MAX, f64::MIN, 0.0f64, 0.0f64), |(a, b, c, d), p| (a.min(p.0), b.max(p.0), c.min(p.1), d.max(p.1)));
    let tr = if t1 > t0 { t1 - t0 } else { 1.0 };
    let r = if hi > lo { hi - lo } else { 1.0 };
    let x = |t: f64| PL + (t - t0) / tr * (W - PL - PR);
    let y = |v: f64| PT + (hi - v) / r * (H - PT - PB);
    let path = |ps: &[(f64, f64)]| ps.iter().enumerate().map(|(i, p)| format!("{}{:.1},{:.1}", if i == 0 { "M" } else { "L" }, x(p.0), y(p.1))).collect::<Vec<_>>().join(" ");
    let has_live = pts.len() >= 2;
    let line = if has_live { path(&pts) } else { String::new() };
    let zero = y(0.0);
    let area = if has_live { format!("{line} L{:.1},{:.1} L{:.1},{:.1} Z", x(pts[pts.len() - 1].0), zero, x(pts[0].0), zero) } else { String::new() };
    let ov_line = if ov.len() >= 2 { path(&ov) } else { String::new() };
    let mut peak = f64::MIN;
    let dd: Vec<(f64, f64)> = pts.iter().map(|p| { peak = peak.max(p.1); (p.0, peak - p.1) }).collect();
    let max_dd = dd.iter().map(|p| p.1).fold(0.0, f64::max).max(0.01);
    let dd_top = H - PB + 14.0;
    let ydd = |d: f64| dd_top + d / max_dd * DDH;
    let dd_path = if has_live {
        dd.iter().enumerate().map(|(i, p)| format!("{}{:.1},{:.1}", if i == 0 { "M" } else { "L" }, x(p.0), ydd(p.1))).collect::<Vec<_>>().join(" ")
            + &format!(" L{:.1},{:.1} L{:.1},{:.1} Z", x(dd[dd.len() - 1].0), dd_top, x(dd[0].0), dd_top)
    } else { String::new() };
    let up = pts.last().map(|p| p.1 >= 0.0).unwrap_or(true);
    let ticks: Vec<(String, String, String)> = (0..=3).map(|i| {
        let v = hi - r * i as f64 / 3.0;
        (format!("{:.1}", y(v)), format!("{:.1}", y(v) + 4.0), format!("{}{}", axis_money(v), pct_of(v, base)))
    }).collect();
    let fmt_t = |t: f64| { let d = (t / 86400.0).floor() as i64; let z = d + 719468; let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
        let doe = z - era * 146097; let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153; let dd_ = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 }; let yy = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
        format!("{yy:04}-{m:02}-{dd_:02}") };
    let (first, mid, last) = (fmt_t(t0), fmt_t((t0 + t1) / 2.0), fmt_t(t1));
    let vb = format!("0 0 {W} {H}");
    let (zx2, zy) = (format!("{:.1}", W - PR), format!("{:.1}", zero));
    let lab_x = format!("{:.1}", W - PR + 6.0);
    let dd_lab_y = format!("{:.1}", dd_top + DDH);
    let dd_lab = format!("DD {}", axis_money(-max_dd));
    let xl_y = format!("{:.1}", H - 4.0);
    let mid_x = format!("{:.1}", (W - PR) / 2.0);
    let right_x = format!("{:.1}", W - PR);
    let dd_cap_y = format!("{:.1}", dd_top - 3.0);
    rsx! {
        svg { class: "eq-svg ad-eq", view_box: "{vb}", preserve_aspect_ratio: "none",
            for (ty, tl, lab) in ticks {
                line { x1: "{PL}", x2: "{zx2}", y1: "{ty}", y2: "{ty}", class: "grid" }
                text { x: "{lab_x}", y: "{tl}", class: "ylab", "{lab}" }
            }
            line { x1: "{PL}", x2: "{zx2}", y1: "{zy}", y2: "{zy}", class: "spark-base" }
            if has_live {
                path { d: "{area}", class: if up { "eq-area up" } else { "eq-area down" } }
                path { d: "{line}", class: if up { "eq-line up" } else { "eq-line down" } }
            }
            if !ov_line.is_empty() { path { d: "{ov_line}", class: "eq-overlay" } }
            if has_live {
                text { x: "{PL}", y: "{dd_cap_y}", class: "xlab left", "Drawdown" }
                path { d: "{dd_path}", class: "dd-area" }
                text { x: "{lab_x}", y: "{dd_lab_y}", class: "ylab", "{dd_lab}" }
            }
            text { x: "{PL}", y: "{xl_y}", class: "xlab left", "{first}" }
            text { x: "{mid_x}", y: "{xl_y}", class: "xlab", "{mid}" }
            text { x: "{right_x}", y: "{xl_y}", class: "xlab right", "{last}" }
        }
    }
}

#[component]
fn DailyBars(days: Vec<DayPnl>) -> Element {
    const W: f64 = 900.0; const H: f64 = 120.0; const PR: f64 = 64.0; const PT: f64 = 8.0; const PB: f64 = 18.0;
    if days.is_empty() { return rsx! { div { class: "chart-empty small", "No daily results yet" } }; }
    let lo = days.iter().map(|d| d.pnl).fold(0.0, f64::min);
    let hi = days.iter().map(|d| d.pnl).fold(0.0, f64::max);
    let r = if hi > lo { hi - lo } else { 1.0 };
    let y = |v: f64| PT + (hi - v) / r * (H - PT - PB);
    let n = days.len() as f64;
    let bw = ((W - PR) / n).max(1.0);
    let zero = y(0.0);
    let bars: Vec<(String, String, String, String, &'static str, String)> = days.iter().enumerate().map(|(i, d)| {
        let (y0, y1) = (y(d.pnl.max(0.0)), y(d.pnl.min(0.0)));
        (format!("{:.1}", i as f64 * bw + bw * 0.12), format!("{:.1}", y0), format!("{:.1}", (bw * 0.76).max(0.8)), format!("{:.1}", (y1 - y0).max(0.6)),
         if d.pnl >= 0.0 { "bar up" } else { "bar down" }, format!("{} · {} · {} trade(s)", d.d, money(d.pnl), d.n))
    }).collect();
    let vb = format!("0 0 {W} {H}");
    let (zy, zx2, lx) = (format!("{:.1}", zero), format!("{:.1}", W - PR), format!("{:.1}", W - PR + 6.0));
    let (hi_y, lo_y) = (format!("{:.1}", y(hi) + 8.0), format!("{:.1}", y(lo)));
    let (hi_l, lo_l) = (axis_money(hi), axis_money(lo));
    let (first, last) = (days[0].d.clone(), days[days.len() - 1].d.clone());
    let xl_y = format!("{:.1}", H - 3.0);
    rsx! {
        svg { class: "eq-svg ad-bars", view_box: "{vb}", preserve_aspect_ratio: "none",
            line { x1: "0", x2: "{zx2}", y1: "{zy}", y2: "{zy}", class: "spark-base" }
            for (bx, by, w, h, cls, tip) in bars { rect { x: "{bx}", y: "{by}", width: "{w}", height: "{h}", class: "{cls}", title { "{tip}" } } }
            text { x: "{lx}", y: "{hi_y}", class: "ylab", "{hi_l}" }
            text { x: "{lx}", y: "{lo_y}", class: "ylab", "{lo_l}" }
            text { x: "0", y: "{xl_y}", class: "xlab left", "{first}" }
            text { x: "{zx2}", y: "{xl_y}", class: "xlab right", "{last}" }
        }
    }
}

#[component]
fn PnlHistogram(pnls: Vec<f64>) -> Element {
    const W: f64 = 420.0; const H: f64 = 120.0; const PB: f64 = 18.0; const BINS: usize = 14;
    if pnls.len() < 3 { return rsx! { div { class: "chart-empty small", "Needs at least 3 closed trades" } }; }
    let lo = pnls.iter().cloned().fold(f64::MAX, f64::min);
    let hi = pnls.iter().cloned().fold(f64::MIN, f64::max);
    let span = if hi > lo { hi - lo } else { 1.0 };
    let mut counts = vec![0usize; BINS];
    for p in &pnls {
        let i = (((p - lo) / span) * BINS as f64).floor() as usize;
        counts[i.min(BINS - 1)] += 1;
    }
    let cmax = *counts.iter().max().unwrap_or(&1) as f64;
    let bw = W / BINS as f64;
    let bars: Vec<(String, String, String, String, &'static str, String)> = counts.iter().enumerate().map(|(i, c)| {
        let a = lo + span * i as f64 / BINS as f64;
        let b = lo + span * (i + 1) as f64 / BINS as f64;
        let h = *c as f64 / cmax * (H - PB - 6.0);
        (format!("{:.1}", i as f64 * bw + 1.0), format!("{:.1}", H - PB - h), format!("{:.1}", bw - 2.0), format!("{:.1}", h.max(0.0)),
         if (a + b) / 2.0 >= 0.0 { "bar up" } else { "bar down" }, format!("{} … {}: {c} trade(s)", money(a), money(b)))
    }).collect();
    let vb = format!("0 0 {W} {H}");
    let (l, rr) = (axis_money(lo), axis_money(hi));
    let xl_y = format!("{:.1}", H - 3.0);
    rsx! {
        svg { class: "eq-svg ad-hist", view_box: "{vb}", preserve_aspect_ratio: "none",
            for (bx, by, w, h, cls, tip) in bars { rect { x: "{bx}", y: "{by}", width: "{w}", height: "{h}", class: "{cls}", title { "{tip}" } } }
            text { x: "0", y: "{xl_y}", class: "xlab left", "{l}" }
            text { x: "{W}", y: "{xl_y}", class: "xlab right", "{rr}" }
        }
    }
}

#[component]
fn MonthlyTable(months: Vec<MonthPnl>) -> Element {
    if months.is_empty() { return rsx! { div { class: "chart-empty small", "No monthly results yet" } }; }
    let maxabs = months.iter().map(|m| m.pnl.abs()).fold(0.0, f64::max).max(0.01);
    let rows: Vec<(String, String, &'static str, String, String, String, String, &'static str)> = months.iter().rev().map(|m| {
        let w = format!("{:.0}%", m.pnl.abs() / maxabs * 100.0);
        let cls = pnl_class(m.pnl);
        let bar = if m.pnl >= 0.0 { "mbar up" } else { "mbar down" };
        (m.m.clone(), money(m.pnl), cls, format!("{:+.2}%", m.return_pct), format!("{}", m.n),
         format!("{}/{}", m.positive_days, m.active_days), w, bar)
    }).collect();
    rsx! {
        table { class: "grid-tbl compact ad-month",
            thead { tr { th { "Month" } th { class: "r", "P&L" } th { class: "r", "Return" } th { class: "r", "Trades" } th { class: "r", "Green days" } th { "" } } }
            tbody {
                for (m, pnl, cls, ret, n, days, w, bar) in rows {
                    tr {
                        td { class: "num", "{m}" }
                        td { class: "r num {cls}", "{pnl}" }
                        td { class: "r num {cls}", "{ret}" }
                        td { class: "r num", "{n}" }
                        td { class: "r num muted", "{days}" }
                        td { class: "mbar-cell", div { class: "{bar}", style: "width: {w}" } }
                    }
                }
            }
        }
    }
}

#[component]
fn PerfSection(live: PerfBlock, bt: Option<PerfBlock>, expected: Option<ExpectedPerf>, acct: String) -> Element {
    let mut view = use_signal(|| "live");
    let mut show_exp = use_signal(|| true);
    let v = *view.read();
    let has_bt = bt.as_ref().map(|b| !b.curve.is_empty()).unwrap_or(false);
    let is_bt = v == "backtest" && has_bt;
    let pb = if is_bt { bt.clone().unwrap_or_default() } else { live.clone() };
    let m = pb.metrics.clone();
    let curve: Vec<(String, f64)> = pb.curve.clone();
    let empty = curve.is_empty();
    let exp = expected.clone().filter(|e| !e.from.is_empty());
    let has_exp = !is_bt && exp.is_some();
    let overlay: Vec<(String, f64)> = if has_exp && *show_exp.read() { exp.as_ref().map(|e| e.perf.curve.clone()).unwrap_or_default() } else { vec![] };
    let cmp_txt = exp.as_ref().map(|e| {
        let t0 = iso_epoch(&e.from);
        let live_since: f64 = live.curve.iter().filter(|p| iso_epoch(&p.0) > t0).count() as f64;
        let start_v = live.curve.iter().filter(|p| iso_epoch(&p.0) <= t0).last().map(|p| p.1).unwrap_or(0.0);
        let live_net = live.curve.last().map(|p| p.1).unwrap_or(0.0) - start_v;
        format!("Since {}: live {} ({} trades) vs backtest {} ({} trades) on the same bars · parity run {}",
                short_date(&e.from), money(live_net), live_since as i64, money(e.perf.metrics.net), e.perf.metrics.trades as i64, jdate(&e.ts))
    }).unwrap_or_default();
    let pf = if m.profit_factor >= 999.0 { "∞".to_string() } else { format!("{:.2}", m.profit_factor) };
    let cards: Vec<(&'static str, String, &'static str)> = vec![
        ("Return", format!("{:+.2}%", m.return_pct), pnl_class(m.return_pct)),
        ("Net P&L", money(m.net), pnl_class(m.net)),
        ("Sharpe (daily)", format!("{:.2}", m.sharpe), if m.sharpe >= 1.0 { "text-up" } else if m.sharpe < 0.0 { "text-down" } else { "" }),
        ("Max drawdown", format!("{} ({:.2}%)", money(m.max_drawdown), m.max_drawdown_pct), "text-down"),
        ("Longest drawdown", format!("{} days", m.longest_drawdown_days as i64), ""),
        ("Profit factor", pf, if m.profit_factor >= 1.0 { "text-up" } else { "text-down" }),
        ("Expectancy / trade", money(m.expectancy), pnl_class(m.expectancy)),
        ("Win rate", format!("{:.1}% of {}", m.win_rate, m.trades as i64), ""),
        ("Avg win / loss", format!("{} / {}", money(m.avg_win), money(m.avg_loss)), ""),
        ("Best / worst trade", format!("{} / {}", money(m.best_trade), money(m.worst_trade)), ""),
        ("Best / worst day", format!("{} / {}", money(m.best_day), money(m.worst_day)), ""),
        ("Positive days", format!("{:.0}% of {}", m.positive_days_pct, m.active_days as i64), ""),
    ];
    let period = if empty { String::new() } else { format!("{} → {} · base {}", short_date(&m.from), short_date(&m.to), money(pb.base)) };
    let title = if is_bt { "Backtest equity (latest report for this config)".to_string() } else { format!("Live equity on {acct} (closed trades, after commission)") };
    let exp_on = *show_exp.read();
    rsx! {
        div { class: "ad-card ad-perf",
            div { class: "ad-card-head",
                h4 { "Equity & returns" }
                div { class: "ad-head-right",
                    if !is_bt {
                        button { class: if exp_on && has_exp { "nd-chip active" } else { "nd-chip" }, disabled: !has_exp,
                            title: if has_exp { "Show or hide the backtest of the same period on the chart" } else { "Available after the next soak parity run" },
                            onclick: move |_| { let cur = *show_exp.peek(); show_exp.set(!cur); },
                            if exp_on && has_exp { "Backtest overlay: on" } else { "Backtest overlay: off" } }
                    }
                    div { class: "tfs",
                        button { class: if v == "live" { "tf active" } else { "tf" }, onclick: move |_| view.set("live"), "Live" }
                        button { class: if v == "backtest" { "tf active" } else { "tf" }, disabled: !has_bt, title: if has_bt { "" } else { "No backtest for this config yet" }, onclick: move |_| view.set("backtest"), "Backtest" }
                    }
                }
            }
            div { class: "muted small", "{title}" span { class: "num", "  {period}" } }
            if has_exp && exp_on {
                div { class: "ad-legend small",
                    span { class: "lg live", "Live" }
                    span { class: "lg exp", "Backtest, same period" }
                    span { class: "muted", "{cmp_txt}" }
                }
            } else if !is_bt && !has_exp {
                div { class: "muted small", "Backtest overlay for the same period appears after the next soak parity run (daily after 06:00, or Run parity)." }
            }
            if empty && overlay.is_empty() {
                p { class: "empty small", if v == "live" { "No closed trades from this node on this account yet — the curve builds up as trades close." } else { "No backtest trades." } }
            } else {
                if !empty {
                    div { class: "ad-metrics",
                        for (lbl, val, cls) in cards { div { span { class: "lbl", "{lbl}" } span { class: "val num {cls}", "{val}" } } }
                    }
                }
                EquityDdChart { curve: curve, base: pb.base, overlay: overlay }
                if !empty {
                    div { class: "ad-sub",
                        div { class: "ad-sub-main", div { class: "muted small", "Daily P&L" } DailyBars { days: pb.daily.clone() } }
                        div { class: "ad-sub-side", div { class: "muted small", "P&L per trade (distribution)" } PnlHistogram { pnls: pb.trade_pnl.clone() } }
                    }
                    div { class: "muted small", "Monthly" }
                    MonthlyTable { months: pb.monthly.clone() }
                }
            }
        }
    }
}

// ---------------- agent detail modal ----------------
#[component]
fn Spark(values: Vec<f64>, #[props(default = 46.0)] h: f64, #[props(default)] base: Option<f64>) -> Element {
    const W: f64 = 300.0;
    if values.len() < 2 {
        return rsx! { div { class: "spark-empty muted small", "not enough data yet" } };
    }
    let b = base.unwrap_or(values[0]);
    let (lo, hi) = values.iter().chain(std::iter::once(&b)).fold((f64::MAX, f64::MIN), |(a, c), v| (a.min(*v), c.max(*v)));
    let r = if hi > lo { hi - lo } else { 1.0 };
    let n = values.len() as f64;
    let y = |v: f64| 2.0 + (hi - v) / r * (h - 4.0);
    let pts = values.iter().enumerate().map(|(i, v)| format!("{:.1},{:.1}", i as f64 / (n - 1.0) * W, y(*v))).collect::<Vec<_>>().join(" ");
    let cls = if values[values.len() - 1] >= b { "spark-line up" } else { "spark-line down" };
    let by = format!("{:.1}", y(b));
    let vb = format!("0 0 {W} {h}");
    rsx! {
        svg { class: "spark", view_box: "{vb}", preserve_aspect_ratio: "none",
            line { x1: "0", x2: "{W}", y1: "{by}", y2: "{by}", class: "spark-base" }
            polyline { points: "{pts}", class: "{cls}" }
        }
    }
}

fn pct(v: Option<f64>) -> String {
    v.map(|x| format!("{:.1}%", x * 100.0)).unwrap_or_else(|| "—".into())
}

fn day10(s: &str) -> String {
    s.chars().take(10).collect()
}

#[derive(Clone, PartialEq)]
struct AdTrade { entry: String, side: String, qty: String, entry_px: String, exit_px: String, hold: String, net: String, net_cls: &'static str }

#[derive(Clone, PartialEq)]
struct AdEvent { ts: String, kind: String, text: String }

#[derive(Clone, PartialEq)]
struct AdCheck { status: String, icon: &'static str, name: String, value: String }

// ---------------- live ramp-up card (step 14) ----------------
fn ramp_check_label(name: &str) -> &'static str {
    match name {
        "loss" => "Net P&L this stage", "drawdown" => "Drawdown this stage", "crashes" => "Crashes", "rejects" => "Rejected orders",
        "reconcile" => "Reconcile issues", "days" => "Trading days at this size", "trades" => "Closed trades", "parity" => "Parity vs backtest",
        _ => "Check",
    }
}

#[component]
fn RampCard(nid: String, view: serde_json::Value, on_change: EventHandler<()>) -> Element {
    let mut st = app_state();
    let mut code = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let rp = view.get("ramp").cloned().unwrap_or(serde_json::Value::Null);
    let stages: Vec<f64> = rp.get("config").and_then(|c| c.get("stages")).and_then(|s| s.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_f64()).collect()).unwrap_or_default();
    let stage = rp.get("stage").and_then(|x| x.as_i64()).unwrap_or(0) as usize;
    let paused = js(&rp, &["status"]) == "paused";
    let ev = view.get("evaluation").cloned().unwrap_or(serde_json::Value::Null);
    let action = js(&ev, &["action"]);
    let reasons: Vec<String> = ev.get("reasons").and_then(|r| r.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let checks: Vec<serde_json::Value> = ev.get("checks").and_then(|c| c.as_array()).cloned().unwrap_or_default();
    let can_up = action == "advance" && !paused;
    let last = stages.len().saturating_sub(1);
    let base = js(&view, &["base_trade_size"]);
    let live = js(&view, &["live_trade_size"]);
    let running = view.get("running_size").and_then(|r| r.get("trade_size")).and_then(|x| x.as_str()).map(String::from);
    let cap = jf(&view, &["cap_usd"]);
    let pending = view.get("restart_pending").and_then(|x| x.as_bool()).unwrap_or(false);
    let auto = rp.get("config").and_then(|c| c.get("auto_advance")).and_then(|x| x.as_bool()).unwrap_or(false);
    let note = js(&rp, &["note"]);
    let since = jdate(&js(&rp, &["stage_started"]));
    let mut act = move |what: &'static str, id: String| {
        let c = code.peek().clone();
        busy.set(true);
        spawn(async move {
            match node_ramp_action(id.clone(), what.to_string(), c, false).await {
                Ok(_) => { st.toast.set(Some((true, format!("{id}: ramp {what}")))); code.set(String::new()); on_change.call(()); }
                Err(e) => st.toast.set(Some((false, format!("{id}: {e}")))),
            }
            busy.set(false);
        });
    };
    let busy_now = *busy.read();
    let (i1, i2, i3, i4) = (nid.clone(), nid.clone(), nid.clone(), nid.clone());
    rsx! {
        div { class: "ad-card pkg-ramp",
            div { class: "ad-card-head",
                h4 { "Live ramp-up" }
                span { class: if paused { "pill bad" } else if stage >= last { "pill ok" } else { "pill warn" },
                    if paused { "paused" } else if stage >= last { "full size" } else { "ramping" } }
            }
            div { class: "pkg-ramp-steps",
                for (i, m) in stages.iter().enumerate() {
                    span { class: if i < stage { "pkg-step done" } else if i == stage { "pkg-step cur" } else { "pkg-step" }, "{(m * 100.0).round()}%" }
                }
            }
            div { class: "muted small",
                "Size {live} of {base} units · loss cap this stage {money(cap)} · since {since}"
                if let Some(r) = running.clone() { if r != live { span { class: "text-down", " · node still running {r} — restarts when flat" } } }
                if pending { " · restart pending (waits until flat)" }
            }
            if !note.is_empty() { div { class: "small", "Last change: {note}" } }
            table { class: "grid-tbl compact pkg-ramp-tbl",
                tbody {
                    for c in checks.iter() {
                        {
                            let ok = c.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
                            let v = c.get("value").map(|x| if x.is_null() { "not run".to_string() } else { js(c, &["value"]) }).unwrap_or_default();
                            rsx! { tr {
                                td { span { class: if ok { "text-up" } else { "text-down" }, if ok { "✓" } else { "✕" } } }
                                td { "{ramp_check_label(&js(c, &[\"name\"]))}" }
                                td { class: "r num", "{v}" }
                                td { class: "muted small", "{js(c, &[\"target\"])}" }
                            } }
                        }
                    }
                }
            }
            div { class: "muted small", "{reasons.join(\" · \")}" if auto { " · auto step-up on" } }
            div { class: "pkg-ramp-btns",
                button { class: "mini accent", disabled: !can_up || busy_now || stage >= last,
                    title: if can_up { "Step up to the next size" } else { "All criteria must pass first" },
                    onclick: move |_| act("advance", i1.clone()), "Step up" }
                button { class: "mini", disabled: stage == 0 || busy_now, title: "Go back to the previous size", onclick: move |_| act("demote", i2.clone()), "Step down" }
                input { class: "pkg-ramp-code", r#type: "password", placeholder: "Live code", value: "{code}", oninput: move |e| code.set(e.value()) }
                if paused {
                    button { class: "mini accent", disabled: code.read().is_empty() || busy_now, title: "Resume ramping at the current size", onclick: move |_| act("resume", i3.clone()), "Resume" }
                }
                button { class: "mini", disabled: code.read().is_empty() || busy_now, title: "Back to the smallest size", onclick: move |_| act("reset", i4.clone()), "Restart ramp" }
            }
        }
    }
}

#[component]
pub fn AgentDetailModal() -> Element {
    let mut st = app_state();
    let mut bg_down = use_signal(|| false);
    let mut maxed = use_signal(|| false);
    let mut data = use_signal(|| None::<NodeSummary>);
    let mut range = use_signal(|| "all".to_string());
    let mut err = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut tick = use_signal(|| 0u32);
    use_effect(move || {
        let id = st.node_detail.read().clone();
        let r = range.read().clone();
        let tz = st.tz.read().clone();
        let _ = tick.read();
        if let Some(id) = id {
            spawn(async move {
                match get_node_summary(id, r, tz).await {
                    Ok(d) => { data.set(Some(d)); err.set(String::new()); }
                    Err(e) => err.set(e.to_string()),
                }
            });
        } else {
            data.set(None);
        }
    });
    let open = st.node_detail.read().clone();
    let Some(id) = open else { return rsx! {} };
    let loaded = data.read().clone().filter(|d| d.node.id == id);
    let cur_range = range.read().clone();
    let err_txt = err.read().clone();
    let busy_now = *busy.read();

    let Some(d) = loaded else {
        let msg = if err_txt.is_empty() { "Loading…".to_string() } else { err_txt };
        return rsx! {
            div { class: "modal-bg", onmousedown: move |_| bg_down.set(true), onclick: move |_| { if *bg_down.peek() { st.node_detail.set(None); } bg_down.set(false); },
                div { class: if *maxed.read() { "modal agent-modal resizable maximized" } else { "modal agent-modal resizable" }, onmousedown: move |e| { e.stop_propagation(); bg_down.set(false); }, onclick: move |e| e.stop_propagation(),
                    div { class: "modal-head", span { class: "modal-title", "{id}" } div { class: "ad-actions", button { class: "mini", onclick: move |_| st.node_detail.set(None), "✕" } } }
                    div { class: "modal-body", p { class: "empty", "{msg}" } }
                }
            }
        };
    };

    let n = d.node.clone();
    let (dcls, dlbl) = node_dot(&n.state);
    let c = n.config.clone();
    let pair = js(&c, &["instrument"]).split('.').next().unwrap_or_default().to_string();
    let sltp = {
        let (sl, tp) = (jf(&c, &["sl_pips"]), jf(&c, &["tp_pips"]));
        if sl > 0.0 || tp > 0.0 { format!("SL {sl} / TP {tp} pips") } else { "no SL/TP".to_string() }
    };
    let cfg_line = format!("{pair} · {}m · {} · size {} · {sltp} · config {}", js(&c, &["tf_minutes"]), n.summary, qty_fmt(jf(&c, &["trade_size"])), n.config_file);
    let head_sub = format!("{} · {dlbl} · up {}", n.id, n.uptime_s.map(|s| dur_fmt(s as f64)).unwrap_or_else(|| "—".into()));
    let acct = if d.account.is_empty() { "all accounts".to_string() } else { d.account.clone() };
    let perf_title = format!("Performance on {acct}");

    // live
    let lv = n.live.clone();
    let positions: Vec<serde_json::Value> = lv.get("positions").and_then(|p| p.as_array()).cloned().unwrap_or_default();
    let pos_txt = if !n.running {
        "node stopped".to_string()
    } else if positions.is_empty() {
        "flat".to_string()
    } else {
        positions.iter().map(|p| format!("{} {} @ {}", js(p, &["side"]), qty_fmt(jf(p, &["qty"])), jf(p, &["avg_px"]))).collect::<Vec<_>>().join(" · ")
    };
    let upnl: f64 = positions.iter().map(|p| jf(p, &["unrealized_pnl"])).sum();
    let upnl_txt = money(upnl);
    let upnl_cls = format!("num {}", pnl_class(upnl));
    let realized = lv.get("realized_pnl").and_then(|x| x.as_f64());
    let real_txt = realized.map(money).unwrap_or_else(|| "—".into());
    let real_cls = format!("num {}", pnl_class(realized.unwrap_or(0.0)));
    let sig = lv.get("last_signal").cloned().unwrap_or(serde_json::Value::Null);
    let sig_txt = if sig.is_null() { "—".to_string() } else { format!("{} · {}", js(&sig, &["name"]), jdate(&js(&sig, &["ts"]))) };
    let rk = lv.get("risk").cloned().unwrap_or(serde_json::Value::Null);
    let (rcls, rlbl) = node_risk_pill(&rk, n.running);
    let open_orders = lv.get("open_orders").map(|x| x.to_string()).unwrap_or_else(|| "—".into());

    // performance
    let s = d.stats.clone();
    let net_txt = money(s.net_pnl);
    let net_cls = format!("val num {}", pnl_class(s.net_pnl));
    let trades_txt = format!("{} · {} open", s.trades, s.open_trades);
    let win_txt = format!("{:.0}%", s.win_rate);
    let pf_txt = if s.profit_factor >= 999.0 { "∞".to_string() } else { format!("{:.2}", s.profit_factor) };
    let dd_txt = money(s.max_drawdown);
    let hold_txt = dur_fmt(s.avg_holding_sec);
    let curve: Vec<f64> = d.curve.iter().map(|p| p.v).collect();

    // research
    let rs = d.research.clone();
    let bt = rs.get("backtest").cloned().unwrap_or(serde_json::Value::Null);
    let has_bt = !bt.is_null();
    let bt_file = js(&bt, &["file"]);
    let bt_period = format!("{} → {}", day10(&js(&bt, &["from"])), day10(&js(&bt, &["to"])));
    let bt_pnl = jf(&bt, &["pnl_usd"]);
    let bt_pnl_txt = format!("{} · {:.2}%", money(bt_pnl), jf(&bt, &["return_pct"]));
    let bt_pnl_cls = format!("num {}", pnl_class(bt_pnl));
    let bt_trades = format!("{} · win {}", jf(&bt, &["closed_positions"]) as i64, pct(bt.get("win_rate").and_then(|x| x.as_f64())));
    let bt_dd = money(jf(&bt, &["max_drawdown_usd"]));
    let bt_eq: Vec<f64> = bt.get("equity").and_then(|e| e.as_array()).map(|a| a.iter().filter_map(|p| p.get(1).and_then(|v| v.as_f64())).collect()).unwrap_or_default();
    let par = rs.get("parity").cloned().unwrap_or(serde_json::Value::Null);
    let par_txt = if par.is_null() { "—".to_string() } else { js(&par, &["label"]) };
    let par_when = if par.is_null() { "—".to_string() } else { jdate(&js(&par, &["created"])) };
    let wf = rs.get("walkforward").cloned().unwrap_or(serde_json::Value::Null);
    let wf_file = js(&wf, &["file"]);
    let wf_pnl = jf(&wf, &["pnl"]);
    let wf_txt = if wf.is_null() { "—".to_string() } else { format!("{} · {}", js(&wf, &["label"]), money(wf_pnl)) };
    let wf_cls = format!("num {}", pnl_class(wf_pnl));
    let bt_perf: Option<PerfBlock> = rs.get("backtest_perf").and_then(|v| serde_json::from_value(v.clone()).ok());
    let live_perf = d.live_perf.clone();
    let expected_perf = d.expected_perf.clone();

    // soak
    let soak = d.soak.clone();
    let soak_cls = soak_pill(&soak.verdict);
    let soak_since = soak.since.clone().map(|x| jdate(&x)).unwrap_or_else(|| "—".into());
    let checks: Vec<AdCheck> = soak.checks.iter().map(|ck| AdCheck {
        status: format!("soak-check {}", ck.status),
        icon: if ck.status == "pass" { "✓" } else if ck.status == "fail" { "✕" } else { "…" },
        name: ck.name.clone(),
        value: soak_val(&ck.value),
    }).collect();

    // trades + events
    let trades: Vec<AdTrade> = d.trades.iter().map(|t| AdTrade {
        entry: t.entry_ts.replace('T', " ").chars().take(16).collect(),
        side: t.side.clone(),
        qty: qty_fmt(t.qty),
        entry_px: t.entry_price.map(|v| px(v, decimals_for_key(&t.key))).unwrap_or_else(|| "—".into()),
        exit_px: t.exit_price.map(|x| px(x, decimals_for_key(&t.key))).unwrap_or_else(|| "open".into()),
        hold: t.holding_sec.map(dur_fmt).unwrap_or_default(),
        net: if t.open == 1 { "open".into() } else { t.net_pnl.map(money).unwrap_or_else(|| "— (no USD rate)".into()) },
        net_cls: pnl_class(t.net_pnl.unwrap_or(0.0)),
    }).collect();
    let events: Vec<AdEvent> = d.events.iter().map(|ev| {
        let dd = ev.get("data").cloned().unwrap_or(serde_json::Value::Null);
        let parts = [js(&dd, &["name"]), js(&dd, &["event"]), js(&dd, &["side"]), js(&dd, &["qty"]), js(&dd, &["price"]), js(&dd, &["reason"])];
        AdEvent {
            ts: jdate(&js(ev, &["ts"])).chars().skip(5).collect(),
            kind: js(ev, &["type"]),
            text: parts.iter().filter(|x| !x.is_empty()).cloned().collect::<Vec<_>>().join(" "),
        }
    }).collect();
    let no_trades = format!("No trades from this node on {acct} in this range.");

    let running = n.running;
    let (id_run, id_par) = (n.id.clone(), n.id.clone());
    let pkg_ref = { let r = js(&d.package, &["ref"]); if r.is_empty() { n.strategy_ref.clone().or(n.strategy_latest.clone()).unwrap_or_default() } else { r } };
    let pkg_state = js(&d.package, &["state"]);
    let pkg_note = js(&d.package, &["note"]);
    let pkg_newer = n.running && n.strategy_latest.as_deref().map(|l| l != pkg_ref).unwrap_or(false);
    let pkg_link = pkg_ref.clone();
    rsx! {
        div { class: "modal-bg", onmousedown: move |_| bg_down.set(true), onclick: move |_| { if *bg_down.peek() { st.node_detail.set(None); } bg_down.set(false); },
            div { class: if *maxed.read() { "modal agent-modal resizable maximized" } else { "modal agent-modal resizable" }, onmousedown: move |e| { e.stop_propagation(); bg_down.set(false); }, onclick: move |e| e.stop_propagation(),
                div { class: "modal-head",
                    span { class: "{dcls}" }
                    span { class: "modal-title", "{n.name}" }
                    span { class: "muted small", "{head_sub}" }
                    span { class: "pill", title: "Live performance is for this account", "{acct}" }
                    div { class: "ad-actions",
                        button { class: if running { "mini danger" } else { "mini accent" }, disabled: busy_now || !st.can_trade(), onclick: move |_| {
                            let id = id_run.clone();
                            if running {
                                st.ask("Stop node?", format!("Stop {id}? The node cancels its orders and closes its positions when it stops."), "Stop node", ConfirmAction::NodeStop(id));
                                return;
                            }
                            busy.set(true);
                            spawn(async move {
                                let action = if running { "stop" } else { "start" };
                                match node_action(id.clone(), action.to_string()).await {
                                    Ok(s) => st.toast.set(Some((true, format!("{id}: {action} → {s}")))),
                                    Err(e) => st.toast.set(Some((false, format!("{id}: {action} failed — {e}")))),
                                }
                                busy.set(false);
                                tick += 1;
                            });
                        }, if running { "Stop" } else { "Start" } }
                        button { class: "mini", title: "Compare live decisions with a backtest on catalog bars", disabled: busy_now, onclick: move |_| {
                            let id = id_par.clone();
                            busy.set(true);
                            spawn(async move {
                                match soak_parity(id.clone()).await {
                                    Ok(v) => {
                                        let m = pct(v.get("live_vs_backtest").and_then(|x| x.get("match_rate")).and_then(|x| x.as_f64()));
                                        st.toast.set(Some((true, format!("{id}: parity {m}"))));
                                    }
                                    Err(e) => st.toast.set(Some((false, format!("{id}: parity failed — {e}")))),
                                }
                                busy.set(false);
                                tick += 1;
                            });
                        }, "Run parity" }
                        button { class: "mini", title: "Edit the config or run new tests", onclick: move |_| { st.node_detail.set(None); st.bottom_tab.set(11); }, "Strategy lab" }
                        button { class: "mini", title: if *maxed.read() { "Restore size (drag the bottom-right corner to resize)" } else { "Maximize (drag the bottom-right corner to resize)" }, onclick: move |_| { let v = !*maxed.peek(); maxed.set(v); }, if *maxed.read() { "⤡" } else { "⤢" } }
                        button { class: "mini", onclick: move |_| st.node_detail.set(None), "✕" }
                    }
                }
                div { class: "modal-body ad-body",
                    if !pkg_ref.is_empty() {
                        div { class: "pkg-ad",
                            span { class: "pkg-ad-ref", title: "Strategy package and version this node runs", "{pkg_ref}" }
                            span { class: "{pkg_badge_class(&pkg_state)}", title: "{pkg_note}", "{pkg_state_label(&pkg_state)}" }
                            if !pkg_note.is_empty() { span { class: "muted small", "{pkg_note}" } }
                            if pkg_newer { span { class: "muted small", "· restart to use {n.strategy_latest.clone().unwrap_or_default()}" } }
                            button { class: "linkbtn", title: "Open this package in the Strategies tab", onclick: move |_| {
                                st.strat_pkg.set(Some(pkg_link.clone()));
                                st.node_detail.set(None);
                                st.bottom_tab.set(11);
                            }, "Open package" }
                        }
                    }
                    div { class: "ad-cfg muted", "{cfg_line}" }
                    if !d.ramp.is_null() && !d.ramp.get("ramp").map(|r| r.is_null()).unwrap_or(true) {
                        RampCard { nid: n.id.clone(), view: d.ramp.clone(), on_change: move |_| tick += 1 }
                    }
                    div { class: "ad-grid",
                        div { class: "ad-card",
                            h4 { "Live now" }
                            div { class: "kv", span { "Position" } span { class: "num", "{pos_txt}" } }
                            div { class: "kv", span { "Unrealized" } span { class: "{upnl_cls}", "{upnl_txt}" } }
                            div { class: "kv", span { "Realized (this run)" } span { class: "{real_cls}", "{real_txt}" } }
                            div { class: "kv", span { "Last signal" } span { class: "num", "{sig_txt}" } }
                            div { class: "kv", span { "Risk" } span { class: "{rcls}", "{rlbl}" } }
                            div { class: "kv", span { "Open orders" } span { class: "num", "{open_orders}" } }
                        }
                        div { class: "ad-card",
                            div { class: "ad-card-head",
                                h4 { "{perf_title}" }
                                div { class: "tfs",
                                    for (rid, lbl) in [("today", "Today"), ("week", "7D"), ("month", "30D"), ("all", "All")] {
                                        button { class: if cur_range == rid { "tf active" } else { "tf" }, onclick: move |_| range.set(rid.to_string()), "{lbl}" }
                                    }
                                }
                            }
                            div { class: "ad-stats",
                                div { span { class: "lbl", "Net P&L" } span { class: "{net_cls}", "{net_txt}" } }
                                div { span { class: "lbl", "Trades" } span { class: "val num", "{trades_txt}" } }
                                div { span { class: "lbl", "Win rate" } span { class: "val num", "{win_txt}" } }
                                div { span { class: "lbl", "Profit factor" } span { class: "val num", "{pf_txt}" } }
                                div { span { class: "lbl", "Max drawdown" } span { class: "val num text-down", "{dd_txt}" } }
                                div { span { class: "lbl", "Avg hold" } span { class: "val num", "{hold_txt}" } }
                            }
                            div { class: "ad-spark-lbl muted small", "Cumulative P&L (realized − commission)" }
                            Spark { values: curve, h: 52.0, base: Some(0.0) }
                        }
                    }
                    PerfSection { live: live_perf, bt: bt_perf, expected: expected_perf, acct: acct.clone() }
                    div { class: "ad-grid three",
                        div { class: "ad-card",
                            div { class: "ad-card-head",
                                h4 { "Backtest" }
                                if !bt_file.is_empty() {
                                    button { class: "linkbtn", onclick: move |_| { st.node_detail.set(None); st.bt_open.set(Some(("backtest".to_string(), bt_file.clone()))); st.bottom_tab.set(9); }, "Open report" }
                                }
                            }
                            if !has_bt {
                                p { class: "empty small", "No backtest for this config yet — run one in the Strategy lab." }
                            } else {
                                div { class: "kv", span { "Period" } span { class: "num", "{bt_period}" } }
                                div { class: "kv", span { "P&L / return" } span { class: "{bt_pnl_cls}", "{bt_pnl_txt}" } }
                                div { class: "kv", span { "Trades" } span { class: "num", "{bt_trades}" } }
                                div { class: "kv", span { "Max drawdown" } span { class: "num text-down", "{bt_dd}" } }
                                Spark { values: bt_eq, h: 40.0 }
                            }
                        }
                        div { class: "ad-card",
                            h4 { "Parity & walk-forward" }
                            div { class: "kv", span { "Legacy parity" } span { class: "num", "{par_txt}" } }
                            div { class: "kv", span { "Parity run" } span { class: "num muted", "{par_when}" } }
                            div { class: "kv", span { "Walk-forward OOS" } span { class: "{wf_cls}", "{wf_txt}" } }
                            if !wf_file.is_empty() {
                                button { class: "linkbtn", onclick: move |_| { st.node_detail.set(None); st.bt_open.set(Some(("walkforward".to_string(), wf_file.clone()))); st.bottom_tab.set(9); }, "Open walk-forward" }
                            }
                        }
                        div { class: "ad-card",
                            div { class: "ad-card-head", h4 { "Paper soak" } span { class: "{soak_cls}", "{soak.verdict}" } }
                            div { class: "kv", span { "Since" } span { class: "num", "{soak_since}" } }
                            for ck in checks {
                                div { class: "{ck.status}",
                                    span { class: "soak-ic", "{ck.icon}" }
                                    span { class: "small", "{ck.name}" }
                                    span { class: "num small", "{ck.value}" }
                                }
                            }
                        }
                    }
                    div { class: "ad-grid split",
                        div { class: "ad-card",
                            h4 { "Recent trades" }
                            if trades.is_empty() {
                                p { class: "empty small", "{no_trades}" }
                            } else {
                                table { class: "grid-tbl compact",
                                    thead { tr { th { "Entry (UTC)" } th { "Side" } th { class: "r", "Qty" } th { class: "r", "Entry" } th { class: "r", "Exit" } th { class: "r", "Hold" } th { class: "r", "Net" } } }
                                    tbody {
                                        for t in trades {
                                            tr {
                                                td { class: "num muted", "{t.entry}" }
                                                td { class: if t.side == "LONG" { "text-up" } else { "text-down" }, "{t.side}" }
                                                td { class: "r num", "{t.qty}" }
                                                td { class: "r num", "{t.entry_px}" }
                                                td { class: "r num", "{t.exit_px}" }
                                                td { class: "r num muted", "{t.hold}" }
                                                td { class: "r num {t.net_cls}", "{t.net}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "ad-card",
                            h4 { "Activity" }
                            if events.is_empty() { p { class: "empty small", "No recent events." } }
                            for ev in events {
                                div { class: "ad-ev small",
                                    span { class: "num muted", "{ev.ts} " }
                                    span { class: "tag xs src-node", "{ev.kind}" }
                                    " {ev.text}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------- daily risk bar (header) ----------------
#[component]
pub fn DailyRisk() -> Element {
    let st = app_state();
    let r = st.risk.read().clone();
    let (pnl, limit, used, halted, reason) = match &r {
        Some(r) => (r.daily.pnl, r.daily.limit, r.daily.used_pct, r.halted, r.halt_reason.clone()),
        None => (0.0, 0.0, 0.0, false, String::new()),
    };
    let day = r.as_ref().map(|r| r.daily.date.clone()).unwrap_or_default();
    let baseline_ok = r.as_ref().map(|r| r.daily.baseline_confirmed).unwrap_or(true);
    let bar_cls = if halted { "bar halted" } else if used >= 75.0 { "bar warn" } else { "bar" };
    rsx! {
        div { class: "daily",
            div { class: "daily-top",
                span { class: "lbl", "Daily P&L" }
                span { class: "num {pnl_class(pnl)}", "{money(pnl)}" }
                if halted { span { class: "pill bad", title: "{reason}", "HALTED" } }
                if !baseline_ok { span { class: "pill warn", title: "Today's daily-loss baseline is not confirmed — orders are blocked; confirm it in the Risk tab", "BASELINE?" } }
            }
            div { class: "track", title: "Daily loss limit {money(limit)} USD · {used:.0}% used · trading day {day} (starts 17:00 New York)", div { class: "{bar_cls}", style: "width: {used:.0}%;" } }
        }
    }
}

// ---------------- kill switch ----------------
#[component]
pub fn KillSwitch() -> Element {
    let mut st = app_state();
    let mut arm = use_signal(|| false);
    let halted = st.risk.read().as_ref().map(|r| r.halted).unwrap_or(false);
    let (can_kill, can_risk) = (st.can_kill(), st.can_risk());
    rsx! {
        if halted && !can_risk {
            span { class: "pill bad", title: "Only a risk or admin operator can resume trading", "HALTED" }
        } else if !halted && !can_kill {
        } else if halted {
            button { class: "kill resume", title: "Clear the halt and allow trading again", onclick: move |_| {
                spawn(async move {
                    match resume_trading().await {
                        Ok(_) => st.toast.set(Some((true, "Trading resumed".into()))),
                        Err(e) => st.toast.set(Some((false, format!("Resume failed: {e}")))),
                    }
                    if let Ok(r) = get_risk().await { st.risk.set(Some(r)); }
                });
            }, "Resume trading" }
        } else if *arm.read() {
            button { class: "kill armed", onclick: move |_| {
                arm.set(false);
                spawn(async move {
                    match kill_switch().await {
                        Ok(_) => { st.mode.set("manual".into()); st.toast.set(Some((true, "KILL SWITCH: agents stopped, orders cancelled, positions closed, trading halted".into()))); }
                        Err(e) => st.toast.set(Some((false, format!("Kill switch failed: {e}")))),
                    }
                    if let Ok(r) = get_risk().await { st.risk.set(Some(r)); }
                    st.refresh_tick += 1;
                });
            }, "{KILL_CONFIRM_LABEL}" }
            button { class: "mini", onclick: move |_| arm.set(false), "Cancel" }
        } else {
            button { class: "kill", title: "Stop all agents, cancel all orders, close all positions and halt trading on ALL accounts", onclick: move |_| arm.set(true), "⏻ Kill" }
        }
    }
}

// ---------------- risk panel ----------------
#[component]
pub fn RiskPanel() -> Element {
    let mut st = app_state();
    let r = st.risk.read().clone().unwrap_or_default();
    let mut d_loss = use_signal(String::new);
    let mut d_order = use_signal(String::new);
    let mut d_pos = use_signal(String::new);
    let mut d_open = use_signal(String::new);
    let mut dirty = use_signal(|| false);
    let plain = |v: f64| if v.fract() == 0.0 { format!("{}", v as i64) } else { format!("{v}") };
    let sv = |dirty_: bool, draft: &Signal<String>, live: f64| if dirty_ { draft.read().clone() } else { plain(live) };
    let v_loss = sv(*dirty.read(), &d_loss, r.settings.max_daily_loss_usd);
    let v_order = sv(*dirty.read(), &d_order, r.settings.max_order_notional_usd);
    let v_pos = sv(*dirty.read(), &d_pos, r.settings.max_position_notional_usd);
    let v_open = if *dirty.read() { d_open.read().clone() } else { r.settings.max_open_positions.to_string() };
    let (b_loss, b_order, b_pos, b_open) = (r.settings.max_daily_loss_usd, r.settings.max_order_notional_usd, r.settings.max_position_notional_usd, r.settings.max_open_positions);
    let mut begin = move || {
        if !*dirty.peek() {
            d_loss.set(plain(b_loss));
            d_order.set(plain(b_order));
            d_pos.set(plain(b_pos));
            d_open.set(b_open.to_string());
            dirty.set(true);
        }
    };
    let save = move |_: Event<MouseData>| {
        // every field must be a valid number: a blank or mistyped field must never save as 0 (0 = daily loss limit OFF)
        use crate::calc::parse_required;
        let parsed = (|| -> Result<RiskSettings, String> {
            let open = parse_required("Max open positions", &d_open.read(), 1.0)?;
            if open.fract() != 0.0 || open > 100.0 { return Err("Max open positions must be a whole number from 1 to 100".into()); }
            let order = parse_required("Max notional per order", &d_order.read(), 0.0)?;
            let pos = parse_required("Max notional per symbol", &d_pos.read(), 0.0)?;
            if order <= 0.0 || pos <= 0.0 { return Err("Notional limits must be greater than 0".into()); }
            Ok(RiskSettings { allow_live: None, max_daily_loss_usd: parse_required("Max daily loss", &d_loss.read(), 0.0)?,
                              max_order_notional_usd: order, max_position_notional_usd: pos, max_open_positions: open as i64 })
        })();
        let settings = match parsed { Ok(x) => x, Err(e) => { st.toast.set(Some((false, format!("Not saved: {e}")))); return; } };
        spawn(async move {
            match put_risk(settings).await {
                Ok(ri) => { st.risk.set(Some(ri)); dirty.set(false); st.toast.set(Some((true, "Risk limits saved".into()))); }
                Err(e) => st.toast.set(Some((false, format!("Save failed: {e}")))),
            }
        });
    };
    let mut baseline_in = use_signal(String::new);
    let cur_netliq = r.daily.netliq;
    let bl_val = { let v = baseline_in.read().clone(); if v.is_empty() && cur_netliq > 0.0 { plain(cur_netliq) } else { v } };
    let a = r.account.clone();
    let margin_pct = if a.net_liquidation > 0.0 { a.maint_margin / a.net_liquidation * 100.0 } else { 0.0 };

    rsx! {
        div { class: "risk-grid four",
            div { class: "risk-col",
                h4 { "Account & margin" }
                div { class: "kv", span { "Net liquidation" } span { class: "num", "{money(a.net_liquidation)}" } }
                div { class: "kv", span { "Gross exposure (USD)" } span { class: "num", title: if a.gross_exposure_partial { "At least this much: a position could not be valued in USD" } else { "" }, if a.gross_exposure_partial { "≥ {money(a.gross_exposure)}" } else { "{money(a.gross_exposure)}" } } }
                div { class: "kv", span { "Leverage" } span { class: "num", "{a.leverage:.2}x" } }
                div { class: "kv", span { "Initial margin" } span { class: "num", "{money(a.init_margin)}" } }
                div { class: "kv", span { "Maintenance margin" } span { class: "num", "{money(a.maint_margin)}  ({margin_pct:.1}% of NLV)" } }
                div { class: "kv", span { "Excess liquidity" } span { class: "num {pnl_class(a.excess_liquidity)}", "{money(a.excess_liquidity)}" } }
                div { class: "kv", span { "Buying power" } span { class: "num", "{money(a.buying_power)}" } }
                div { class: "kv", span { "Open positions / working orders" } span { class: "num", "{r.open_positions} / {r.working_orders}" } }
                div { class: "kv", span { "Running nodes" } span { class: "num", if r.active_agents.is_empty() { "none" } else { "{r.active_agents.join(\", \")}" } } }
            }
            div { class: "risk-col",
                h4 { "Limits" }
                label { class: "fld", span { "Max daily loss (USD) · 0 = off" }
                    input { r#type: "number", step: "100", value: "{v_loss}", onfocus: move |_| begin(), oninput: move |e| { begin(); d_loss.set(e.value()); } } }
                label { class: "fld", span { "Max notional per order (USD)" }
                    input { r#type: "number", step: "1000", value: "{v_order}", onfocus: move |_| begin(), oninput: move |e| { begin(); d_order.set(e.value()); } } }
                label { class: "fld", span { "Max notional per symbol (USD)" }
                    input { r#type: "number", step: "1000", value: "{v_pos}", onfocus: move |_| begin(), oninput: move |e| { begin(); d_pos.set(e.value()); } } }
                label { class: "fld", span { "Max open positions" }
                    input { r#type: "number", step: "1", value: "{v_open}", onfocus: move |_| begin(), oninput: move |e| { begin(); d_open.set(e.value()); } } }
                button { class: if *dirty.read() { "submit confirm" } else { "submit disabled" }, disabled: !*dirty.read(), onclick: save, "Save limits" }
                if r.halted { div { class: "notice", "Trading is halted: {r.halt_reason}. Use Resume trading in the header to continue." } }
                if !r.daily.baseline_confirmed {
                    div { class: "notice",
                        b { "Daily-loss baseline not confirmed" }
                        " — the API could not record NetLiq at the start of today's trading day (17:00 New York), so new orders are blocked while a daily-loss limit is set. Enter today's starting NetLiq (prefilled with the current value) and confirm."
                        if st.can_risk() {
                            div { class: "btn-row",
                                input { class: "num", r#type: "number", step: "any", value: "{bl_val}", placeholder: "{plain(r.daily.netliq)}", oninput: move |e| baseline_in.set(e.value()) }
                                button { class: "mini primary", onclick: move |_| {
                                    let raw = baseline_in.peek().clone();
                                    let v = if raw.trim().is_empty() { None } else { match crate::calc::parse_required("Start NetLiq", &raw, 0.01) { Ok(v) => Some(v), Err(e) => { st.toast.set(Some((false, e))); return; } } };
                                    let shown = v.map(money).unwrap_or_else(|| format!("the current NetLiq ({})", money(cur_netliq)));
                                    st.ask("Confirm daily-loss baseline?", format!("Today's starting NetLiq = {shown}.\nToday's P&L and the daily-loss limit are measured from this value."), "Confirm baseline", ConfirmAction::ConfirmBaseline(v));
                                }, "Confirm baseline" }
                            }
                        } else { span { class: "muted small", " Ask a risk or admin operator to confirm it." } }
                    }
                }
                p { class: "muted xs", "Daily P&L resets at the start of the IBKR trading day (17:00 New York). When the loss limit is hit, all new orders are rejected until you resume." }
            }
            div { class: "risk-col wide",
                h4 { "Exposure by instrument" }
                if r.exposure.is_empty() { p { class: "empty small", "No open exposure" } } else {
                    table { class: "grid-tbl",
                        thead { tr { th { "Symbol" } th { class: "r", "Qty" } th { class: "r", "Market value" } th { class: "r", "% of gross" } th { class: "r", "Unrealized" } } }
                        tbody {
                            for e in r.exposure.clone() {
                                {
                                    let share_txt = if e.usd && a.gross_exposure > 0.0 { format!("{:.1}%", e.market_value.abs() / a.gross_exposure * 100.0) } else { "—".to_string() };
                                    rsx! {
                                        tr {
                                            td { class: "sym", "{e.symbol}" }
                                            td { class: "r num {pnl_class(e.qty)}", "{qty_fmt(e.qty)}" }
                                            td { class: "r num", "{money_in(e.market_value, e.usd, &e.currency)}" }
                                            td { class: "r num", "{share_txt}" }
                                            td { class: "r num {pnl_class(e.unrealized_pnl)}", "{money_in(e.unrealized_pnl, e.usd, &e.currency)}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            NotifySettings {}
            SecurityPanel {}
        }
    }
}

/// Security: allow Live, Live confirmation code, manage operators (admin only)
#[component]
fn SecurityPanel() -> Element {
    let mut st = app_state();
    let admin = st.is_admin();
    let sess = st.session.read().clone().unwrap_or_default();
    let allow_live = st.risk.read().as_ref().map(|r| r.settings.allow_live.unwrap_or(false)).unwrap_or(false);
    let mut ops_list = use_signal(Vec::<Operator>::new);
    let mut live_code = use_signal(String::new);
    let mut new_name = use_signal(String::new);
    let mut new_role = use_signal(|| "trader".to_string());
    let mut new_pin = use_signal(String::new);
    let mut msg = use_signal(String::new);
    let mut reload = use_signal(|| 0u32);
    use_effect(move || { let _ = reload.read(); if admin { spawn(async move { if let Ok(l) = op_list().await { ops_list.set(l); } }); } });
    let configured = st.op.read().as_ref().map(|o| o.configured).unwrap_or(false);
    let toggle_live = move |on: bool| {
        spawn(async move {
            let cur = st.risk.peek().as_ref().map(|r| r.settings.clone()).unwrap_or_default();
            let body = RiskSettings { allow_live: Some(on), ..cur };
            match put_risk(body).await { Ok(ri) => { st.risk.set(Some(ri)); st.notify("info", if on { "Live trading ENABLED by admin" } else { "Live trading disabled" }); }
                Err(e) => st.toast.set(Some((false, format!("Failed: {e}")))) }
        });
    };
    rsx! {
        div { class: "risk-col wide security",
            h4 { "Security & operators" }
            if !admin {
                p { class: "muted small", "Signed in as {st.role()} — only an admin can change live-trading permission, the live confirmation code and operators." }
            } else {
                div { class: "sec-row",
                    label { class: "chk", input { r#type: "checkbox", checked: allow_live, oninput: move |e| toggle_live(e.checked()) }
                        span { b { "Allow live trading" } " — required before anyone can sign in to a LIVE gateway or pick a live account" } }
                    span { class: if sess.live_code_set { "tag mint" } else { "tag rose" }, if sess.live_code_set { "live code set" } else { "no live code" } }
                }
                div { class: "sec-row",
                    input { r#type: "password", placeholder: "New live confirmation code (min 6 chars)", value: "{live_code}", oninput: move |e| live_code.set(e.value()) }
                    button { class: "mini", disabled: live_code.read().len() < 6, onclick: move |_| {
                        let c = live_code.peek().clone();
                        spawn(async move { match set_live_code(c).await { Ok(_) => { live_code.set(String::new()); msg.set("Live confirmation code updated".into()); }
                            Err(e) => msg.set(format!("Failed: {e}")) } });
                    }, "Set live code" }
                    span { class: "muted small", "Asked every time someone signs in to or switches to a live gateway (second factor on top of the IBKR password)." }
                }
                div { class: "sec-row", h5 { "Markets" } }
                MarketToggles {}
                div { class: "sec-row", h5 { "AI assistant (MCP — Claude / ChatGPT)" } }
                AssistantSettings {}
                div { class: "sec-row",
                    h5 { "Operators" }
                    if !configured { span { class: "tag peach xs", "not set up — everyone on this computer is admin" } }
                }
                table { class: "grid-tbl",
                    thead { tr { th { "ID" } th { "Name" } th { "Role" } th { "Status" } th { "" } } }
                    tbody {
                        for o in ops_list.read().clone() {
                            {
                                let (id, id2, id3) = (o.id.clone(), o.id.clone(), o.id.clone());
                                let active = o.active;
                                rsx! { tr {
                                    td { class: "num", "{o.id}" } td { "{o.name}" }
                                    td { Dropdown { class: "wl-market", value: "{o.role}", onchange: move |r: String| { let id = id.clone(); spawn(async move { match op_update(id, Some(r), None, None).await { Ok(_) => reload += 1, Err(e) => msg.set(format!("Failed: {e}")) } }); },
                                        options: dd_opts(["admin", "trader", "risk", "viewer", "assistant"].map(|r| (r, r))) } }
                                    td { span { class: if active { "tag mint xs" } else { "tag xs" }, if active { "active" } else { "disabled" } } }
                                    td {
                                        button { class: "mini", onclick: move |_| { let id = id2.clone(); spawn(async move { match op_update(id, None, Some(!active), None).await { Ok(_) => reload += 1, Err(e) => msg.set(format!("Failed: {e}")) } }); }, if active { "Disable" } else { "Enable" } }
                                        button { class: "mini", onclick: move |_| { let id = id3.clone(); let pin = new_pin.peek().clone(); if pin.len() < 4 { msg.set("Type a new PIN (min 4) in the PIN box first, then click Reset PIN".into()); return; } spawn(async move { match op_update(id, None, None, Some(pin)).await { Ok(_) => { new_pin.set(String::new()); msg.set("PIN reset".into()); } Err(e) => msg.set(format!("Failed: {e}")) } }); }, "Reset PIN" }
                                    }
                                } }
                            }
                        }
                    }
                }
                div { class: "sec-row",
                    input { placeholder: "Name", value: "{new_name}", oninput: move |e| new_name.set(e.value()) }
                    Dropdown { class: "wl-market", value: "{new_role}", onchange: move |v| new_role.set(v),
                        options: dd_opts([("admin", "admin"), ("trader", "trader"), ("risk", "risk"), ("viewer", "viewer"), ("assistant", "assistant (MCP)")]) }
                    input { r#type: "password", placeholder: "PIN (min 4)", value: "{new_pin}", oninput: move |e| new_pin.set(e.value()) }
                    button { class: "mini primary", disabled: new_name.read().trim().is_empty() || new_pin.read().len() < 4, onclick: move |_| {
                        let (n, r, pn) = (new_name.peek().clone(), new_role.peek().clone(), new_pin.peek().clone());
                        spawn(async move { match op_create(n, r, pn).await { Ok(o) => { msg.set(format!("Added operator {}", o.id)); new_name.set(String::new()); new_pin.set(String::new()); reload += 1; } Err(e) => msg.set(format!("Failed: {e}")) } });
                    }, "Add operator" }
                }
                p { class: "muted xs", "Roles: admin = everything · trader = orders, agents, mode · risk = risk limits, kill switch, notifications (no orders) · viewer = read-only. Sessions lock after 30 min idle." }
                if !msg.read().is_empty() { div { class: "notice", "{msg}" } }
            }
        }
    }
}

/// Enable/disable order submission per market (crypto is off by default: IBKR Regulatory Restrictions)
#[component]
fn MarketToggles() -> Element {
    let mut st = app_state();
    let mt = st.risk.read().as_ref().map(|r| r.markets_tradable.clone()).unwrap_or_default();
    let probe = st.risk.read().as_ref().map(|r| r.markets_probe.clone()).unwrap_or_default();
    let mut probing = use_signal(|| false);
    let rows = [("FX", "Forex"), ("STK", "US Stocks"), ("CRYPTO", "Crypto (Paxos)")];
    let probe_txt = if probe.ts.is_empty() { "IBKR permissions not checked yet".to_string() } else {
        format!("Checked {} on {}: {}", probe.ts.chars().take(16).collect::<String>().replace('T', " "), probe.account,
            probe.results.iter().map(|r| format!("{} {}", r.market, if r.allowed { "✓" } else { "✗" })).collect::<Vec<_>>().join(" · "))
    };
    let blocked: Vec<String> = probe.results.iter().filter(|r| !r.allowed).map(|r| format!("{}: {}", r.market, r.reason)).collect();
    rsx! {
        div { class: "sec-row",
            button { class: "mini", disabled: *probing.read(), onclick: move |_| {
                probing.set(true);
                spawn(async move { match probe_markets().await {
                    Ok(res) => { if let Ok(r) = get_risk().await { st.risk.set(Some(r)); }
                        let ok: Vec<String> = res.iter().filter(|r| r.allowed).map(|r| r.market.clone()).collect();
                        st.notify("info", format!("IBKR permissions checked — tradable: {}", if ok.is_empty() { "none".into() } else { ok.join(", ") })); }
                    Err(e) => st.toast.set(Some((false, format!("Permission check failed: {e}")))) }
                    probing.set(false); });
            }, if *probing.read() { "Checking with IBKR…" } else { "Check IBKR permissions" } }
            span { class: "muted small", "{probe_txt}" }
        }
        for b in blocked { p { class: "muted xs", "✗ {b}" } }
        div { class: "sec-row",
            for (id, name) in rows {
                {
                    let on = mt.get(id).copied().unwrap_or(true);
                    rsx! { label { class: "chk", input { r#type: "checkbox", checked: on, oninput: move |e| {
                        let v = e.checked();
                        spawn(async move { match set_market_tradable(id.to_string(), v).await {
                            Ok(_) => { if let Ok(r) = get_risk().await { st.risk.set(Some(r)); } st.notify("info", format!("{name} trading {}", if v { "enabled" } else { "disabled (view-only)" })); }
                            Err(e) => st.toast.set(Some((false, format!("Failed: {e}")))) } });
                    } } span { "{name}" if id == "CRYPTO" && !on { span { class: "tag peach xs", style: "margin-left:6px", "view-only · IBKR regulatory restriction" } } } } }
                }
            }
        }
        p { class: "muted xs", "Permissions are checked automatically with a what-if order each time an account is selected: markets IBKR allows are enabled, markets it rejects (e.g. crypto under Regulatory Restrictions) become view-only. A manual tick here overrides the automatic result until you re-check." }
    }
}

#[component]
fn AssistantSettings() -> Element {
    let mut st = app_state();
    let a = st.risk.read().as_ref().map(|r| r.assistant.clone()).unwrap_or_default();
    let mut notional = use_signal(String::new);
    let mut per_hour = use_signal(String::new);
    let mut msg = use_signal(String::new);
    let mut seeded = use_signal(|| false);
    use_effect(move || {
        let r = st.risk.read().clone();
        if let Some(r) = r { if !*seeded.peek() && r.assistant.max_order_notional_usd > 0.0 { notional.set(format!("{}", r.assistant.max_order_notional_usd)); per_hour.set(format!("{}", r.assistant.max_orders_per_hour)); seeded.set(true); } }
    });
    let mut apply = move |w: Option<bool>, live: Option<bool>, short: Option<bool>, agents: Option<bool>, lim: bool| {
        let n: Option<f64> = if lim { notional.peek().replace(',', "").parse().ok() } else { None };
        let h: Option<i64> = if lim { per_hour.peek().parse().ok() } else { None };
        spawn(async move { match put_assistant(w, live, n, h, short, agents).await {
            Ok(_) => { if let Ok(r) = get_risk().await { st.risk.set(Some(r)); } msg.set("Assistant settings saved".into()); }
            Err(e) => msg.set(format!("Failed: {e}")) } });
    };
    rsx! {
        div { class: "sec-row",
            label { class: "chk", input { r#type: "checkbox", checked: a.write_enabled, oninput: move |e| apply(Some(e.checked()), None, None, None, false) } span { b { "Allow the assistant to trade" } " (off = read-only)" } }
            label { class: "chk", input { r#type: "checkbox", checked: a.live, oninput: move |e| apply(None, Some(e.checked()), None, None, false) } span { "…also on live accounts" } }
            label { class: "chk", input { r#type: "checkbox", checked: a.allow_short, oninput: move |e| apply(None, None, Some(e.checked()), None, false) } span { "Allow short" } }
        }
        div { class: "sec-row",
            span { class: "muted small", "Max notional per order (USD)" } input { class: "num short wide", value: "{notional}", oninput: move |e| notional.set(e.value()) }
            span { class: "muted small", "Max orders / hour" } input { class: "num short", value: "{per_hour}", oninput: move |e| per_hour.set(e.value()) }
            button { class: "mini", onclick: move |_| apply(None, None, None, None, true), "Save limits" }
            span { class: "muted small", "{a.orders_last_hour} orders in the last hour" }
            if !a.operator_exists { span { class: "tag peach xs", "create an operator with role 'assistant' below and put its PIN in bot/.env" } }
        }
        p { class: "muted xs", "Every assistant order is two-step (prepare → confirm) and logged with source and request id. Setup: see bot/README.md → MCP." }
        if !msg.read().is_empty() { div { class: "notice", "{msg}" } }
    }
}

// ---------------- audit panel ----------------
#[component]
pub fn AuditPanel() -> Element {
    let mut items = use_signal(Vec::<AuditItem>::new);
    let mut loading = use_signal(|| false);
    use_future(move || async move {
        loop {
            loading.set(true);
            if let Ok(a) = get_audit(150).await { items.set(a); }
            loading.set(false);
            #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
            crate::sleep_ms(5000).await;
            #[cfg(not(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web")))]
            break;
        }
    });
    let rows = items.read().clone();
    rsx! {
        if rows.is_empty() { p { class: "empty", if *loading.read() { "Loading audit log..." } else { "No audit events yet" } } } else {
            table { class: "grid-tbl audit",
                thead { tr { th { "Time (UTC)" } th { "Event" } th { "Details" } } }
                tbody {
                    for it in rows {
                        {
                            let details = it.fields.iter().filter(|(_, v)| !v.is_null())
                                .map(|(k, v)| format!("{k}={}", v.as_str().map(String::from).unwrap_or_else(|| v.to_string())))
                                .collect::<Vec<_>>().join("  ");
                            let cls = match it.event.as_str() {
                                "order" => "ev order", "order_rejected" => "ev rejected", "kill_switch" | "halt" => "ev kill",
                                "agent_start" | "agent_stop" | "agent_config" => "ev agent", _ => "ev",
                            };
                            rsx! {
                                tr {
                                    td { class: "num muted", "{it.ts.replace('T', \" \").chars().take(19).collect::<String>()}" }
                                    td { span { class: "{cls}", "{it.event}" } }
                                    td { class: "num small details", "{details}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


// ---------------- window fullscreen (native window on desktop, Fullscreen API in the web demo) ----------------
/// JS that toggles browser fullscreen for the whole page (web demo only)
pub const WEB_FULLSCREEN_JS: &str = "if (document.fullscreenElement) { document.exitFullscreen().catch(() => {}); } else if (document.documentElement.requestFullscreen) { document.documentElement.requestFullscreen({ navigationUI: 'hide' }).catch(() => {}); }";

#[component]
pub fn FullscreenButton() -> Element {
    let mut is_full = use_signal(|| false);
    #[cfg(not(feature = "desktop"))]
    use_future(move || async move {
        let mut ev = document::eval("dioxus.send(!!document.fullscreenElement); document.addEventListener('fullscreenchange', () => dioxus.send(!!document.fullscreenElement)); await new Promise(() => {});");
        while let Ok(v) = ev.recv::<bool>().await { is_full.set(v); }
    });
    let toggle = move |_: Event<MouseData>| {
        #[cfg(feature = "desktop")]
        {
            let next = !*is_full.peek();
            dioxus::desktop::window().set_fullscreen(next);
            is_full.set(next);
        }
        #[cfg(not(feature = "desktop"))]
        { let _ = document::eval(WEB_FULLSCREEN_JS); }
    };
    rsx! {
        button { class: "tool win", title: if *is_full.read() { "Exit full screen" } else { "Full screen" }, onclick: toggle,
            if *is_full.read() { "⤡" } else { "⤢" }
        }
    }
}


// ---------------- performance panel ----------------
fn dur_fmt(sec: f64) -> String {
    let s = sec.max(0.0) as i64;
    if s < 60 { format!("{s}s") } else if s < 3600 { format!("{}m", s / 60) } else if s < 86400 { format!("{}h {}m", s / 3600, (s % 3600) / 60) } else { format!("{}d {}h", s / 86400, (s % 86400) / 3600) }
}

#[component]
pub fn PerformancePanel() -> Element {
    let mut st = app_state();
    let mut range = use_signal(|| "all".to_string());
    let mut stats = use_signal(|| None::<Stats>);
    let mut equity = use_signal(Vec::<EquityPoint>::new);
    let mut trades = use_signal(Vec::<Trade>::new);
    let mut src_filter = use_signal(String::new);
    let mut known_src = use_signal(Vec::<String>::new);
    let mut tick = use_signal(|| 0u32);
    let mut syncing = use_signal(|| false);
    let mut full = st.perf_full;

    use_effect(move || {
        let r = range.read().clone();
        let src = src_filter.read().clone();
        let _ = tick.read();
        let _ = st.hist_scope.read();
        let tz = st.tz.read().clone();
        spawn(async move {
            let sc = st.hist_scope.peek().clone();
            if let Ok(s) = get_stats(r.clone(), sc.clone(), src.clone(), tz.clone()).await {
                if src.is_empty() { known_src.set(s.by_source.iter().map(|g| g.name.clone()).filter(|n| n.starts_with("node:") || n.starts_with("agent:")).collect()); }
                stats.set(Some(s));
            }
            if let Ok(e) = get_equity(r.clone(), sc.clone(), tz.clone()).await { equity.set(e); }
            if let Ok(t) = get_trades(r, String::new(), src, sc, tz).await { trades.set(t); }
        });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop { crate::sleep_ms(30000).await; tick += 1; }
    });

    let s = stats.read().clone().unwrap_or_default();
    let eq = equity.read().clone();
    let eq_partial = equity_partial(&eq) || s.equity_partial == Some(true);
    let tz_tip = { let z = st.tz.read().clone(); if z.is_empty() { "Since 00:00 UTC".to_string() } else { format!("Since local midnight ({z})") } };
    let cur_range = range.read().clone();

    let mut export = move |what: &'static str| {
        let r = range.read().clone();
        spawn(async move {
            match get_export(what.to_string(), r.clone(), st.hist_scope.peek().clone(), st.tz.peek().clone()).await {
                Ok(csv) => {
                    #[cfg(feature = "desktop")]
                    {
                        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                        let path = format!("{home}/Downloads/qos_{what}_{r}.csv");
                        match std::fs::write(&path, csv) {
                            Ok(_) => st.toast.set(Some((true, format!("Saved {path}")))),
                            Err(e) => st.toast.set(Some((false, format!("Save failed: {e}")))),
                        }
                    }
                    #[cfg(not(feature = "desktop"))]
                    { let _ = csv; st.toast.set(Some((true, "Export ready".into()))); }
                }
                Err(e) => st.toast.set(Some((false, format!("Export failed: {e}")))),
            }
        });
    };

    rsx! {
        div { class: if *full.read() { "perf fullscreen-panel" } else { "perf" },
            div { class: "perf-bar",
                div { class: "tfs",
                    for (id, lbl) in [("today", "Today"), ("week", "7D"), ("month", "30D"), ("all", "All")] {
                        button { class: if cur_range == id { "tf active" } else { "tf" }, title: if id == "today" { tz_tip.clone() } else if id == "all" { String::new() } else { format!("Rolling {} — the last {} × 24 h up to now", lbl, if id == "week" { 7 } else { 30 }) }, onclick: move |_| range.set(id.to_string()), "{lbl}" }
                    }
                }
                Dropdown { class: "wl-market", value: "{src_filter}", onchange: move |v| src_filter.set(v),
                    options: dd_opts([("", "All sources"), ("manual", "Manual"), ("agent:", "All agents"), ("node:", "All nodes")]).into_iter()
                        .chain(known_src.read().iter().map(|n| (n.clone(), if n.starts_with("node:") { format!("Node · {}", source_label(n)) } else { format!("Agent · {}", source_label(n)) }))).collect::<DdOpts>() }
                Dropdown { class: "wl-market", value: "{st.hist_scope}", title: "Account scope", onchange: move |v| st.hist_scope.set(v),
                    options: dd_opts([("", "This account"), ("ALL", "All accounts")]) }
                div { class: "tabs-right",
                    button { class: "mini", onclick: move |_| export("trades"), "Export trades" }
                    button { class: "mini", onclick: move |_| export("equity"), "Export equity" }
                    button { class: "mini accent", disabled: *syncing.read(), onclick: move |_| {
                        syncing.set(true);
                        spawn(async move {
                            match sync_databricks().await {
                                Ok(v) => st.toast.set(Some((true, format!("Databricks sync: {}", v.get("synced").map(|x| x.to_string()).unwrap_or_default())))),
                                Err(e) => st.toast.set(Some((false, format!("Sync failed: {e}")))),
                            }
                            syncing.set(false);
                        });
                    }, if *syncing.read() { "Syncing..." } else { "Sync to Databricks" } }
                    button { class: "tool", title: if *full.read() { "Close" } else { "Expand" }, onclick: move |_| full.toggle(), if *full.read() { "✕" } else { "⛶" } }
                }
            }
            div { class: "perf-cards",
                StatCard { label: "Net P&L".to_string(), value: if s.unknown_pnl_trades > 0 { format!("{} (+{} trade(s) without USD P&L)", money(s.net_pnl), s.unknown_pnl_trades) } else { money(s.net_pnl) }, cls: (pnl_class(s.net_pnl)).to_string() }
                StatCard { label: "Trades".to_string(), value: format!("{} ({} open) · {}W {}L{}", s.trades, s.open_trades, s.wins, s.losses, if s.breakeven > 0 { format!(" {}BE", s.breakeven) } else { String::new() }), cls: ("").to_string() }
                StatCard { label: "Win rate".to_string(), value: format!("{:.1}%", s.win_rate), cls: (if s.win_rate >= 50.0 { "text-up" } else { "text-down" }).to_string() }
                StatCard { label: "Profit factor".to_string(), value: if s.profit_factor >= 999.0 { "∞".into() } else { format!("{:.2}", s.profit_factor) }, cls: (if s.profit_factor >= 1.0 { "text-up" } else { "text-down" }).to_string() }
                StatCard { label: "Expectancy / trade".to_string(), value: money(s.expectancy), cls: (pnl_class(s.expectancy)).to_string() }
                StatCard { label: "Avg win / loss".to_string(), value: format!("{} / {}", money(s.avg_win), money(s.avg_loss)), cls: ("").to_string() }
                StatCard { label: if eq_partial { "Max drawdown · PARTIAL".to_string() } else { "Max drawdown".to_string() }, value: format!("{} ({:.2}%)", money(s.max_drawdown), s.max_drawdown_pct), cls: ("text-down").to_string() }
                StatCard { label: if eq_partial { "Sharpe (daily) · PARTIAL".to_string() } else { "Sharpe (daily)".to_string() }, value: format!("{:.2}", s.sharpe), cls: (if s.sharpe >= 1.0 { "text-up" } else { "" }).to_string() }
                StatCard { label: "Avg holding".to_string(), value: dur_fmt(s.avg_holding_sec), cls: ("").to_string() }
                StatCard { label: "Commission".to_string(), value: money(s.commission), cls: ("text-down").to_string() }
            }
            div { class: "perf-main",
                div { class: "perf-chart",
                    h4 { "Equity curve" span { class: "muted small", " {money(s.equity_start)} → {money(s.equity_end)}" }
                        if eq_partial { span { class: "tag peach xs", style: "margin-left:6px", title: "{PARTIAL_TIP}", "PARTIAL" } } }
                    EquityChart { points: eq }
                }
                div { class: "perf-groups",
                    GroupTable { title: "By instrument".to_string(), rows: s.by_symbol.clone() }
                    GroupTable { title: "By source".to_string(), rows: s.by_source.clone(), sources: true }
                    GroupTable { title: "By weekday".to_string(), rows: s.by_weekday.clone() }
                }
            }
            h4 { class: "perf-h", "Trades" span { class: "muted small", " {trades.read().len()} shown · click a row to add a note" } }
            TradesTable { trades: trades.read().clone(), on_change: move |_| tick += 1 }
        }
    }
}

#[component]
fn StatCard(label: String, value: String, cls: String) -> Element {
    rsx! { div { class: "scard", div { class: "lbl", "{label}" } div { class: "val num {cls}", "{value}" } } }
}

#[component]
fn GroupTable(title: String, rows: Vec<GroupStat>, #[props(default)] sources: bool) -> Element {
    rsx! {
        div { class: "gtable",
            h4 { "{title}" }
            if rows.is_empty() { p { class: "empty small", "—" } } else {
                table { class: "grid-tbl compact",
                    thead { tr { th { "" } th { class: "r", "N" } th { class: "r", "Win%" } th { class: "r", "Net" } } }
                    tbody { for r in rows { tr {
                        td { class: "sym", if sources { SourceTag { source: r.name.clone() } } else { "{r.name}" } } td { class: "r num", "{r.trades}" }
                        td { class: "r num", "{r.win_rate:.0}%" } td { class: "r num {pnl_class(r.net_pnl)}", "{money(r.net_pnl)}" }
                    } } }
                }
            }
        }
    }
}

#[component]
fn EquityChart(points: Vec<EquityPoint>) -> Element {
    const W: f64 = 900.0; const H: f64 = 220.0; const PL: f64 = 8.0; const PR: f64 = 70.0; const PT: f64 = 10.0; const PB: f64 = 50.0; const DDH: f64 = 36.0;
    if points.len() < 2 { return rsx! { div { class: "chart-empty small", "Equity history builds up while the API is running (one point per minute)" } }; }
    let vals: Vec<f64> = points.iter().map(|p| p.netliq).collect();
    let (lo, hi) = vals.iter().fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    let range = if hi > lo { hi - lo } else { 1.0 };
    let n = vals.len() as f64;
    let x = |i: usize| PL + i as f64 / (n - 1.0) * (W - PL - PR);
    let y = |v: f64| PT + (hi - v) / range * (H - PT - PB);
    let line = vals.iter().enumerate().map(|(i, v)| format!("{}{:.1},{:.1}", if i == 0 { "M" } else { "L" }, x(i), y(*v))).collect::<Vec<_>>().join(" ");
    let area = format!("{line} L{:.1},{:.1} L{:.1},{:.1} Z", x(vals.len() - 1), H - PB, x(0), H - PB);
    // drawdown
    let mut peak = f64::MIN; let mut dd: Vec<f64> = Vec::new();
    for v in &vals { peak = peak.max(*v); dd.push(if peak > 0.0 { (peak - v) / peak * 100.0 } else { 0.0 }); }
    let max_dd = dd.iter().cloned().fold(0.0, f64::max).max(0.01);
    let ydd = |d: f64| H - PB + 6.0 + d / max_dd * (DDH - 6.0);
    let dd_path = dd.iter().enumerate().map(|(i, d)| format!("{}{:.1},{:.1}", if i == 0 { "M" } else { "L" }, x(i), ydd(*d))).collect::<Vec<_>>().join(" ")
        + &format!(" L{:.1},{:.1} L{:.1},{:.1} Z", x(vals.len() - 1), H - PB + 6.0, x(0), H - PB + 6.0);
    let up = vals[vals.len() - 1] >= vals[0];
    let ticks: Vec<(f64, String)> = (0..=3).map(|i| { let v = hi - range * i as f64 / 3.0; (y(v), money(v)) }).collect();
    let first_t = points[0].ts.replace('T', " ").chars().take(16).collect::<String>();
    let last_t = points[points.len() - 1].ts.replace('T', " ").chars().take(16).collect::<String>();
    rsx! {
        svg { class: "eq-svg", view_box: "0 0 {W} {H}", preserve_aspect_ratio: "none",
            for (yy, lab) in ticks { line { x1: "{PL}", x2: "{W - PR}", y1: "{yy:.1}", y2: "{yy:.1}", class: "grid" } text { x: "{W - PR + 6.0}", y: "{yy + 4.0:.1}", class: "ylab", "{lab}" } }
            path { d: "{area}", class: if up { "eq-area up" } else { "eq-area down" } }
            path { d: "{line}", class: if up { "eq-line up" } else { "eq-line down" } }
            path { d: "{dd_path}", class: "dd-area" }
            text { x: "{W - PR + 6.0}", y: "{H - PB + DDH - 2.0}", class: "ylab", "DD {max_dd:.2}%" }
            text { x: "{PL}", y: "{H - 4.0}", class: "xlab left", "{first_t}" }
            text { x: "{W - PR}", y: "{H - 4.0}", class: "xlab right", "{last_t}" }
        }
    }
}

#[component]
fn TradesTable(trades: Vec<Trade>, on_change: EventHandler<()>) -> Element {
    rsx! {
        if trades.is_empty() { p { class: "empty", "No trades in this range" } } else {
            table { class: "grid-tbl",
                thead { tr { th { "Entry (UTC)" } th { "Symbol" } th { "Side" } th { "Source" } th { class: "r", "Qty" } th { class: "r", "Entry" } th { class: "r", "Exit" } th { class: "r", "Hold" } th { class: "r", "Net P&L" } th { "Journal" } } }
                tbody { for t in trades { TradeRow { trade: t, on_change } } }
            }
        }
    }
}

#[component]
fn TradeRow(trade: Trade, on_change: EventHandler<()>) -> Element {
    let mut st = app_state();
    let mut editing = use_signal(|| false);
    let mut note = use_signal(|| trade.note.clone());
    let mut tags = use_signal(|| trade.tags.clone());
    let mut rating = use_signal(|| trade.rating);
    let fe = trade.first_exec.clone();
    let dec = decimals_for_key(&trade.key);
    let entry = trade.entry_ts.replace('T', " ").chars().take(16).collect::<String>();
    let exit_px = trade.exit_price.map(|v| format!("{:.*}", dec, v)).unwrap_or("open".into());
    let entry_px = trade.entry_price.map(|v| px(v, dec)).unwrap_or_else(|| "—".into());
    let net_txt = trade.net_pnl.map(money).unwrap_or_else(|| "—".into());
    let net_tip = if trade.net_pnl.is_none() && trade.open != 1 { "P&L could not be converted to USD (no rate for this cross pair)" } else { "" };
    let stars: String = (1..=5).map(|i| if i <= *rating.read() { '★' } else { '☆' }).collect();
    let save = move |_: Event<MouseData>| {
        let fe = fe.clone();
        let (n, tg, r) = (note.read().clone(), tags.read().clone(), *rating.read());
        editing.set(false);
        spawn(async move {
            match set_journal(fe, n, tg, r).await {
                Ok(_) => { st.toast.set(Some((true, "Journal saved".into()))); on_change.call(()); }
                Err(e) => st.toast.set(Some((false, format!("Journal failed: {e}")))),
            }
        });
    };
    rsx! {
        tr { class: if trade.open == 1 { "open-trade" } else { "" }, onclick: move |_| if !*editing.read() { editing.set(true) },
            td { class: "num muted", "{entry}" }
            td { class: "sym", "{trade.symbol}" }
            td { class: if trade.side == "LONG" { "text-up" } else { "text-down" }, "{trade.side}" }
            td { class: "src-cell", SourceTag { source: trade.source.clone(), strategy: trade.strategy.clone() } }
            td { class: "r num", "{qty_fmt(trade.qty)}" }
            td { class: "r num", "{entry_px}" }
            td { class: "r num", "{exit_px}" }
            td { class: "r num muted", "{trade.holding_sec.map(dur_fmt).unwrap_or_default()}" }
            td { class: "r num {pnl_class(trade.net_pnl.unwrap_or(0.0))}", title: "{net_tip}", if trade.open == 1 { "open" } else { "{net_txt}" } }
            td { class: "journal",
                if *editing.read() {
                    div { class: "jform", onclick: move |e| e.stop_propagation(),
                        div { class: "stars",
                            for i in 1..=5i64 { span { class: if i <= *rating.read() { "star on" } else { "star" }, onclick: move |_| rating.set(i), "★" } }
                        }
                        input { class: "inline wide", placeholder: "tags, comma separated", value: "{tags}", oninput: move |e| tags.set(e.value()) }
                        input { class: "inline wide", placeholder: "note", value: "{note}", oninput: move |e| note.set(e.value()) }
                        button { class: "mini accent", onclick: save, "Save" }
                        button { class: "mini", onclick: move |_| editing.set(false), "Cancel" }
                    }
                } else {
                    span { class: "muted small", "{stars} {tags} {note}" }
                }
            }
        }
    }
}


// ---------------- notification settings (in Risk tab) ----------------
#[component]
pub fn NotifySettings() -> Element {
    let mut st = app_state();
    let mut cfg = use_signal(|| None::<NotifyCfg>);
    use_effect(move || { spawn(async move { if let Ok(c) = get_notify().await { cfg.set(Some(c)); } }); });
    let c = cfg.read().clone().unwrap_or_default();
    let mut save = move |next: NotifyCfg| {
        spawn(async move {
            match put_notify(next).await { Ok(c2) => cfg.set(Some(c2)), Err(e) => st.toast.set(Some((false, format!("Notify: {e}")))) }
        });
    };
    let events = [("order", "Agent orders"), ("fill", "Fills"), ("agent_stop", "Agent start / stop"), ("error", "Errors & rejections"), ("halt", "Trading halted"), ("kill", "Kill switch"), ("daily", "Daily summary")];
    rsx! {
        div { class: "risk-col",
            h4 { "Telegram alerts" }
            if !c.configured { div { class: "notice", "Add TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID to bot/.env and restart the API to enable alerts." } }
            label { class: "chk", input { r#type: "checkbox", checked: c.enabled, oninput: move |e| { let mut n = cfg.peek().clone().unwrap_or_default(); n.enabled = e.checked(); save(n); } } span { "Enable alerts" } }
            for (k, lbl) in events {
                label { class: "chk", input { r#type: "checkbox", checked: *c.events.get(k).unwrap_or(&true), oninput: move |e| { let mut n = cfg.peek().clone().unwrap_or_default(); n.events.insert(k.to_string(), e.checked()); save(n); } } span { "{lbl}" } }
            }
            button { class: "mini accent", onclick: move |_| { spawn(async move {
                match notify_test().await { Ok(_) => st.toast.set(Some((true, "Test alert sent".into()))), Err(e) => st.toast.set(Some((false, format!("{e}")))) }
            }); }, "Send test" }
        }
    }
}


// ---------------- login / account setup (Interactive Brokers only) ----------------
#[component]
pub fn LoginScreen() -> Element {
    let mut st = app_state();
    let status = st.session.read().clone().unwrap_or_default();
    let op = st.op.read().clone();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut mode = use_signal(|| "paper".to_string());
    let mut live_code = use_signal(String::new);
    let mut chosen = use_signal(String::new);
    let mut err = use_signal(String::new);
    let mut confirm_live = use_signal(|| false);
    let mut show_new = use_signal(|| false);
    // operator step
    let mut op_id = use_signal(String::new);
    let mut op_pin = use_signal(String::new);
    let mut boot_name = use_signal(String::new);
    let mut boot_pin = use_signal(String::new);
    let mut skip_ops = use_signal(|| false);
    let busy = *st.login_busy.read();
    let starting = status.setup.state == "starting";
    let api_down = st.api_error.read().is_some() && st.session.read().is_none();
    use_effect(move || {
        let sess = st.session.read().clone();
        if let Some(sess) = sess {
            if !sess.accounts.is_empty() && chosen.peek().is_empty() {
                let pre = if !sess.selected_account.is_empty() { sess.selected_account.clone() } else { sess.accounts[0].account_id.clone() };
                chosen.set(pre);
            }
        }
    });
    // steps: 0 = operator, 1 = gateway/IBKR login, 2 = choose account
    let op_ok = match &op { Some(o) => o.operator.is_some() || (!o.configured && *skip_ops.read()), None => false };
    let step = if !op_ok { 0 } else if status.ib_connected && !status.accounts.is_empty() && !starting { 2 } else { 1 };
    let is_live = *mode.read() == "live";
    let has_gateways = !status.gateways.is_empty();

    let mut do_op_login = move |_: ()| {
        let (id, pin) = (op_id.read().trim().to_string(), op_pin.read().clone());
        if id.is_empty() || pin.is_empty() { err.set("Choose your operator and enter your PIN".into()); return; }
        st.login_busy.set(true);
        spawn(async move {
            match op_login(id, pin).await {
                Ok(o) => { op_pin.set(String::new()); err.set(String::new()); st.op.set(Some(OpStatus { configured: true, operator: Some(o), ..Default::default() }));
                    spawn(reconnect_live_stream()); }   // web: the live stream needs the new session
                Err(e) => err.set(format!("{e}")),
            }
            st.login_busy.set(false);
        });
    };
    let do_bootstrap = move |_| {
        let (n, pin) = (boot_name.read().trim().to_string(), boot_pin.read().clone());
        if n.is_empty() || pin.len() < 4 { err.set("Enter a name and a PIN of at least 4 characters".into()); return; }
        st.login_busy.set(true);
        spawn(async move {
            match op_bootstrap(n, pin).await {
                Ok(o) => { err.set(String::new()); st.op.set(Some(OpStatus { configured: true, operator: Some(o), ..Default::default() }));
                    spawn(reconnect_live_stream()); }
                Err(e) => err.set(format!("{e}")),
            }
            st.login_busy.set(false);
        });
    };
    let mut do_login = move |_: ()| {
        let (u, p, m, lc) = (username.read().trim().to_string(), password.read().clone(), mode.read().clone(), live_code.read().clone());
        if u.is_empty() || p.is_empty() { err.set("Enter your Interactive Brokers username and password".into()); return; }
        if m == "live" && !*confirm_live.read() { err.set("Tick the confirmation box to sign in to a LIVE account".into()); return; }
        if m == "live" && lc.is_empty() { err.set("Enter the live confirmation code".into()); return; }
        err.set(String::new());
        st.login_busy.set(true);
        spawn(async move {
            match setup_login(u, p, m, lc).await {
                Ok(_) => { password.set(String::new()); live_code.set(String::new()); show_new.set(false); }
                Err(e) => err.set(format!("Login failed: {e}")),
            }
            st.login_busy.set(false);
        });
    };
    let do_enter = move |_| {
        let acct = chosen.read().clone();
        if acct.is_empty() { return; }
        st.login_busy.set(true);
        spawn(async move {
            match setup_account(acct.clone()).await {
                Ok(_) => { if let Ok(s) = get_setup_status().await { st.session.set(Some(s)); } st.notify("info", format!("Signed in · account {acct}")); st.refresh_tick += 1; }
                Err(e) => err.set(format!("Could not select account: {e}")),
            }
            st.login_busy.set(false);
        });
    };
    let gateways = status.gateways.clone();
    let operators = op.as_ref().map(|o| o.operators.clone()).unwrap_or_default();
    let configured_ops = op.as_ref().map(|o| o.configured).unwrap_or(false);
    rsx! {
        div { class: "login-bg",
            div { class: "login-card",
                div { class: "login-brand qos", img { class: "logo-img big", src: LOGO, alt: "QOS" } span { class: "qos-tag", "Quant Operating System" } }
                div { class: "login-broker", span { class: "tag sky", "Interactive Brokers" } span { class: "muted small", "Only IBKR usernames and accounts are supported in this version." } }
                if api_down {
                    div { class: "notice", "Cannot reach the engine API (127.0.0.1:8000). If the engine runs on a server, open the SSH tunnel first (deploy/tunnel.sh in the bot folder) and keep it running." }
                }
                div { class: "login-steps",
                    span { class: if step == 0 { "step active" } else { "step done" }, "1 · Operator" }
                    span { class: if step == 1 { "step active" } else if step > 1 { "step done" } else { "step" }, "2 · IBKR gateway" }
                    span { class: if step == 2 { "step active" } else { "step" }, "3 · Account" }
                }

                // ---------- step 0: operator ----------
                if step == 0 {
                    if op.is_none() {
                        p { class: "muted small", "Connecting to the engine…" }       // status not loaded yet: never offer to create an admin
                    } else if configured_ops {
                        div { class: "fld", span { "Operator" }
                            Dropdown { class: "wl-market", value: "{op_id}", onchange: move |v| op_id.set(v),
                                options: std::iter::once((String::new(), "— choose —".to_string())).chain(operators.iter().map(|o| (o.id.to_string(), format!("{} ({})", o.name, o.role)))).collect::<DdOpts>() } }
                        label { class: "fld", span { "PIN" }
                            input { r#type: "password", value: "{op_pin}", disabled: busy, oninput: move |e| op_pin.set(e.value()), onkeydown: move |e| { if e.key() == Key::Enter { do_op_login(()); } } } }
                        if !err.read().is_empty() { div { class: "notice bad", "{err}" } }
                        div { class: "btn-row", button { class: "mini primary", disabled: busy, onclick: move |_| do_op_login(()), "Unlock" } }
                    } else {
                        p { class: "muted small", "No operators are set up yet. Create the first admin so every order and setting change is attributed to a person — or continue as the local admin on this computer." }
                        label { class: "fld", span { "Admin name" } input { value: "{boot_name}", oninput: move |e| boot_name.set(e.value()) } }
                        label { class: "fld", span { "PIN (min 4)" } input { r#type: "password", value: "{boot_pin}", oninput: move |e| boot_pin.set(e.value()) } }
                        if !err.read().is_empty() { div { class: "notice bad", "{err}" } }
                        div { class: "btn-row",
                            button { class: "mini", onclick: move |_| skip_ops.set(true), "Continue as local admin" }
                            button { class: "mini primary", disabled: busy, onclick: do_bootstrap, "Create admin & continue" }
                        }
                    }
                }

                // ---------- step 1: gateway ----------
                if step == 1 {
                    if has_gateways && !*show_new.read() {
                        div { class: "acct-list",
                            for g in gateways {
                                {
                                    let gid = g.id.clone();
                                    let gid2 = g.id.clone();
                                    let live = g.mode == "live";
                                    rsx! { div { class: if g.active { "acct-row sel gw" } else { "acct-row gw" },
                                        span { class: "acct-id", "{g.username_masked}" }
                                        span { class: if live { "tag rose" } else { "tag mint" }, "{g.mode}" }
                                        span { class: if g.running { "tag sky xs" } else { "tag xs" }, if g.running { "running" } else { "stopped" } }
                                        span { class: "muted small num", "port {g.api_port}" }
                                        button { class: if live { "mini danger" } else { "mini primary" }, disabled: busy || starting, onclick: move |_| {
                                            let gid = gid.clone(); let lc = live_code.peek().clone();
                                            if live && lc.is_empty() { err.set("Enter the live confirmation code below first".into()); return; }
                                            st.login_busy.set(true);
                                            spawn(async move { match setup_connect(gid, lc).await { Ok(_) => err.set(String::new()), Err(e) => err.set(format!("{e}")) } st.login_busy.set(false); });
                                        }, if g.running { "Connect" } else { "Start & connect" } }
                                        if g.running {
                                            button { class: "mini", title: "Stop this gateway container", onclick: move |_| { let gid = gid2.clone(); spawn(async move { let _ = gateway_stop(gid).await; }); }, "Stop" }
                                        }
                                    } }
                                }
                            }
                        }
                        if gateways_need_live_code(&status) {
                            label { class: "fld", span { "Live confirmation code (for live gateways)" }
                                input { r#type: "password", value: "{live_code}", oninput: move |e| live_code.set(e.value()) } }
                        }
                        p { class: "muted xs", "Paper and live gateways can run side by side — switching between them does not restart anything. A stopped gateway whose password was cleared needs a fresh sign-in." }
                        if !err.read().is_empty() { div { class: "notice bad", "{err}" } }
                        if starting { div { class: "progress", div { class: "spinner" } span { "{status.setup.message}" } } }
                        else if status.setup.state == "error" { div { class: "notice bad", "{status.setup.message}" } }
                        div { class: "btn-row", button { class: "mini", onclick: move |_| show_new.set(true), "+ Add another IBKR login" } }
                    } else {
                        div { class: "mode login-mode",
                            button { class: if !is_live { "mode-btn active" } else { "mode-btn" }, onclick: move |_| { mode.set("paper".into()); confirm_live.set(false); }, "Paper trading" }
                            button { class: if is_live { "mode-btn active live" } else { "mode-btn" }, onclick: move |_| mode.set("live".into()), "Live trading" }
                        }
                        if is_live && !status.live_allowed {
                            div { class: "notice bad", "Live trading is disabled. An admin must enable 'Allow live trading' in the Risk tab → Security first." }
                        }
                        label { class: "fld", span { "IBKR username" }
                            input { value: "{username}", placeholder: "e.g. jsmith123", autocomplete: "username", disabled: busy || starting, oninput: move |e| username.set(e.value()) } }
                        label { class: "fld", span { "Password" }
                            input { r#type: "password", value: "{password}", autocomplete: "current-password", disabled: busy || starting, oninput: move |e| password.set(e.value()),
                                onkeydown: move |e| { if e.key() == Key::Enter { do_login(()); } } } }
                        if is_live {
                            label { class: "fld", span { "Live confirmation code" } input { r#type: "password", value: "{live_code}", oninput: move |e| live_code.set(e.value()) } }
                            label { class: "chk live-warn", input { r#type: "checkbox", checked: *confirm_live.read(), oninput: move |e| confirm_live.set(e.checked()) }
                                span { "I understand this signs in to a LIVE account and orders will use real money." } }
                        }
                        p { class: "muted small", "Credentials are written only to this gateway's configuration on this computer (one container per IBKR login and mode). If IBKR prompts for two-factor authentication, approve it in the IBKR Mobile app." }
                        if !err.read().is_empty() { div { class: "notice bad", "{err}" } }
                        if starting { div { class: "progress", div { class: "spinner" } span { "{status.setup.message}" } } }
                        else if status.setup.state == "error" { div { class: "notice bad", "{status.setup.message}" } }
                        div { class: "btn-row",
                            if has_gateways { button { class: "mini", onclick: move |_| show_new.set(false), "Back" } }
                            button { class: if is_live { "mini danger" } else { "mini primary" }, disabled: busy || starting || (is_live && !status.live_allowed), onclick: move |_| do_login(()),
                                if starting { "Starting gateway…" } else if is_live { "Sign in (LIVE)" } else { "Sign in (Paper)" } }
                        }
                    }
                }

                // ---------- step 2: account ----------
                if step == 2 {
                    div { class: "acct-list",
                        for a in status.accounts.clone() {
                            {
                                let id = a.account_id.clone();
                                let id2 = a.account_id.clone();
                                let sel = *chosen.read() == a.account_id;
                                let kind = if a.account_id.starts_with('D') { "Paper" } else { "Live" };
                                rsx! {
                                    button { class: if sel { "acct-row sel" } else { "acct-row" }, onclick: move |_| chosen.set(id.clone()),
                                        span { class: "acct-id num", "{id2}" }
                                        span { class: if kind == "Paper" { "tag mint" } else { "tag rose" }, "{kind}" }
                                        span { class: "acct-nl num", "{money(a.net_liquidation)} {a.currency}" }
                                    }
                                }
                            }
                        }
                    }
                    if !err.read().is_empty() { div { class: "notice bad", "{err}" } }
                    div { class: "btn-row",
                        button { class: "mini", disabled: busy, onclick: move |_| {
                            st.login_busy.set(true);
                            spawn(async move { let _ = setup_logout(false).await; if let Ok(s) = get_setup_status().await { st.session.set(Some(s)); } st.login_busy.set(false); });
                        }, "Other gateway" }
                        button { class: "mini primary", disabled: busy || chosen.read().is_empty(), onclick: do_enter, "Enter terminal" }
                    }
                }
            }
        }
    }
}

fn gateways_need_live_code(s: &SetupStatus) -> bool { s.gateways.iter().any(|g| g.mode == "live") }

/// AI assistant (MCP) status: read-only / trading — admin can toggle it
#[component]
fn AssistantChip() -> Element {
    let mut st = app_state();
    let a = st.risk.read().as_ref().map(|r| r.assistant.clone()).unwrap_or_default();
    let admin = st.is_admin();
    let (cls, label, state) = if a.write_enabled { if a.live { ("asst-chip on live", "AI LIVE", "trading (LIVE)") } else { ("asst-chip on", "AI", "trading (paper)") } } else { ("asst-chip", "AI", "read-only") };
    let title = format!("AI assistant (MCP): {state} · {} orders in the last hour · limit {} USD/order{}", a.orders_last_hour, a.max_order_notional_usd, if admin { " · click to toggle" } else { "" });
    let on_now = a.write_enabled;
    rsx! {
        button { class: "{cls}", title: "{title}", disabled: !admin, onclick: move |_| {
            let on = !on_now;
            spawn(async move { match put_assistant(Some(on), None, None, None, None, None).await {
                Ok(_) => { if let Ok(r) = get_risk().await { st.risk.set(Some(r)); } st.notify("info", if on { "AI assistant trading enabled" } else { "AI assistant set to read-only" }); }
                Err(e) => st.toast.set(Some((false, format!("Failed: {e}")))) } });
        }, "{label}" }
    }
}

#[component]
fn OperatorChip() -> Element {
    let mut st = app_state();
    let op = st.op.read().clone().unwrap_or_default();
    let Some(o) = op.operator.clone() else { return rsx! {} };
    let cls = match o.role.as_str() { "admin" => "tag lav xs", "trader" => "tag mint xs", "risk" => "tag peach xs", _ => "tag xs" };
    rsx! {
        div { class: "op-chip", title: "Operator {o.name} ({o.role}) · click to lock the terminal",
            onclick: move |_| { spawn(async move { let _ = op_logout().await; st.op.set(Some(OpStatus { configured: true, ..Default::default() })); st.notify("info", "Terminal locked"); }); },
            if o.role != "admin" { span { class: "{cls}", "{o.role}" } }
            span { "{o.name}" } span { class: "muted", "🔒" }
        }
    }
}

#[component]
pub fn AccountMenu() -> Element {
    let mut st = app_state();
    let mut open = use_signal(|| false);
    let sess = st.session.read().clone().unwrap_or_default();
    let acct = if sess.selected_account.is_empty() { st.snapshot.read().as_ref().map(|s| s.account.account_id.clone()).unwrap_or_default() } else { sess.selected_account.clone() };
    let is_paper = acct.starts_with('D') || sess.mode == "paper";
    rsx! {
        div { class: "acct-menu",
            button { class: "acct-btn", onclick: move |_| { let v = !*open.peek(); open.set(v); },
                span { class: if is_paper { "tag mint xs" } else { "tag rose xs" }, if is_paper { "PAPER" } else { "LIVE" } }
                span { class: "acct num", "{acct}" } span { class: "muted", "▾" }
            }
            if *open.read() {
                div { class: "ctx-bg", onclick: move |_| open.set(false) }
                div { class: "acct-dd",
                    div { class: "muted small", "Interactive Brokers · {sess.username_masked}" }
                    for a in sess.accounts.clone() {
                        {
                            let id = a.account_id.clone();
                            let id2 = a.account_id.clone();
                            let cur = id == acct;
                            rsx! { button { class: if cur { "ctx-item sel" } else { "ctx-item" }, onclick: move |_| {
                                let id = id.clone(); open.set(false);
                                spawn(async move { match setup_account(id.clone()).await { Ok(_) => { if let Ok(s) = get_setup_status().await { st.session.set(Some(s)); } st.notify("info", format!("Switched to account {id}")); st.refresh_tick += 1; } Err(e) => st.toast.set(Some((false, format!("Switch failed: {e}")))) } });
                            }, span { class: "num", "{id2}" } span { class: "muted small", " {money(a.net_liquidation)}" } } }
                        }
                    }
                    div { class: "ctx-sep" }
                    button { class: "ctx-item", onclick: move |_| { open.set(false); spawn(async move { let _ = setup_logout(false).await; if let Ok(s) = get_setup_status().await { st.session.set(Some(s)); } }); }, "Switch user (keep gateway running)" }
                    button { class: "ctx-item danger", onclick: move |_| { open.set(false); spawn(async move { let _ = setup_logout(true).await; if let Ok(s) = get_setup_status().await { st.session.set(Some(s)); } }); }, "Sign out & stop IB Gateway" }
                }
            }
        }
    }
}

// ---------------- QOS Cloud menu (step 34) ----------------
fn cloud_dur(s: i64) -> String {
    if s <= 0 { "0h".into() } else if s >= 86400 { format!("{}d {}h", s / 86400, s % 86400 / 3600) } else if s >= 3600 { format!("{}h {}m", s / 3600, s % 3600 / 60) } else { format!("{}m", (s / 60).max(1)) }
}

fn cloud_ago(now: i64, ts: Option<i64>) -> String {
    match ts { Some(t) if t > 0 => format!("{} ago", cloud_dur(now - t)), _ => "never".into() }
}

#[component]
pub fn CloudMenu() -> Element {
    let mut st = app_state();
    let mut open = use_signal(|| false);
    let mut data = use_signal(|| serde_json::Value::Null);
    let mut devices = use_signal(|| None::<serde_json::Value>);
    let mut datasets = use_signal(|| None::<serde_json::Value>);
    let mut busy = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut tick = use_signal(|| 0u32);
    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            if let Ok(t) = get_cloud_status().await { data.set(serde_json::from_str(&t).unwrap_or_default()); }
        });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        let mut n = 0u32;
        loop {
            crate::sleep_ms(3000).await;
            n += 1;
            let pending = matches!(js(&data.peek(), &["login", "state"]).as_str(), "pending" | "slow_down");
            if pending || n % 20 == 0 { tick += 1; }
        }
    });
    let d = data.read().clone();
    if d.is_null() { return rsx! {}; }
    let now = d.get("now").and_then(|x| x.as_i64()).unwrap_or(0);
    let signed_in = d.get("signed_in").and_then(|x| x.as_bool()).unwrap_or(false);
    let features = d.get("cloud_features").and_then(|x| x.as_bool()).unwrap_or(false);
    let left = d.get("offline_left_s").and_then(|x| x.as_i64()).unwrap_or(0);
    let plan = plan_label(&js(&d, &["plan"]));
    let err = js(&d, &["last_error"]);
    let login = d.get("login").cloned().unwrap_or(serde_json::Value::Null);
    let lstate = js(&login, &["state"]);
    let pending = matches!(lstate.as_str(), "pending" | "slow_down");
    let (chip_cls, chip_txt) = if !signed_in { ("pill off", "☁".to_string()) }
        else if !features { ("pill bad", "Cloud · sign-in expired".to_string()) }
        else if left < 2 * 86400 { ("pill warn", format!("Cloud · offline {}", cloud_dur(left))) }
        else { ("pill ok", format!("Cloud · {}", if plan.is_empty() { "on".to_string() } else { plan.clone() })) };
    let grants: Vec<serde_json::Value> = d.get("grants").and_then(|g| g.as_array()).cloned().unwrap_or_default();
    let busy_now = busy.read().clone();
    let mut act = move |what: &'static str, arg: String| {
        busy.set(what.to_string());
        spawn(async move {
            match cloud_action(what.to_string(), arg.clone()).await {
                Ok(t) => {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    match what {
                        "devices" => devices.set(Some(v)),
                        "datasets" => datasets.set(Some(v)),
                        "pull" | "drop" => {
                            if what == "pull" { st.toast.set(Some((true, format!("{} {} ready", js(&v, &["slug"]), js(&v, &["version"]))))); }
                            if let Ok(t) = cloud_action("datasets".into(), String::new()).await { datasets.set(serde_json::from_str(&t).ok()); }
                        }
                        "remove" => {
                            st.toast.set(Some((true, "Device removed from your QOS account".to_string())));
                            if let Ok(t) = cloud_action("devices".into(), String::new()).await { devices.set(serde_json::from_str(&t).ok()); }
                        }
                        "sync" => st.toast.set(Some((true, "QOS Cloud synced".to_string()))),
                        "logout" => { devices.set(None); st.toast.set(Some((true, "Signed out of QOS Cloud".to_string()))); }
                        _ => {}
                    }
                }
                Err(e) => st.toast.set(Some((false, format!("QOS Cloud: {e}")))),
            }
            busy.set(String::new());
            confirm.set(String::new());
            tick += 1;
        });
    };
    rsx! {
        div { class: "acct-menu cloud-menu",
            button { class: "cloud-chip {chip_cls}", title: "QOS Cloud account · data and agents (trading never depends on it)",
                onclick: move |_| { let v = !*open.peek(); open.set(v); }, "{chip_txt}" }
            if *open.read() {
                div { class: "ctx-bg", onclick: move |_| open.set(false) }
                div { class: "acct-dd cloud-dd",
                    div { class: "cloud-head",
                        span { class: "recon-title", "QOS Cloud" }
                        span { class: "muted small", "{js(&d, &[\"url\"])}" }
                    }
                    if !signed_in {
                        if pending {
                            div { class: "cloud-body",
                                div { class: "small", "Approve this code on the QOS Cloud page:" }
                                div { class: "cloud-code num", "{js(&login, &[\"user_code\"])}" }
                                div { class: "muted small cloud-url", "{js(&login, &[\"verification_uri_complete\"])}" }
                                div { class: "cloud-btns",
                                    button { class: "mini accent", onclick: move |_| act("open", String::new()), "Open approval page" }
                                    button { class: "mini", onclick: move |_| act("cancel", String::new()), "Cancel" }
                                }
                                div { class: "muted small", "Waiting for approval…" }
                            }
                        } else {
                            div { class: "cloud-body",
                                div { class: "small", "Sign in to use data and agents from QOS Cloud. Your broker account and trades stay on this computer." }
                                if matches!(lstate.as_str(), "denied" | "expired" | "error") {
                                    div { class: "text-down small",
                                        match lstate.as_str() { "denied" => "Sign-in was denied.".to_string(), "expired" => "The code expired. Try again.".to_string(), _ => format!("Sign-in failed: {}", js(&login, &["message"])) } }
                                }
                                if !err.is_empty() { div { class: "text-down small", "{err}" } }
                                div { class: "cloud-btns",
                                    button { class: "mini accent", disabled: busy_now == "login", onclick: move |_| act("login", String::new()),
                                        if busy_now == "login" { "Starting…" } else { "Sign in" } }
                                }
                            }
                        }
                    } else {
                        div { class: "cloud-body",
                            div { class: "cloud-kv", span { class: "muted", "Account" } span { "{js(&d, &[\"email\"])}" } }
                            div { class: "cloud-kv", span { class: "muted", "Plan" } span { if plan.is_empty() { "—" } else { "{plan}" } } }
                            div { class: "cloud-kv", span { class: "muted", "Cloud features" }
                                span { class: if features { "text-up" } else { "text-down" },
                                    if features { "On · works offline for {cloud_dur(left)}" } else { "Off — sync to renew" } } }
                            div { class: "cloud-kv", span { class: "muted", "Last sync" } span { "{cloud_ago(now, d.get(\"last_sync\").and_then(|x| x.as_i64()))}" } }
                            if !grants.is_empty() {
                                div { class: "cloud-grants",
                                    for g in grants.iter() { span { class: "tag xs", "{js(g, &[\"sku\"])}" } }
                                }
                            }
                            if !err.is_empty() { div { class: "text-down small", "{err}" } }
                            div { class: "cloud-btns",
                                button { class: "mini", disabled: busy_now == "sync", onclick: move |_| act("sync", String::new()), if busy_now == "sync" { "Syncing…" } else { "Sync now" } }
                                button { class: "mini", onclick: move |_| { if devices.peek().is_some() { devices.set(None); } else { act("devices", String::new()); } }, "Devices" }
                                button { class: "mini", onclick: move |_| { if datasets.peek().is_some() { datasets.set(None); } else { act("datasets", String::new()); } }, "Data" }
                                button { class: "mini", title: "Purchases, payment method, cancel subscriptions", onclick: move |_| {
                                    spawn(async move {
                                        match billing_action("portal".into(), String::new()).await {
                                            Ok(_) => st.toast.set(Some((true, "Billing opened in your browser".to_string()))),
                                            Err(e) => st.toast.set(Some((false, format!("Billing: {e}")))),
                                        }
                                    });
                                }, "Billing" }
                                if confirm.read().as_str() == "logout" {
                                    button { class: "mini danger", onclick: move |_| act("logout", String::new()), "Sign out?" }
                                    button { class: "mini", onclick: move |_| confirm.set(String::new()), "No" }
                                } else {
                                    button { class: "mini", onclick: move |_| confirm.set("logout".into()), "Sign out" }
                                }
                            }
                            if let Some(ds) = datasets.read().clone() {
                                div { class: "cloud-devs",
                                    if !js(&ds, &["error"]).is_empty() { div { class: "text-down small", "{js(&ds, &[\"error\"])}" } }
                                    if ds.get("datasets").and_then(|a| a.as_array()).map(|a| a.is_empty()).unwrap_or(true) {
                                        div { class: "muted small", "No datasets in the QOS data store yet." }
                                    }
                                    for x in ds.get("datasets").and_then(|a| a.as_array()).cloned().unwrap_or_default() {
                                        {
                                            let slug = js(&x, &["slug"]);
                                            let (s1, s2) = (slug.clone(), slug.clone());
                                            let lat = x.get("latest").cloned().unwrap_or(serde_json::Value::Null);
                                            let loc = x.get("local").cloned().unwrap_or(serde_json::Value::Null);
                                            let entitled = x.get("entitled").and_then(|b| b.as_bool());
                                            let update = x.get("update").and_then(|b| b.as_bool()).unwrap_or(false);
                                            let lstate = js(&loc, &["state"]);
                                            let pulling = busy_now == "pull";
                                            rsx! {
                                                div { class: "cloud-ds",
                                                    div { class: "cloud-ds-top",
                                                        span { class: "cloud-dev-name", title: "{js(&x, &[\"description\"])}", "{js(&x, &[\"title\"])}" }
                                                        span { class: "muted small", "{slug}" }
                                                    }
                                                    div { class: "muted small",
                                                        if !lat.is_null() { "Latest {js(&lat, &[\"version\"])} · {jf(&lat, &[\"bytes\"]) / 1e6:.1} MB" }
                                                        if !loc.is_null() { " · on this computer: {js(&loc, &[\"version\"])}" }
                                                    }
                                                    if !loc.is_null() && lstate != "ok" { div { class: "text-down small", "{js(&loc, &[\"reason\"])}" } }
                                                    div { class: "cloud-btns",
                                                        if entitled == Some(false) {
                                                            if let Some(pr) = x.get("prices").and_then(|p| p.as_array()).and_then(|p| p.first()).cloned() {
                                                                {
                                                                    let pid = js(&pr, &["id"]);
                                                                    rsx! { button { class: "mini accent", onclick: move |_| {
                                                                        let pid = pid.clone();
                                                                        spawn(async move {
                                                                            match billing_action("checkout".into(), pid).await {
                                                                                Ok(_) => st.toast.set(Some((true, "Complete the payment in your browser, then reopen Data".to_string()))),
                                                                                Err(e) => st.toast.set(Some((false, format!("Buy: {e}")))),
                                                                            }
                                                                        });
                                                                    }, "Buy · {price_label(&pr)}" } }
                                                                }
                                                            } else {
                                                                span { class: "pill warn", "Needs {js(&x, &[\"sku\"])}" }
                                                            }
                                                        } else if entitled == Some(true) && (loc.is_null() || update || lstate == "withdrawn") {
                                                            button { class: "mini accent", disabled: pulling, onclick: move |_| act("pull", s1.clone()),
                                                                if pulling { "Downloading…" } else if loc.is_null() { "Download" } else { "Update" } }
                                                        } else if !loc.is_null() && lstate == "ok" {
                                                            span { class: "pill ok", "Ready · source=\"cloud:{slug}\"" }
                                                        }
                                                        if !loc.is_null() {
                                                            if *confirm.read() == format!("ds:{s2}") {
                                                                button { class: "mini danger", onclick: move |_| act("drop", s2.clone()), "Delete?" }
                                                            } else {
                                                                button { class: "mini", title: "Delete the downloaded copy", onclick: move |_| confirm.set(format!("ds:{s2}")), "✕" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(dv) = devices.read().clone() {
                                div { class: "cloud-devs",
                                    div { class: "muted small", "{plan_label(&js(&dv, &[\"plan\"]))} plan · {dv.get(\"devices\").and_then(|x| x.as_array()).map(|a| a.iter().filter(|x| x.get(\"counts_to_limit\").and_then(|b| b.as_bool()).unwrap_or(false)).count()).unwrap_or(0)} of {js(&dv, &[\"limit\"])} devices" }
                                    for x in dv.get("devices").and_then(|a| a.as_array()).cloned().unwrap_or_default() {
                                        {
                                            let id = js(&x, &["id"]);
                                            let (id2, id3) = (id.clone(), id.clone());
                                            let cur = x.get("current").and_then(|b| b.as_bool()).unwrap_or(false);
                                            let asking = *confirm.read() == id;
                                            rsx! {
                                                div { class: "cloud-dev",
                                                    span { class: "tag xs", "{js(&x, &[\"kind\"])}" }
                                                    span { class: "cloud-dev-name", "{js(&x, &[\"name\"])}" if cur { span { class: "muted", " (this computer)" } } }
                                                    span { class: "muted small", "{jdate(&js(&x, &[\"last_seen_at\"]))}" }
                                                    if asking {
                                                        button { class: "mini danger", onclick: move |_| act("remove", id2.clone()), "Remove?" }
                                                    } else {
                                                        button { class: "mini", title: if cur { "Signs this Terminal out" } else { "Sign that device out" }, onclick: move |_| confirm.set(id3.clone()), "✕" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------- Backtests tab: Nautilus backtest / parity / walk-forward reports ----------------
fn jf(v: &serde_json::Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for p in path { cur = match cur.get(*p) { Some(x) => x, None => return 0.0 }; }
    cur.as_f64().unwrap_or(0.0)
}
fn js(v: &serde_json::Value, path: &[&str]) -> String {
    let mut cur = v;
    for p in path { cur = match cur.get(*p) { Some(x) => x, None => return String::new() }; }
    match cur { serde_json::Value::String(s) => s.clone(), serde_json::Value::Null => String::new(), x => x.to_string() }
}
fn jdate(s: &str) -> String { s.replace('T', " ").chars().take(16).collect() }
fn jday(s: &str) -> String { s.chars().take(10).collect() }
fn jequity(v: Option<&serde_json::Value>) -> Vec<EquityPoint> {
    v.and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|p| {
        let arr = p.as_array()?;
        Some(EquityPoint { ts: arr.first()?.as_str().unwrap_or_default().to_string(), netliq: arr.get(1)?.as_f64()?, ..Default::default() })
    }).collect()).unwrap_or_default()
}
fn jparams(v: Option<&serde_json::Value>) -> String {
    v.and_then(|x| x.as_object()).map(|m| m.iter().map(|(k, x)| format!("{k}={x}")).collect::<Vec<_>>().join(" ")).unwrap_or_default()
}
fn pos_label(v: Option<&serde_json::Value>) -> &'static str {
    match v.and_then(|x| x.as_i64()) { Some(1) => "Long", Some(-1) => "Short", Some(0) => "Flat", _ => "—" }
}
fn bt_kind_tag(kind: &str) -> (&'static str, &'static str) {
    match kind { "parity" => ("tag lav xs", "Parity"), "walkforward" => ("tag mint xs", "Walk-fwd"), _ => ("tag sky xs", "Backtest") }
}

#[component]
fn BtEquity(points: Vec<EquityPoint>) -> Element {
    if points.len() < 2 { return rsx! { div { class: "chart-empty small", "Not enough closed trades for an equity curve" } }; }
    rsx! { EquityChart { points } }
}

#[component]
fn BacktestsTab() -> Element {
    let mut st = app_state();
    let mut items = use_signal(Vec::<BtReport>::new);
    let mut sel = use_signal(|| None::<(String, String)>);
    use_hook(move || { if let Some(x) = st.bt_open.write().take() { sel.set(Some(x)); } });
    let mut doc = use_signal(|| None::<serde_json::Value>);
    let mut err = use_signal(String::new);
    let mut tick = use_signal(|| 0u32);
    let mut filter = use_signal(String::new);

    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            match get_bt_reports().await {
                Ok(v) => {
                    if sel.peek().is_none() { if let Some(f) = v.first() { sel.set(Some((f.kind.clone(), f.file.clone()))); } }
                    items.set(v);
                    err.set(String::new());
                }
                Err(e) => err.set(e.to_string()),
            }
        });
    });
    use_effect(move || {
        if let Some((k, f)) = sel.read().clone() {
            spawn(async move {
                match get_bt_report(k, f).await {
                    Ok(t) => doc.set(serde_json::from_str(&t).ok()),
                    Err(e) => { doc.set(None); err.set(e.to_string()); }
                }
            });
        }
    });

    let flt = filter.read().clone();
    let list: Vec<BtReport> = items.read().iter().filter(|r| flt.is_empty() || r.kind == flt).cloned().collect();
    let cur = sel.read().clone();
    let d = doc.read().clone();

    rsx! {
        div { class: "bt-grid",
            div { class: "bt-list",
                div { class: "bt-head",
                    div { class: "tfs",
                        for (id, lbl) in [("", "All"), ("backtest", "Backtests"), ("parity", "Parity"), ("walkforward", "Walk-fwd")] {
                            button { class: if flt == id { "tf active" } else { "tf" }, onclick: move |_| filter.set(id.to_string()), "{lbl}" }
                        }
                    }
                    button { class: "mini", style: "margin-left:auto", title: "Reload reports", onclick: move |_| tick += 1, "↻" }
                }
                if !err.read().is_empty() { p { class: "empty small", "{err}" } }
                if list.is_empty() {
                    p { class: "empty small", "No reports yet. Run scripts/backtest.py, parity.py or walkforward.py in the bot folder." }
                }
                for r in list {
                    {
                        let (tcls, tlbl) = bt_kind_tag(&r.kind);
                        let active = cur.as_ref().map(|c| c.0 == r.kind && c.1 == r.file).unwrap_or(false);
                        let key = (r.kind.clone(), r.file.clone());
                        let pnl = r.pnl.unwrap_or(0.0);
                        let tf = r.tf_minutes.map(|m| format!("{m}m")).unwrap_or_default();
                        let created = jdate(&r.created);
                        let trades = r.trades.map(|t| format!("{t} trades")).unwrap_or_default();
                        rsx! {
                            div { class: if active { "bt-item active" } else { "bt-item" }, onclick: move |_| sel.set(Some(key.clone())),
                                div { class: "bt-item-top", span { class: "{tcls}", "{tlbl}" } span { class: "sym", "{r.instrument} {tf}" } }
                                div { class: "r num {pnl_class(pnl)}", if r.pnl.is_some() { "{money(pnl)}" } else { "—" } }
                                div { class: "muted small", "{created} · {r.label}" }
                                div { class: "r muted small", "{trades}" }
                            }
                        }
                    }
                }
            }
            div { class: "bt-detail",
                match d {
                    None => rsx! { p { class: "empty", "Select a report" } },
                    Some(v) => match js(&v, &["kind"]).as_str() {
                        "parity" => rsx! { ParityView { v } },
                        "walkforward" => rsx! { WalkForwardView { v } },
                        _ => rsx! { BtReportView { v } },
                    },
                }
            }
        }
    }
}

#[component]
fn BtReportView(v: serde_json::Value) -> Element {
    let pnl = jf(&v, &["pnl_usd"]);
    let wr_opt = v.get("win_rate").and_then(|x| x.as_f64());
    let wr = wr_opt.unwrap_or(0.0) * 100.0;
    let bdec = { let inst = js(&v, &["instrument"]); if inst.is_empty() { 5 } else { decimals_for_key(&format!("FX:{}", crate::calc::split_pair(&inst).0 + &crate::calc::split_pair(&inst).1)) } };
    let eq = jequity(v.get("equity"));
    let trades: Vec<serde_json::Value> = v.get("trades").and_then(|t| t.as_array()).cloned().unwrap_or_default();
    let total = v.get("trades_total").and_then(|x| x.as_i64()).unwrap_or(trades.len() as i64);
    let shown = trades.len();
    rsx! {
        div { class: "bt-title",
            span { class: "sym", "{js(&v, &[\"strategy\"])}" }
            span { class: "muted small", "{js(&v, &[\"bar_type\"])} · {jdate(&js(&v, &[\"from\"]))} → {jdate(&js(&v, &[\"to\"]))} · {jf(&v, &[\"bars\"])} bars" }
        }
        div { class: "perf-cards bt",
            StatCard { label: "Net P&L".to_string(), value: money(pnl), cls: pnl_class(pnl).to_string() }
            StatCard { label: "Return".to_string(), value: format!("{:.2}%", jf(&v, &["return_pct"])), cls: pnl_class(pnl).to_string() }
            StatCard { label: "Trades".to_string(), value: format!("{}", jf(&v, &["closed_positions"])), cls: String::new() }
            StatCard { label: "Win rate".to_string(), value: frac_pct(wr_opt, 1), cls: (if wr_opt.is_none() { "" } else if wr >= 50.0 { "text-up" } else { "text-down" }).to_string() }
            StatCard { label: "Max drawdown".to_string(), value: money(jf(&v, &["max_drawdown_usd"])), cls: "text-down".to_string() }
            StatCard { label: "Avg win / loss".to_string(), value: format!("{} / {}", money(jf(&v, &["avg_win"])), money(jf(&v, &["avg_loss"]))), cls: String::new() }
            StatCard { label: "Signals".to_string(), value: format!("{}", jf(&v, &["signals"])), cls: String::new() }
            StatCard { label: "Orders".to_string(), value: format!("{}", jf(&v, &["orders"])), cls: String::new() }
            StatCard { label: "Params".to_string(), value: jparams_short(v.get("config")), cls: String::new() }
            StatCard { label: "Balance".to_string(), value: money(jf(&v, &["balance_usd"])), cls: String::new() }
        }
        div { class: "perf-chart", h4 { "Equity curve" } BtEquity { points: eq } }
        h4 { class: "perf-h", "Trades" span { class: "muted small", " last {shown} of {total}" } }
        div { class: "bt-trades",
            table { class: "grid-tbl compact",
                thead { tr { th { "Side" } th { "Opened" } th { "Closed" } th { class: "r", "Entry" } th { class: "r", "Exit" } th { class: "r", "P&L" } } }
                tbody {
                    for t in trades.into_iter().rev() {
                        {
                            let p = jf(&t, &["pnl"]);
                            let side = js(&t, &["side"]);
                            let scls = if side == "LONG" { "tag mint xs" } else { "tag rose xs" };
                            rsx! { tr {
                                td { span { class: "{scls}", "{side}" } }
                                td { class: "num", "{jdate(&js(&t, &[\"opened\"]))}" } td { class: "num", "{jdate(&js(&t, &[\"closed\"]))}" }
                                td { class: "r num", "{px(jf(&t, &[\"entry\"]), bdec)}" } td { class: "r num", "{px(jf(&t, &[\"exit\"]), bdec)}" }
                                td { class: "r num {pnl_class(p)}", "{money(p)}" }
                            } }
                        }
                    }
                }
            }
        }
    }
}

fn jparams_short(cfg: Option<&serde_json::Value>) -> String {
    let Some(c) = cfg else { return String::new() };
    let g = |k: &str| c.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".into());
    format!("EMA {}/{} · SL {} TP {}", g("fast"), g("slow"), g("sl_pips"), g("tp_pips"))
}

#[component]
fn ParityView(v: serde_json::Value) -> Element {
    let rate = jf(&v, &["match_rate"]) * 100.0;
    let min = { let m = jf(&v, &["min_match"]); if m > 0.0 { m * 100.0 } else { 95.0 } };
    let pass = rate >= min;
    let (old_p, new_p, diff) = (jf(&v, &["old", "pnl_usd"]), jf(&v, &["new", "pnl_usd"]), jf(&v, &["pnl_diff_usd"]));
    let rows: Vec<serde_json::Value> = v.get("divergence_rows").and_then(|t| t.as_array()).cloned().unwrap_or_default();
    let eq = jequity(v.get("new").and_then(|n| n.get("equity")));
    rsx! {
        div { class: "bt-title",
            span { class: "sym", "Parity · {js(&v, &[\"config\", \"instrument\"])} {js(&v, &[\"config\", \"tf_minutes\"])}m" }
            span { class: if pass { "pill up" } else { "pill down" }, if pass { "PASS" } else { "FAIL" } }
            span { class: "muted small", "{jdate(&js(&v, &[\"from\"]))} → {jdate(&js(&v, &[\"to\"]))} · {jf(&v, &[\"bars\"])} bars · {jparams_short(v.get(\"config\"))}" }
        }
        div { class: "perf-cards bt",
            StatCard { label: "Decision match".to_string(), value: format!("{rate:.2}%"), cls: (if pass { "text-up" } else { "text-down" }).to_string() }
            StatCard { label: "Decisions old / new".to_string(), value: format!("{} / {}", jf(&v, &["old_actions"]), jf(&v, &["new_actions"])), cls: String::new() }
            StatCard { label: "Divergences".to_string(), value: format!("{}", jf(&v, &["divergences"])), cls: String::new() }
            StatCard { label: "P&L old / new".to_string(), value: format!("{} / {}", money(old_p), money(new_p)), cls: String::new() }
            StatCard { label: "P&L difference".to_string(), value: money(diff), cls: String::new() }
        }
        p { class: "muted small", "Legacy agent replayed with its own code on the same bars as Nautilus. Fills at bar close, 0.2 bps fees, no SL/TP." }
        div { class: "perf-chart", h4 { "Equity · Nautilus strategy" } BtEquity { points: eq } }
        h4 { class: "perf-h", "Where decisions differ" span { class: "muted small", " first {rows.len()}" } }
        if rows.is_empty() { p { class: "empty small", "No differences" } } else {
            div { class: "bt-trades",
                table { class: "grid-tbl compact",
                    thead { tr { th { "Bar (UTC)" } th { "Legacy agent" } th { "Nautilus" } } }
                    tbody { for r in rows { tr {
                        td { class: "num", "{jdate(&js(&r, &[\"ts\"]))}" } td { "{pos_label(r.get(\"old\"))}" } td { "{pos_label(r.get(\"new\"))}" }
                    } } }
                }
            }
        }
    }
}

#[component]
fn WalkForwardView(v: serde_json::Value) -> Element {
    let oos = jf(&v, &["oos", "pnl_usd"]);
    let base = jf(&v, &["baseline", "pnl_usd"]);
    let wr_txt = frac_pct(v.get("oos").and_then(|o| o.get("win_rate")).and_then(|x| x.as_f64()), 1);
    let folds: Vec<serde_json::Value> = v.get("folds").and_then(|t| t.as_array()).cloned().unwrap_or_default();
    let eq = jequity(v.get("oos").and_then(|n| n.get("equity")));
    let picked = js(&v, &["most_picked", "params"]).replace('\'', "");
    rsx! {
        div { class: "bt-title",
            span { class: "sym", "Walk-forward · {js(&v, &[\"config\", \"instrument\"])} {js(&v, &[\"config\", \"tf_minutes\"])}m" }
            span { class: "muted small", "{jdate(&js(&v, &[\"start\"]))} → {jdate(&js(&v, &[\"end\"]))} · train {jf(&v, &[\"train_days\"])}d / test {jf(&v, &[\"test_days\"])}d" }
        }
        div { class: "perf-cards bt",
            StatCard { label: "Out-of-sample P&L".to_string(), value: money(oos), cls: pnl_class(oos).to_string() }
            StatCard { label: "Baseline P&L".to_string(), value: money(base), cls: pnl_class(base).to_string() }
            StatCard { label: "Positive folds".to_string(), value: format!("{} / {}", jf(&v, &["oos", "positive_folds"]), jf(&v, &["oos", "folds"])), cls: String::new() }
            StatCard { label: "OOS trades · win".to_string(), value: format!("{} · {wr_txt}", jf(&v, &["oos", "trades"])), cls: String::new() }
            StatCard { label: "OOS max drawdown".to_string(), value: money(jf(&v, &["oos", "max_drawdown_usd"])), cls: "text-down".to_string() }
        }
        if !picked.is_empty() { p { class: "muted small", "Most picked: {picked} · {jf(&v, &[\"most_picked\", \"count\"])} folds. Baseline = current config without tuning." } }
        div { class: "perf-chart", h4 { "Out-of-sample equity (test periods stitched)" } BtEquity { points: eq } }
        h4 { class: "perf-h", "Folds" }
        div { class: "bt-trades",
            table { class: "grid-tbl compact",
                thead { tr { th { "Test period" } th { "Best params (train)" } th { class: "r", "Train P&L" } th { class: "r", "Test P&L" } th { class: "r", "Trades" } th { class: "r", "Win%" } th { class: "r", "Baseline" } } }
                tbody { for f in folds {
                    {
                        let (tp, bp) = (jf(&f, &["test_pnl"]), jf(&f, &["baseline_pnl"]));
                        rsx! { tr {
                            td { class: "num", "{jday(&js(&f, &[\"test_start\"]))} → {jday(&js(&f, &[\"test_end\"]))}" }
                            td { class: "small", "{jparams(f.get(\"best_params\"))}" }
                            td { class: "r num {pnl_class(jf(&f, &[\"train_pnl\"]))}", "{money(jf(&f, &[\"train_pnl\"]))}" }
                            td { class: "r num {pnl_class(tp)}", "{money(tp)}" }
                            td { class: "r num", "{jf(&f, &[\"test_trades\"])}" }
                            td { class: "r num", "{frac_pct(f.get(\"test_win_rate\").and_then(|x| x.as_f64()), 0)}" }
                            td { class: "r num {pnl_class(bp)}", "{money(bp)}" }
                        } }
                    }
                } }
            }
        }
    }
}

// ---------------- Nodes tab: Nautilus TradingNode manager ----------------
fn node_state_pill(state: &str) -> (&'static str, &'static str) {
    match state {
        "running" => ("pill ok", "Running"),
        "starting" => ("pill ok", "Starting"),
        "degraded" => ("pill warn", "Disconnected"),
        "backoff" => ("pill warn", "Restarting"),
        "crashed" => ("pill bad", "Crashed"),
        _ => ("pill", "Stopped"),
    }
}

#[derive(Clone, PartialEq, Default)]
struct NodeForm { id: String, name: String, config_file: String, auto_restart: bool }

fn jnum(v: &serde_json::Value, k: &str) -> String {
    match v.get(k) { Some(serde_json::Value::Number(n)) => n.to_string(), Some(serde_json::Value::String(s)) => s.clone(), _ => String::new() }
}

#[component]
fn NodesTab() -> Element {
    let mut st = app_state();
    let mut data = use_signal(NodesResp::default);
    let mut err = use_signal(String::new);
    let mut tick = use_signal(|| 0u32);
    let mut form = use_signal(|| None::<NodeForm>);
    let mut busy = use_signal(String::new);
    let mut log_node = use_signal(String::new);
    let mut log_level = use_signal(String::new);
    let mut log_lines = use_signal(Vec::<String>::new);
    let mut confirm_del = use_signal(String::new);

    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            match get_nodes().await { Ok(d) => { data.set(d); err.set(String::new()); } Err(e) => err.set(e.to_string()) }
            let id = log_node.peek().clone();
            if !id.is_empty() { if let Ok(l) = get_node_log(id, log_level.peek().clone()).await { log_lines.set(l); } }
        });
    });
    use_effect(move || {
        let id = log_node.read().clone();
        let lv = log_level.read().clone();
        if id.is_empty() { return; }
        spawn(async move { if let Ok(l) = get_node_log(id, lv).await { log_lines.set(l); } });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop { crate::sleep_ms(3000).await; tick += 1; }
    });

    let mut act = move |id: String, action: &'static str| {
        busy.set(id.clone());
        spawn(async move {
            match node_action(id.clone(), action.to_string()).await {
                Ok(s) => st.toast.set(Some((true, format!("{id}: {action} → {s}")))),
                Err(e) => st.toast.set(Some((false, format!("{id}: {action} failed — {e}")))),
            }
            if action == "delete" && *log_node.peek() == id { log_node.set(String::new()); }
            busy.set(String::new());
            tick += 1;
        });
    };

    let d = data.read().clone();
    let gw = d.gateway.clone();
    let gw_text = format!("{} · {} · port {} · {}", js(&gw, &["gateway_id"]), js(&gw, &["mode"]), js(&gw, &["port"]),
                          { let a = js(&gw, &["account_id"]); if a.is_empty() { "no account".to_string() } else { a } });
    let live = js(&gw, &["mode"]) == "live";
    let configs = d.configs.clone();
    let f = form.read().clone();
    let busy_id = busy.read().clone();
    let sel_log = log_node.read().clone();
    let live_map = st.node_live.read().clone();

    rsx! {
        div { class: "nodes",
            div { class: "perf-bar",
                span { class: "panel-title", style: "margin:0", "Nautilus nodes" }
                span { class: if live { "pill bad" } else { "pill" }, title: "Nodes trade on the gateway selected at sign-in", "{gw_text}" }
                div { class: "tabs-right",
                    button { class: "mini", title: "Reload", onclick: move |_| tick += 1, "↻" }
                    button { class: "mini accent", onclick: move |_| {
                        let c = configs.first().cloned().unwrap_or_default();
                        form.set(Some(NodeForm { config_file: c.file.clone(), auto_restart: true, ..Default::default() }));
                    }, "New node" }
                }
            }
            if !err.read().is_empty() { p { class: "empty small", "{err}" } }
            if let Some(fv) = f.clone() {
                NodeFormView { fv, configs: d.configs.clone(), on_close: move |_| form.set(None), on_saved: move |_| { form.set(None); tick += 1; } }
            }
            if d.nodes.is_empty() && f.is_none() {
                p { class: "empty", "No nodes yet. Press New node, pick a strategy config from configs/strategies and start it." }
            }
            if !d.nodes.is_empty() {
                table { class: "grid-tbl nodes-tbl",
                    thead { tr {
                        th { "Node" } th { "Strategy" } th { "State" } th { "Risk" } th { "Position" } th { "Last signal" } th { class: "r", "Uptime" } th { class: "r", "Restarts" }
                        th { "Client IDs" } th { "Health" } th { class: "r", "" }
                    } }
                    tbody {
                        for n in d.nodes.clone() {
                            {
                                let (pcls, plbl) = node_state_pill(&n.state);
                                let c = n.config.clone();
                                let strat = format!("{} {}m · {}", js(&c, &["instrument"]), js(&c, &["tf_minutes"]), n.summary);
                                let sltp = { let (sl, tp) = (jf(&c, &["sl_pips"]), jf(&c, &["tp_pips"])); if sl > 0.0 || tp > 0.0 { format!("SL {sl} · TP {tp} pips") } else { "no SL/TP".to_string() } };
                                let size = js(&c, &["trade_size"]);
                                let uptime = n.uptime_s.map(|s| dur_fmt(s as f64)).unwrap_or_else(|| "—".into());
                                let cids = n.client_id.map(|b| format!("{b}/{}", b + 1)).unwrap_or_else(|| "auto".into());
                                let h = n.health.clone();
                                let conn = match h.get("connected").and_then(|x| x.as_bool()) { Some(true) => "IB connected", Some(false) => "IB disconnected", None => "" };
                                let errs = jf(&h, &["errors"]) as i64;
                                let last_err = js(&h, &["last_error"]);
                                let is_busy = busy_id == n.id;
                                let (id1, id2, id3, id4, id5, id6, id7) = (n.id.clone(), n.id.clone(), n.id.clone(), n.id.clone(), n.id.clone(), n.id.clone(), n.id.clone());
                                let edit_form = NodeForm { id: n.id.clone(), name: n.name.clone(), config_file: n.config_file.clone(), auto_restart: n.auto_restart };
                                let row_cls = if sel_log == n.id { "sel" } else { "" };
                                // live from the heartbeat (WebSocket) if available, otherwise the value from /nodes
                                let hb = live_map.get(&n.id).and_then(|e| e.get("data")).cloned().unwrap_or_else(|| n.live.clone());
                                let positions: Vec<serde_json::Value> = if n.running { hb.get("positions").and_then(|p| p.as_array()).cloned().unwrap_or_default() } else { vec![] };
                                let realized = hb.get("realized_pnl").and_then(|x| x.as_f64());
                                let sig = n.live.get("last_signal").cloned().filter(|_| n.running).unwrap_or(serde_json::Value::Null);
                                let sig_txt = if sig.is_null() { "—".to_string() } else { format!("{} · {}", if js(&sig, &["name"]) == "cross_up" { "Cross up" } else { "Cross down" }, jdate(&js(&sig, &["ts"])).chars().skip(11).collect::<String>()) };
                                let hb_age = n.live.get("heartbeat_age_s").and_then(|x| x.as_f64());
                                let rk = hb.get("risk").cloned().unwrap_or(serde_json::Value::Null);
                                let (rcls, rlbl) = node_risk_pill(&rk, n.running);
                                let halt_reason = js(&rk, &["halt_reason"]);
                                let blocked: i64 = rk.get("blocked").and_then(|b| b.as_object()).map(|m| m.values().filter_map(|v| v.as_i64()).sum()).unwrap_or(0);
                                let blocked_txt = if blocked == 1 { "1 entry blocked".to_string() } else { format!("{blocked} entries blocked") };
                                let blocked_tip = rk.get("blocked").and_then(|b| b.as_object()).map(|m| m.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join("\n")).unwrap_or_default();
                                let deleting = *confirm_del.read() == n.id;
                                rsx! {
                                    tr { class: "{row_cls}",
                                        td { div { class: "sym", "{n.name}" } div { class: "muted small", "{n.id}" if !n.auto_restart { " · no auto-restart" } } }
                                        td { div { "{strat}" } div { class: "muted small", "{size} · {sltp}" } }
                                        td { span { class: "{pcls}", "{plbl}" } if !n.reason.is_empty() { div { class: "muted small node-reason", "{n.reason}" } } }
                                        td { class: "small",
                                            span { class: "{rcls}", title: "{halt_reason}", "{rlbl}" }
                                            if !halt_reason.is_empty() && n.running { div { class: "muted node-reason", "{halt_reason}" } }
                                            if blocked > 0 { div { class: "text-down", title: "{blocked_tip}", "{blocked_txt}" } }
                                        }
                                        td { class: "small",
                                            if positions.is_empty() { span { class: "muted", if n.running { "Flat" } else { "—" } } }
                                            for p in positions.iter() {
                                                {
                                                    let side = js(p, &["side"]);
                                                    let up = jf(p, &["unrealized_pnl"]);
                                                    rsx! { div {
                                                        span { class: if side == "LONG" { "tag mint xs" } else { "tag rose xs" }, "{side}" }
                                                        " {qty_fmt(jf(p, &[\"qty\"]))} @ {jf(p, &[\"avg_px\"]):.5} "
                                                        span { class: "num {pnl_class(up)}", "{money(up)}" }
                                                    } }
                                                }
                                            }
                                            if let Some(r) = realized { div { class: "muted", "Realized " span { class: "num {pnl_class(r)}", "{money(r)}" } } }
                                        }
                                        td { class: "small", "{sig_txt}" }
                                        td { class: "r num", "{uptime}" }
                                        td { class: "r num", "{n.restarts_15m}" }
                                        td { class: "num", "{cids}" }
                                        td { class: "small",
                                            if !conn.is_empty() { div { class: if conn == "IB connected" { "text-up" } else { "text-down" }, "{conn}" } }
                                            if errs > 0 { div { class: "text-down", title: "{last_err}", "{errs} error(s) since start" } }
                                            if n.running { match hb_age {
                                                Some(a) if a <= 20.0 => rsx! { div { class: "muted", "Events ♥ {a:.0}s ago" } },
                                                Some(a) => rsx! { div { class: "text-down", "No events for {a:.0}s" } },
                                                None => rsx! { div { class: "muted", "Events: waiting" } },
                                            } }
                                        }
                                        td { class: "r node-actions",
                                            if n.running || n.desired {
                                                button { class: "mini", disabled: is_busy || !st.can_trade(), onclick: move |_| act(id1.clone(), "restart"), "Restart" }
                                                button { class: "mini danger", disabled: is_busy || !st.can_trade(), onclick: move |_| { let id = id2.clone(); st.ask("Stop node?", format!("Stop {id}? The node cancels its orders and closes its positions when it stops."), "Stop node", ConfirmAction::NodeStop(id)); }, if is_busy { "…" } else { "Stop" } }
                                            } else {
                                                button { class: "mini accent", disabled: is_busy || !st.can_trade(), onclick: move |_| act(id3.clone(), "start"), if is_busy { "…" } else { "Start" } }
                                                button { class: "mini", onclick: move |_| form.set(Some(edit_form.clone())), "Edit" }
                                                if deleting {
                                                    button { class: "mini danger", onclick: move |_| { confirm_del.set(String::new()); act(id4.clone(), "delete"); }, "Delete?" }
                                                    button { class: "mini", onclick: move |_| confirm_del.set(String::new()), "No" }
                                                } else {
                                                    button { class: "mini", disabled: !st.can_trade(), onclick: move |_| confirm_del.set(id5.clone()), "✕" }
                                                }
                                            }
                                            button { class: if sel_log == id6 { "mini accent" } else { "mini" }, onclick: move |_| {
                                                if *log_node.peek() == id7 { log_node.set(String::new()); } else { log_lines.set(vec![]); log_node.set(id7.clone()); }
                                            }, "Log" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SystemPanel {}
            PortfolioPanel {}
            if !d.nodes.is_empty() { SoakPanel {} }
            NodeEventsFeed { nodes: d.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>() }
            if !sel_log.is_empty() {
                div { class: "node-log",
                    div { class: "perf-bar",
                        span { class: "panel-title", style: "margin:0", "Log · {sel_log}" }
                        div { class: "tfs",
                            for (lv, lbl) in [("", "All"), ("warn", "Warnings"), ("error", "Errors")] {
                                button { class: if *log_level.read() == lv { "tf active" } else { "tf" }, onclick: move |_| log_level.set(lv.to_string()), "{lbl}" }
                            }
                        }
                        span { class: "muted small", "auto-refresh 3s · last {log_lines.read().len()} lines" }
                        div { class: "tabs-right", button { class: "mini", onclick: move |_| log_node.set(String::new()), "Close" } }
                    }
                    pre { class: "node-log-body",
                        for ln in log_lines.read().iter().rev().take(400) {
                            {
                                let cls = if ln.contains("[ERROR]") { "text-down" } else if ln.contains("[WARN]") { "log-warn" } else if ln.starts_with("=====") { "log-start" } else { "" };
                                rsx! { div { class: "{cls}", "{ln}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NodeFormView(fv: NodeForm, configs: Vec<NodeConfigFile>, on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let mut st = app_state();
    let mut f = use_signal(|| fv.clone());
    let mut saving = use_signal(|| false);
    let mut msg = use_signal(String::new);
    let editing = !fv.id.is_empty();
    let cur = f.read().clone();
    let save = move |_| {
        let v = f.peek().clone();
        let name = if v.name.trim().is_empty() { v.config_file.trim_end_matches(".json").to_string() } else { v.name.clone() };
        // Strategy parameters are edited in the Strategies tab (config) · editing a node = name / config / auto-restart (existing overrides are kept)
        let overrides = if editing { "null".to_string() } else { "{}".to_string() };
        saving.set(true);
        spawn(async move {
            match save_node(v.id.clone(), name, v.config_file.clone(), overrides, v.auto_restart).await {
                Ok(n) => { st.toast.set(Some((true, format!("Node {} saved", n.id)))); on_saved.call(()); }
                Err(e) => msg.set(e.to_string()),
            }
            saving.set(false);
        });
    };
    rsx! {
        div { class: "node-form",
            div { class: "panel-title", style: "margin:0 0 6px", if editing { "Edit node · {cur.id}" } else { "New node" } }
            div { class: "node-form-grid",
                label { span { "Name" } input { value: "{cur.name}", placeholder: "e.g. Breakout USDJPY 15m", disabled: editing, oninput: move |e| f.write().name = e.value() } }
                label { style: "grid-column: span 2", span { "Strategy config" }
                    Dropdown { value: "{cur.config_file}", onchange: move |v| f.write().config_file = v,
                        options: configs.iter().map(|c| (c.file.clone(), format!("{} · {} {}m", c.file, c.instrument, c.tf_minutes.unwrap_or(0)))).collect::<DdOpts>() }
                }
                div { class: "node-form-checks",
                    label { class: "chk", input { r#type: "checkbox", checked: cur.auto_restart, oninput: move |e| f.write().auto_restart = e.checked() } span { "Auto-restart if it stops" } }
                }
            }
            if !msg.read().is_empty() { p { class: "text-down small", "{msg}" } }
            div { class: "node-form-btns",
                button { class: "mini accent", disabled: *saving.read() || cur.config_file.is_empty(), onclick: save, if *saving.read() { "Saving…" } else if editing { "Save" } else { "Create" } }
                button { class: "mini", onclick: move |_| on_close.call(()), "Cancel" }
                span { class: "muted small", "Edit strategy parameters in the Strategies tab." }
            }
        }
    }
}


fn node_risk_pill(rk: &serde_json::Value, running: bool) -> (&'static str, &'static str) {
    if !running { return ("pill", "—"); }
    if rk.is_null() { return ("pill", "Unknown"); }
    if rk.get("killed").and_then(|x| x.as_bool()).unwrap_or(false) { return ("pill bad", "Killed"); }
    if !rk.get("synced").and_then(|x| x.as_bool()).unwrap_or(false) { return ("pill warn", "Waiting for limits"); }
    match rk.get("trading_state").and_then(|x| x.as_str()).unwrap_or("") {
        "HALTED" => ("pill bad", "Halted"),
        "REDUCING" => ("pill warn", "Reduce-only"),
        _ => if rk.get("age_s").and_then(|x| x.as_f64()).unwrap_or(0.0) > 60.0 { ("pill warn", "Limits stale") } else { ("pill ok", "Limits synced") },
    }
}

// ---------------- live event feed from Nautilus nodes (via Redis → API → WebSocket) ----------------
fn event_summary(ev: &serde_json::Value) -> String {
    let d = ev.get("data").cloned().unwrap_or(serde_json::Value::Null);
    let g = |k: &str| js(&d, &[k]);
    let qty = qty_fmt(jf(&d, &["qty"]));
    match js(ev, &["type"]).as_str() {
        "signal" => {
            let action = match d.get("action").and_then(|x| x.as_i64()) { Some(1) => "→ Long", Some(-1) => "→ Short", Some(0) => "→ Flat", _ => "· no trade" };
            format!("{} · close {} · EMA {} / {} {action}", if g("name") == "cross_up" { "Cross up" } else { "Cross down" }, g("close"), g("fast"), g("slow"))
        }
        "order" => {
            let px = if d.get("price").is_some() { format!(" @ {}", g("price")) } else if d.get("trigger_price").is_some() { format!(" stop {}", g("trigger_price")) } else { String::new() };
            let reason = if g("reason").is_empty() { String::new() } else { format!(" — {}", g("reason")) };
            format!("{} {} {} {}{px}{reason}", g("event"), g("side"), qty, g("order_type").to_lowercase().replace('_', " "))
        }
        "fill" => format!("{} {} @ {} · fee {}", g("side"), qty, g("price"), g("commission")),
        "position" => {
            let side = if g("side") == "FLAT" { g("entry").replace("BUY", "LONG").replace("SELL", "SHORT") } else { g("side") };
            let pnl = if g("event") == "Closed" { format!(" · P&L {}", money(jf(&d, &["realized_pnl"]))) } else { String::new() };
            format!("{} {} {} @ {}{pnl}", g("event"), side, qty_fmt(jf(&d, &["peak_qty"]).max(jf(&d, &["qty"]))), g("avg_px_open"))
        }
        "node" => format!("Node {}", g("event")),
        "risk" => match g("event").as_str() {
            "blocked" => format!("Entry blocked · {} {} — {}", g("side"), qty, g("reason")),
            "trading_state" => format!("Trading state → {}", g("state")),
            "kill" => format!("KILL SWITCH — orders cancelled, {} position(s) closing", g("positions_closing")),
            "halt" => format!("Halted — {}", g("reason")),
            "resume" => "Trading allowed again".to_string(),
            "stale" => g("reason"),
            e => format!("{e} {}", g("reason")),
        },
        _ => String::new(),
    }
}

fn event_tag(t: &str) -> (&'static str, &'static str) {
    match t { "signal" => ("tag lav xs", "Signal"), "order" => ("tag sky xs", "Order"), "fill" => ("tag mint xs", "Fill"), "position" => ("tag peach xs", "Position"), "risk" => ("tag rose xs", "Risk"), _ => ("tag xs", "Node") }
}

fn soak_val(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "—".into(),
        serde_json::Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if n.is_i64() { format!("{}", n) } else if f.abs() <= 1.0 && f.fract() != 0.0 && f >= 0.0 { format!("{:.3}", f) } else { format!("{:.2}", f) }
        }
        other => other.to_string().trim_matches('"').to_string(),
    }
}

fn soak_pill(v: &str) -> &'static str {
    match v { "GO" => "pill ok", "NO-GO" => "pill bad", _ => "pill" }
}

fn opt_fmt(v: Option<f64>, dec: usize, suffix: &str) -> String {
    v.map(|x| format!("{:.*}{suffix}", dec, x)).unwrap_or_else(|| "—".into())
}

// ---------------- portfolio risk (step 16) ----------------
const PF_FIELDS: [(&str, &str, &str); 9] = [
    ("max_currency_pct", "Max exposure per currency", "% of NetLiq"), ("max_cluster_pct", "Max correlated group", "% of NetLiq"),
    ("max_gross_x", "Max gross FX exposure", "× NetLiq"), ("max_var_pct", "Max 1-day VaR (95%)", "% of NetLiq"),
    ("max_dd_pct", "Max portfolio drawdown", "% from peak → reduce-only"), ("min_excess_liq_pct", "Min excess liquidity", "% of NetLiq → reduce-only"),
    ("alert_pct", "Alert at", "% of a limit"), ("corr_threshold", "Correlation for groups", "|corr| ≥"), ("lookback_days", "History", "days"),
];

fn pf_pill(u: Option<f64>, alert: f64) -> &'static str {
    match u { Some(x) if x >= 100.0 => "pill bad", Some(x) if x >= alert => "pill warn", Some(_) => "pill ok", None => "pill" }
}

#[component]
fn PortfolioPanel() -> Element {
    let mut st = app_state();
    let mut data = use_signal(|| serde_json::Value::Null);
    let mut tick = use_signal(|| 0u32);
    let mut editing = use_signal(|| false);
    let mut form = use_signal(std::collections::BTreeMap::<String, String>::new);
    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            if let Ok(t) = get_portfolio_risk().await { data.set(serde_json::from_str(&t).unwrap_or_default()); }
        });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop { crate::sleep_ms(30000).await; if !*editing.peek() { tick += 1; } }
    });
    let d = data.read().clone();
    if d.is_null() { return rsx! {}; }
    let avail = d.get("available").and_then(|x| x.as_bool()).unwrap_or(false);
    let lim = d.get("limits").cloned().unwrap_or_default();
    let alert = jf(&lim, &["alert_pct"]);
    let m = d.get("metrics").cloned().unwrap_or_default();
    let lu = d.get("limits_usd").cloned().unwrap_or_default();
    let util = d.get("utilization").cloned().unwrap_or_default();
    let u = |k: &str| util.get(k).and_then(|x| x.as_f64());
    let dd = jf(&d, &["drawdown", "dd_pct"]);
    let dd_u = if jf(&lim, &["max_dd_pct"]) > 0.0 { Some(dd / jf(&lim, &["max_dd_pct"]) * 100.0) } else { None };
    let ex = jf(&d, &["excess_pct"]);
    let min_ex = jf(&lim, &["min_excess_liq_pct"]);
    let ex_cls = if ex < min_ex { "pill bad" } else if ex < min_ex * 1.25 { "pill warn" } else { "pill ok" };
    let var_txt = m.get("var").and_then(|x| x.as_f64()).map(money).unwrap_or_else(|| "n/a".into());
    let cur_lim = jf(&lu, &["currency"]).max(1.0);
    let exps: Vec<(String, f64)> = {
        let mut v: Vec<(String, f64)> = d.get("exposure_all").and_then(|e| e.as_object())
            .map(|o| o.iter().map(|(k, x)| (k.clone(), x.as_f64().unwrap_or(0.0))).filter(|(_, x)| x.abs() >= 1.0).collect()).unwrap_or_default();
        v.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap_or(std::cmp::Ordering::Equal));
        v
    };
    let clusters: Vec<(String, f64)> = m.get("clusters").and_then(|c| c.as_array())
        .map(|a| a.iter().map(|c| (c.get("members").and_then(|x| x.as_array()).map(|x| x.iter().filter_map(|y| y.as_str()).collect::<Vec<_>>().join(" + ")).unwrap_or_default(),
                                   jf(c, &["exposure"]))).collect()).unwrap_or_default();
    let asof = jdate(&js(&d, &["stats", "asof"]));
    let unpriced: Vec<String> = d.get("unpriced").and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let start_edit = move |_| {
        let l = data.peek().get("limits").cloned().unwrap_or_default();
        let mut f = std::collections::BTreeMap::new();
        for (k, _, _) in PF_FIELDS.iter() { f.insert(k.to_string(), jnum(&l, k)); }
        form.set(f);
        editing.set(true);
    };
    let save = move |_| {
        let mut body = serde_json::Map::new();
        for (k, v) in form.peek().iter() { if let Ok(x) = v.trim().parse::<f64>() { body.insert(k.clone(), serde_json::json!(x)); } }
        spawn(async move {
            match put_portfolio_limits(serde_json::Value::Object(body).to_string()).await {
                Ok(_) => { st.toast.set(Some((true, "Portfolio limits saved".into()))); editing.set(false); tick += 1; }
                Err(e) => st.toast.set(Some((false, format!("Save failed — {e}")))),
            }
        });
    };
    rsx! {
        div { class: "pkg-pf",
            div { class: "pkg-sys-row",
                span { class: "recon-title", "Portfolio risk" }
                if !avail { span { class: "muted small", "{js(&d, &[\"reason\"])}" } } else {
                    span { class: "muted small", "NetLiq {money(jf(&d, &[\"netliq\"]))} · correlations {js(&d, &[\"stats\", \"days\"])} days to {asof}" }
                    span { class: "{pf_pill(u(\"currency\"), alert)}", title: "largest single-currency exposure (USD = −sum of the others)",
                        "{js(&m, &[\"currency_max\"])} {money(jf(&m, &[\"currency\"]))} / {money(jf(&lu, &[\"currency\"]))}" }
                    span { class: "{pf_pill(u(\"cluster\"), alert)}", title: "largest correlated group", "Group {money(jf(&m, &[\"cluster\"]))} / {money(jf(&lu, &[\"cluster\"]))}" }
                    span { class: "{pf_pill(u(\"gross\"), alert)}", title: "sum of |exposure| of non-USD currencies", "Gross {money(jf(&m, &[\"gross\"]))} / {money(jf(&lu, &[\"gross\"]))}" }
                    span { class: "{pf_pill(u(\"var\"), alert)}", title: "1-day 95% VaR, historical simulation", "VaR {var_txt} / {money(jf(&lu, &[\"var\"]))}" }
                    span { class: "{pf_pill(dd_u, alert)}", title: "from the equity peak · over the limit = all nodes reduce-only", "Drawdown {dd:.2}% / {jf(&lim, &[\"max_dd_pct\"])}%" }
                    span { class: "{ex_cls}", title: "IBKR excess liquidity · under the minimum = reduce-only", "Excess liq {ex:.0}% (min {min_ex}%)" }
                }
                button { class: "mini", style: "margin-left:auto", onclick: start_edit, "Limits" }
            }
            if avail && !exps.is_empty() {
                div { class: "pkg-pf-bars",
                    for (c, v) in exps.iter() {
                        {
                            let w = format!("{:.0}%", (v.abs() / cur_lim * 100.0).min(100.0));
                            let cls = if *v >= 0.0 { "pkg-pf-fill up" } else { "pkg-pf-fill down" };
                            rsx! { div { class: "pkg-pf-bar",
                                span { class: "pkg-pf-ccy", "{c}" }
                                div { class: "pkg-pf-track", div { class: "{cls}", style: "width:{w}" } }
                                span { class: "num small {pnl_class(*v)}", "{money(*v)}" }
                            } }
                        }
                    }
                }
            }
            if avail && exps.is_empty() { div { class: "muted small", "No foreign-currency exposure." } }
            if !clusters.is_empty() {
                div { class: "muted small", "Correlated groups: " for (g, v) in clusters.iter() { span { class: "pkg-pf-grp", "{g} {money(*v)}" } } }
            }
            if !unpriced.is_empty() { div { class: "small text-down", "No price for {unpriced.join(\", \")} — not counted" } }
            if *editing.read() {
                div { class: "node-form-grid pkg-pf-form",
                    for (k, label, unit) in PF_FIELDS.iter() {
                        {
                            let key = k.to_string();
                            let val = form.read().get(&key).cloned().unwrap_or_default();
                            rsx! { label { title: "{unit}", span { "{label} ({unit})" }
                                input { class: "num", value: "{val}", oninput: move |e| { form.write().insert(key.clone(), e.value()); } } } }
                        }
                    }
                }
                div { class: "pkg-ramp-btns",
                    button { class: "mini accent", onclick: save, "Save limits" }
                    button { class: "mini", onclick: move |_| editing.set(false), "Cancel" }
                    span { class: "muted small", "Admin only · saved for this account · live trading requires saved limits" }
                }
            }
        }
    }
}

// ---------------- system health: sleep guard, watchdog, backup, holds (step 15) ----------------
#[component]
fn SystemPanel() -> Element {
    let mut st = app_state();
    let mut data = use_signal(|| serde_json::Value::Null);
    let mut tick = use_signal(|| 0u32);
    let mut busy = use_signal(|| false);
    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            if let Ok(t) = get_system_health().await { data.set(serde_json::from_str(&t).unwrap_or_default()); }
        });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop { crate::sleep_ms(30000).await; tick += 1; }
    });
    let d = data.read().clone();
    if d.is_null() { return rsx! {}; }
    let pw = d.get("power").cloned().unwrap_or_default();
    let supported = pw.get("supported").and_then(|x| x.as_bool()).unwrap_or(false);
    let awake = pw.get("active").and_then(|x| x.as_bool()).unwrap_or(false);
    let running = d.get("nodes_running").and_then(|x| x.as_bool()).unwrap_or(false);
    let will_sleep = pw.get("pmset").and_then(|p| p.get("will_sleep")).and_then(|x| x.as_bool()).unwrap_or(false);
    let hint = js(&pw, &["pmset", "hint"]);
    let (pw_cls, pw_txt) = if !supported { ("pill", "Sleep guard: not on this OS".to_string()) }
        else if awake { ("pill ok", "Keeping the Mac awake".to_string()) }
        else if running { ("pill bad", "Nodes running but sleep guard is off".to_string()) }
        else { ("pill", "Sleep guard idle (no nodes running)".to_string()) };
    let events: Vec<serde_json::Value> = d.get("events").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    let last_ev = events.last().cloned();
    let wd = d.get("watchdog").cloned().unwrap_or(serde_json::Value::Null);
    let wd_age = wd.get("age_s").and_then(|x| x.as_i64());
    let wd_bad: Vec<String> = wd.get("checks").and_then(|c| c.as_object())
        .map(|o| o.iter().filter(|(_, v)| v.as_bool() == Some(false)).map(|(k, _)| k.clone()).collect()).unwrap_or_default();
    let (wd_cls, wd_txt) = match wd_age {
        None => ("pill bad", "Watchdog not installed".to_string()),
        Some(a) if a > 180 => ("pill bad", format!("Watchdog silent for {} min", a / 60)),
        Some(_) if !wd_bad.is_empty() => ("pill warn", format!("Watchdog: {} down", wd_bad.join(", "))),
        Some(a) => ("pill ok", format!("Watchdog OK · checked {a}s ago")),
    };
    let bk = d.get("backup").cloned().unwrap_or_default();
    let last_bk = bk.get("last").cloned().filter(|x| !x.is_null());
    let bk_err = js(&bk, &["last_error"]);
    let bk_running = bk.get("running").and_then(|x| x.as_bool()).unwrap_or(false);
    let bk_txt = match &last_bk {
        Some(b) => format!("Last backup {} · {:.1} MB", jdate(&js(b, &["created"])), jf(b, &["bytes"]) / 1e6),
        None => "No backup yet".to_string(),
    };
    let holds: Vec<(String, String)> = d.get("holds").and_then(|h| h.as_object())
        .map(|o| o.iter().map(|(k, v)| (k.clone(), js(v, &["reason"]))).collect()).unwrap_or_default();
    rsx! {
        div { class: "pkg-sys",
            div { class: "pkg-sys-row",
                span { class: "recon-title", "System" }
                span { class: "{pw_cls}", title: "macOS power assertion (caffeinate) while nodes run", "{pw_txt}" }
                if will_sleep { span { class: "pill warn", title: "{hint}", "Mac may sleep on power adapter — run: {hint}" } }
                span { class: "{wd_cls}", title: "scripts/install_launchd.sh installs it", "{wd_txt}" }
                span { class: if last_bk.is_some() && bk_err.is_empty() { "pill" } else { "pill warn" }, title: "{js(&bk, &[\"settings\", \"dir\"])}", "{bk_txt}" }
                if !bk_err.is_empty() { span { class: "text-down small", "Backup failed: {bk_err}" } }
                button { class: "mini", disabled: bk_running || *busy.read(), onclick: move |_| {
                    busy.set(true);
                    spawn(async move {
                        match run_backup().await {
                            Ok(t) => {
                                let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                                st.toast.set(Some((true, format!("Backup saved: {}", js(&v, &["file"])))));
                            }
                            Err(e) => st.toast.set(Some((false, format!("Backup failed — {e}")))),
                        }
                        busy.set(false);
                        tick += 1;
                    });
                }, if bk_running { "Backing up…" } else { "Back up now" } }
                if let Some(e) = last_ev.clone() {
                    span { class: "muted small", title: "{js(&e, &[\"from\"])} → {js(&e, &[\"to\"])}",
                        "Last gap: {js(&e, &[\"kind\"])} {jf(&e, &[\"minutes\"]):.0} min ({jdate(&js(&e, &[\"to\"]))})" }
                }
            }
            for (node, reason) in holds {
                {
                    let n2 = node.clone();
                    rsx! {
                        div { class: "pkg-sys-hold",
                            span { class: "pill bad", "On hold" }
                            span { class: "small", "{node}: {reason}" }
                            span { class: "muted small", "New entries paused. Check the position in IBKR, then release." }
                            button { class: "mini accent", onclick: move |_| {
                                let id = n2.clone();
                                spawn(async move {
                                    match release_hold(id.clone()).await {
                                        Ok(_) => st.toast.set(Some((true, format!("{id}: released")))),
                                        Err(e) => st.toast.set(Some((false, format!("{id}: {e}")))),
                                    }
                                    tick += 1;
                                });
                            }, "Positions OK — release" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SoakPanel() -> Element {
    let mut st = app_state();
    let mut data = use_signal(SoakResp::default);
    let mut sel = use_signal(String::new);
    let mut detail = use_signal(|| None::<SoakNode>);
    let mut busy = use_signal(String::new);
    let mut confirm_reset = use_signal(String::new);
    let mut tick = use_signal(|| 0u32);

    use_effect(move || {
        let _ = tick.read();
        let id = sel.read().clone();
        spawn(async move {
            if let Ok(d) = get_soak().await { data.set(d); }
            if !id.is_empty() { if let Ok(x) = get_soak_detail(id).await { detail.set(Some(x)); } } else { detail.set(None); }
        });
    });
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop { crate::sleep_ms(30000).await; tick += 1; }
    });

    let mut run_parity = move |id: String| {
        busy.set(id.clone());
        spawn(async move {
            match soak_parity(id.clone()).await {
                Ok(v) => {
                    let m = v.get("live_vs_backtest").and_then(|x| x.get("match_rate")).and_then(|x| x.as_f64());
                    st.toast.set(Some((true, format!("{id}: parity {}", m.map(|x| format!("{:.1}% match", x * 100.0)).unwrap_or_else(|| "done, no actions yet".into())))));
                }
                Err(e) => st.toast.set(Some((false, format!("{id}: parity failed — {e}")))),
            }
            busy.set(String::new());
            tick += 1;
        });
    };

    let d = data.read().clone();
    let busy_id = busy.read().clone();
    let sel_id = sel.read().clone();
    let det = detail.read().clone();
    let min_days = d.criteria.get("min_days").and_then(|x| x.as_i64()).unwrap_or(5);

    rsx! {
        div { class: "soak",
            div { class: "perf-bar",
                span { class: "panel-title", style: "margin:0", "Paper soak" }
                span { class: "muted small", "Go/No-go per node before cutover · parity runs daily after the catalog build" }
                div { class: "tabs-right", button { class: "mini", title: "Reload", onclick: move |_| tick += 1, "↻" } }
            }
            table { class: "grid-tbl soak-tbl",
                thead { tr {
                    th { "Node" } th { "Verdict" } th { class: "r", "Days" } th { class: "r", "Decisions" } th { class: "r", "Slippage" }
                    th { class: "r", "p95 latency" } th { class: "r", "Rejects" } th { class: "r", "Crashes" } th { class: "r", "Gaps" }
                    th { class: "r", "Recon" } th { "Parity" } th { class: "r", "" }
                } }
                tbody {
                    for n in d.nodes.clone() {
                        {
                            let t = n.totals.clone();
                            let ti = move |k: &str| t.get(k).and_then(|x| x.as_i64()).unwrap_or(0);
                            let (rej, cr, gaps, rec) = (ti("rejects"), ti("crashes"), ti("gaps"), ti("reconcile"));
                            let dm = n.decision_match.map(|x| format!("{:.1}%", x * 100.0)).unwrap_or_else(|| "—".into());
                            let ptxt = n.parity_ts.clone().map(|x| jdate(&x)).unwrap_or_else(|| "not run".into());
                            let since = n.since.clone().map(|x| format!("since {}", jdate(&x))).unwrap_or_else(|| "no data yet".into());
                            let running = busy_id == n.node || js(&d.jobs, &[n.node.as_str()]) == "running";
                            let (id1, id2, id3, id4) = (n.node.clone(), n.node.clone(), n.node.clone(), n.node.clone());
                            let row_cls = if sel_id == n.node { "sel" } else { "" };
                            let resetting = *confirm_reset.read() == n.node;
                            rsx! {
                                tr { class: "{row_cls}",
                                    td { div { class: "sym", "{n.node}" } div { class: "muted small", "{since}" } }
                                    td { span { class: "{soak_pill(&n.verdict)}", "{n.verdict}" } }
                                    td { class: "r num", "{n.days} / {min_days}" }
                                    td { class: "r num", "{dm}" }
                                    td { class: "r num", "{opt_fmt(n.avg_slippage_pips, 2, \" pip\")}" }
                                    td { class: "r num", "{opt_fmt(n.p95_latency_s, 2, \" s\")}" }
                                    td { class: if rej > 0 { "r num text-down" } else { "r num" }, "{rej}" }
                                    td { class: if cr > 0 { "r num text-down" } else { "r num" }, "{cr}" }
                                    td { class: "r num", "{gaps}" }
                                    td { class: if rec > 0 { "r num text-down" } else { "r num" }, "{rec}" }
                                    td { class: "muted small", "{ptxt}" }
                                    td { class: "r actions",
                                        button { class: "mini", onclick: move |_| { if *sel.peek() == id1 { sel.set(String::new()) } else { sel.set(id1.clone()) } }, if sel_id == n.node { "Hide" } else { "Details" } }
                                        button { class: "mini accent", disabled: running, title: "Compare live decisions with a backtest on catalog bars", onclick: move |_| run_parity(id2.clone()),
                                            if running { "Running…" } else { "Run parity" } }
                                        if resetting {
                                            button { class: "mini danger", onclick: move |_| {
                                                let id = id3.clone();
                                                confirm_reset.set(String::new());
                                                spawn(async move {
                                                    match soak_reset(id.clone()).await {
                                                        Ok(_) => st.toast.set(Some((true, format!("{id}: soak restarted")))),
                                                        Err(e) => st.toast.set(Some((false, format!("Reset failed: {e}")))),
                                                    }
                                                    tick += 1;
                                                });
                                            }, "Confirm reset" }
                                        } else {
                                            button { class: "mini", title: "Start counting the soak again from now", onclick: move |_| confirm_reset.set(id4.clone()), "Reset" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(x) = det.filter(|x| x.node == sel_id) { SoakDetailView { x } }
        }
    }
}

#[component]
fn SoakDetailView(x: SoakNode) -> Element {
    let lvb = x.parity.get("live_vs_backtest").cloned().unwrap_or(serde_json::Value::Null);
    let shadow = x.parity.get("shadow").cloned().unwrap_or(serde_json::Value::Null);
    let divs: Vec<serde_json::Value> = lvb.get("divergence_rows").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    let shadow_txt = if shadow.is_null() { "no legacy agent for this strategy".to_string() } else if let Some(e) = shadow.get("error").and_then(|e| e.as_str()) { format!("error: {e}") } else {
        format!("{} · entries {} matched of {} live / {} legacy ({})", js(&shadow, &["agent"]), soak_val(&shadow["matched"]), soak_val(&shadow["live_actions"]),
                soak_val(&shadow["expected_actions"]), shadow.get("match_rate").and_then(|m| m.as_f64()).map(|m| format!("{:.1}%", m * 100.0)).unwrap_or_else(|| "—".into()))
    };
    let par_txt = if x.parity.is_null() { "Parity not run yet".to_string() } else {
        format!("{} → {} · {} bars · offset {} s · backtest P&L {}", jdate(&js(&x.parity, &["from"])), jdate(&js(&x.parity, &["to"])), soak_val(&x.parity["bars"]),
                soak_val(&lvb["median_offset_s"]), soak_val(&x.parity["backtest_pnl_usd"]))
    };
    rsx! {
        div { class: "soak-detail",
            div { class: "soak-checks",
                for c in x.checks.clone() {
                    div { class: "soak-check {c.status}",
                        span { class: "soak-ic", if c.status == "pass" { "✓" } else if c.status == "fail" { "✕" } else { "…" } }
                        span { class: "soak-name", "{c.name}" }
                        span { class: "num", "{soak_val(&c.value)}" }
                        span { class: "muted small", "target {c.target}" }
                        if !c.detail.is_empty() { span { class: "muted small", "· {c.detail}" } }
                    }
                }
            }
            div { class: "soak-cols",
                div {
                    h4 { "Daily" }
                    if x.daily.is_empty() { p { class: "empty small", "—" } } else {
                        table { class: "grid-tbl compact",
                            thead { tr { th { "Day" } th { class: "r", "Up (min)" } th { class: "r", "Signals" } th { class: "r", "Fills" } th { class: "r", "Slip" } th { class: "r", "p95 lat" } th { class: "r", "Rej / Denied" } th { class: "r", "Gaps" } th { class: "r", "Crash" } th { class: "r", "Recon" } } }
                            tbody { for r in x.daily.clone().into_iter().rev() { tr {
                                td { class: "num", "{r.day}" } td { class: "r num", "{r.alive_min}" } td { class: "r num", "{r.actions} / {r.signals}" } td { class: "r num", "{r.fills}" }
                                td { class: "r num", "{opt_fmt(r.avg_slippage_pips, 2, \"\")}" } td { class: "r num", "{opt_fmt(r.p95_latency_s, 2, \"\")}" }
                                td { class: "r num", "{r.rejects} / {r.denied}" } td { class: "r num", "{r.gaps}" } td { class: "r num", "{r.crashes}" } td { class: "r num", "{r.reconcile}" }
                            } } }
                        }
                    }
                }
                div {
                    h4 { "Live vs backtest" }
                    p { class: "muted small", "{par_txt}" }
                    p { class: "small", "Shadow (legacy agent): {shadow_txt}" }
                    if !divs.is_empty() {
                        table { class: "grid-tbl compact",
                            thead { tr { th { "Bar (UTC)" } th { class: "r", "Backtest" } th { class: "r", "Live" } } }
                            tbody { for r in divs { tr {
                                td { class: "num", "{jdate(&js(&r, &[\"ts\"]))}" } td { class: "r num", "{soak_val(&r[\"expected\"])}" } td { class: "r num", "{soak_val(&r[\"live\"])}" }
                            } } }
                        }
                    }
                    h4 { "Incidents" }
                    if x.incidents.is_empty() { p { class: "empty small", "None" } } else {
                        for i in x.incidents.clone() {
                            div { class: "small soak-inc", span { class: "num muted", "{jdate(&js(&i, &[\"ts\"]))} " } span { class: "tag xs src-warn", "{js(&i, &[\"kind\"])}" } " {js(&i, &[\"detail\"])}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NodeEventsFeed(nodes: Vec<String>) -> Element {
    let mut st = app_state();
    let mut node_f = use_signal(String::new);
    let mut type_f = use_signal(String::new);
    let mut status = use_signal(|| None::<serde_json::Value>);
    use_future(move || async move {
        if let Ok(txt) = get_node_events(String::new(), String::new()).await {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                status.set(v.get("status").cloned());
                if st.node_events.peek().is_empty() {
                    st.node_events.set(v.get("events").and_then(|e| e.as_array()).cloned().unwrap_or_default());
                }
            }
        }
    });
    let nf = node_f.read().clone();
    let tf = type_f.read().clone();
    let evs: Vec<serde_json::Value> = st.node_events.read().iter()
        .filter(|e| nf.is_empty() || js(e, &["node"]) == nf)
        .filter(|e| tf.is_empty() || js(e, &["type"]) == tf || (tf == "flow" && matches!(js(e, &["type"]).as_str(), "order" | "fill")))
        .take(150).cloned().collect();
    let redis_ok = status.read().as_ref().and_then(|s| s.get("connected")).and_then(|x| x.as_bool());
    rsx! {
        div { class: "node-events",
            div { class: "perf-bar",
                span { class: "panel-title", style: "margin:0", "Node activity" }
                match redis_ok { Some(true) => rsx! { span { class: "pill ok", "Redis" } }, Some(false) => rsx! { span { class: "pill bad", title: "Start Redis: docker compose up -d (bot folder)", "Redis offline" } }, None => rsx! {} }
                div { class: "tfs",
                    for (t, lbl) in [("", "All"), ("signal", "Signals"), ("flow", "Order flow"), ("risk", "Risk")] {
                        button { class: if tf == t { "tf active" } else { "tf" }, onclick: move |_| type_f.set(t.to_string()), "{lbl}" }
                    }
                }
                Dropdown { class: "wl-market", value: "{nf}", onchange: move |v| node_f.set(v),
                    options: std::iter::once((String::new(), "All nodes".to_string())).chain(nodes.iter().map(|n| (n.to_string(), n.to_string()))).collect::<DdOpts>() }
                div { class: "tabs-right", button { class: "mini", title: "Clear the list (events stay in Redis)", onclick: move |_| st.node_events.set(vec![]), "Clear" } }
            }
            if evs.is_empty() { p { class: "empty small", "No activity yet — signals, order flow and risk events from running nodes appear here instantly. Account-wide positions, orders and fills are in their own tabs." } } else {
                div { class: "node-events-body",
                    table { class: "grid-tbl compact",
                        thead { tr { th { "Time" } th { "Node" } th { "Type" } th { "Instrument" } th { "Details" } } }
                        tbody { for e in evs {
                            {
                                let (tcls, tlbl) = event_tag(&js(&e, &["type"]));
                                let t = jdate(&js(&e, &["ts"])).chars().skip(11).collect::<String>();
                                let summary = event_summary(&e);
                                let bad = matches!(js(&e, &["data", "event"]).as_str(), "Rejected" | "Denied" | "blocked" | "kill" | "halt");
                                rsx! { tr {
                                    td { class: "num", "{t}" } td { "{js(&e, &[\"node\"])}" } td { span { class: "{tcls}", "{tlbl}" } }
                                    td { class: "sym", "{js(&e, &[\"instrument\"])}" } td { class: if bad { "text-down" } else { "" }, "{summary}" }
                                } }
                            }
                        } }
                    }
                }
            }
        }
    }
}

// ---------------- Strategies tab: config editor, backtest/parity/walk-forward, deploy ----------------
#[derive(Clone, PartialEq, Default)]
struct StratForm { strategy: String, strategy_id: String, instrument: String, tf: String, size: String, sl: String, tp: String, allow_short: bool, params: std::collections::BTreeMap<String, String> }

fn spec_params(specs: &serde_json::Value, strategy: &str) -> Vec<serde_json::Value> {
    specs.get(strategy).and_then(|s| s.get("params")).and_then(|p| p.as_array()).cloned().unwrap_or_default()
}
fn strat_form(c: &serde_json::Value, specs: &serde_json::Value) -> StratForm {
    let strategy = { let s = js(c, &["strategy"]); if s.is_empty() { "ema_cross".to_string() } else { s } };
    let mut params = std::collections::BTreeMap::new();
    for p in spec_params(specs, &strategy) {
        let k = js(&p, &["key"]);
        let v = if c.get(&k).is_some() { jnum(c, &k) } else { jnum(&p, "default") };
        let v = if v.is_empty() { js(c, &[k.as_str()]) } else { v };
        params.insert(k, v);
    }
    StratForm { strategy, strategy_id: js(c, &["strategy_id"]), instrument: js(c, &["instrument"]), tf: jnum(c, "tf_minutes"),
                size: jnum(c, "trade_size"), sl: jnum(c, "sl_pips"), tp: jnum(c, "tp_pips"),
                allow_short: c.get("allow_short").and_then(|b| b.as_bool()).unwrap_or(true), params }
}
fn strat_json(f: &StratForm, specs: &serde_json::Value) -> String {
    let n = |s: &str| s.trim().parse::<f64>().ok();
    let mut m = serde_json::Map::new();
    m.insert("strategy".into(), serde_json::json!(f.strategy));
    if !f.strategy_id.trim().is_empty() { m.insert("strategy_id".into(), serde_json::json!(f.strategy_id.trim())); }
    m.insert("instrument".into(), serde_json::json!(f.instrument));
    if let Some(x) = n(&f.tf) { m.insert("tf_minutes".into(), serde_json::json!(x as i64)); }
    if let Ok(Some(x)) = crate::calc::trade_size_str(&f.size) { m.insert("trade_size".into(), serde_json::json!(x)); }
    m.insert("sl_pips".into(), serde_json::json!(n(&f.sl).unwrap_or(0.0)));
    m.insert("tp_pips".into(), serde_json::json!(n(&f.tp).unwrap_or(0.0)));
    m.insert("allow_short".into(), serde_json::json!(f.allow_short));
    for p in spec_params(specs, &f.strategy) {
        let k = js(&p, &["key"]);
        let raw = f.params.get(&k).cloned().unwrap_or_default();
        let v = match js(&p, &["type"]).as_str() {
            "int" => n(&raw).map(|x| serde_json::json!(x as i64)),
            "float" => n(&raw).map(|x| serde_json::json!(x)),
            "bool" => Some(serde_json::json!(raw == "true")),
            _ => if raw.trim().is_empty() { None } else { Some(serde_json::json!(raw.trim())) },
        };
        if let Some(v) = v { m.insert(k, v); }
    }
    serde_json::Value::Object(m).to_string()
}
fn file_slug(s: &str) -> String {
    let mut out: String = s.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    while out.contains("__") { out = out.replace("__", "_"); }
    let out = out.trim_matches('_').trim_end_matches("_json").to_string();
    if out.is_empty() { String::new() } else { format!("{out}.json") }
}

// ---------------- Strategies tab: packages + lifecycle (step 21) ----------------
#[derive(Clone, PartialEq, Default)]
struct PkgForm { instrument: String, size: String, sl: String, tp: String, allow_short: bool, params: std::collections::BTreeMap<String, String> }

const PKG_STEPS: [(&str, &str); 5] = [("installed", "Installed"), ("backtested", "Backtested"), ("wf_passed", "WF passed"), ("soak_go", "Soak GO"), ("live", "Live")];

fn pkg_state_label(s: &str) -> &'static str {
    match s { "installed" => "installed", "backtested" => "backtested", "wf_passed" => "wf passed", "soak_go" => "soak GO", "live" => "live", _ => "—" }
}
fn pkg_badge_class(s: &str) -> String {
    let k = match s { "installed" | "backtested" | "wf_passed" | "soak_go" | "live" => s, _ => "none" };
    format!("pkg-badge {k}")
}
/// What must pass before soak GO (tooltip of the Allow live button)
fn pkg_needs(state: &str) -> String {
    match state {
        "installed" => "Needs: 90-day backtest net > 0 with drawdown under the cap → walk-forward positive in ≥ 60% of folds → soak GO".into(),
        "backtested" => "Needs: walk-forward positive in ≥ 60% of folds → soak GO".into(),
        "wf_passed" => "Needs: soak verdict GO on a node using this version".into(),
        "soak_go" | "live" => String::new(),
        _ => "Not evaluated yet".into(),
    }
}
fn pkg_params(p: &serde_json::Value) -> Vec<serde_json::Value> {
    p.get("manifest").and_then(|m| m.get("params")).and_then(|x| x.as_array()).cloned().unwrap_or_default()
}
fn pkg_instruments(p: &serde_json::Value, catalog: &[String]) -> Vec<String> {
    let m: Vec<String> = p.get("manifest").and_then(|m| m.get("instruments")).and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    if m.is_empty() { catalog.to_vec() } else { m.into_iter().filter(|i| catalog.contains(i)).collect() }
}
fn pkg_default(q: &serde_json::Value) -> String {
    match q.get("default") { Some(serde_json::Value::Bool(b)) => b.to_string(), Some(_) => { let n = jnum(q, "default"); if n.is_empty() { js(q, &["default"]) } else { n } }, None => String::new() }
}
fn pkg_form(p: &serde_json::Value, catalog: &[String]) -> PkgForm {
    let mut params = std::collections::BTreeMap::new();
    for q in pkg_params(p) { params.insert(js(&q, &["key"]), pkg_default(&q)); }
    PkgForm { instrument: pkg_instruments(p, catalog).first().cloned().unwrap_or_default(), size: "20000".into(), sl: "0".into(), tp: "0".into(),
              allow_short: true, params }
}
/// Only values that differ from the defaults → node overrides (the package is not modified)
fn pkg_overrides(f: &PkgForm, p: &serde_json::Value) -> serde_json::Value {
    let n = |s: &str| s.trim().parse::<f64>().ok();
    let mut m = serde_json::Map::new();
    if let Ok(Some(x)) = crate::calc::trade_size_str(&f.size) { if x != "20000" { m.insert("trade_size".into(), serde_json::json!(x)); } }
    if let Some(x) = n(&f.sl) { if x != 0.0 { m.insert("sl_pips".into(), serde_json::json!(x)); } }
    if let Some(x) = n(&f.tp) { if x != 0.0 { m.insert("tp_pips".into(), serde_json::json!(x)); } }
    if !f.allow_short { m.insert("allow_short".into(), serde_json::json!(false)); }
    for q in pkg_params(p) {
        let k = js(&q, &["key"]);
        let raw = f.params.get(&k).cloned().unwrap_or_default();
        if raw.trim() == pkg_default(&q) { continue; }
        let v = match js(&q, &["type"]).as_str() {
            "int" => n(&raw).map(|x| serde_json::json!(x as i64)),
            "float" => n(&raw).map(|x| serde_json::json!(x)),
            "bool" => Some(serde_json::json!(raw.trim() == "true")),
            _ => if raw.trim().is_empty() { None } else { Some(serde_json::json!(raw.trim())) },
        };
        if let Some(v) = v { m.insert(k, v); }
    }
    serde_json::Value::Object(m)
}
fn pkg_result_line(lc: &serde_json::Value) -> String {
    let bt = lc.get("backtest").filter(|x| !x.is_null());
    let wf = lc.get("walkforward").filter(|x| !x.is_null());
    let mut parts = vec![];
    if let Some(b) = bt { parts.push(format!("BT {} · DD {:.1}%", money(jf(b, &["pnl_usd"])), jf(b, &["max_drawdown_pct"]))); }
    if let Some(w) = wf {
        if w.get("folds").is_some() { parts.push(format!("WF {}/{} {}", js(w, &["positive_folds"]), js(w, &["folds"]), money(jf(w, &["pnl_usd"])))); }
        else if !js(w, &["error"]).is_empty() { parts.push("WF error".into()); }
    }
    if parts.is_empty() { if lc.get("evaluating").and_then(|x| x.as_bool()).unwrap_or(false) { "evaluating…".into() } else { "not evaluated".into() } } else { parts.join(" · ") }
}

#[component]
fn StratModeSwitch(mode: Signal<&'static str>) -> Element {
    let m = *mode.read();
    rsx! {
        div { class: "tfs pkg-mode",
            button { class: if m == "packages" { "tf active" } else { "tf" }, title: "Strategy packages (name@version) and their lifecycle", onclick: move |_| mode.set("packages"), "Packages" }
            button { class: if m == "configs" { "tf active" } else { "tf" }, title: "Config files used by existing nodes", onclick: move |_| mode.set("configs"), "Configs" }
            button { class: if m == "store" { "tf active" } else { "tf" }, title: "Agents listed on QOS Cloud", onclick: move |_| mode.set("store"), "Store" }
        }
    }
}

// ---------------- QOS Store (step 38) ----------------
fn store_spark(points: &[serde_json::Value]) -> (String, bool) {
    let vals: Vec<f64> = points.iter().filter_map(|p| p.as_array().and_then(|a| a.get(1)).and_then(|x| x.as_f64())).collect();
    if vals.len() < 2 { return (String::new(), true); }
    let (lo, hi) = vals.iter().fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    let span = if hi - lo < 1e-9 { 1.0 } else { hi - lo };
    let n = (vals.len() - 1) as f64;
    let pts = vals.iter().enumerate().map(|(i, v)| format!("{:.1},{:.1}", i as f64 / n * 300.0, 58.0 - (v - lo) / span * 54.0)).collect::<Vec<_>>().join(" ");
    (pts, vals.last() >= vals.first())
}

fn price_label(p: &serde_json::Value) -> String {
    let per = match js(p, &["kind"]).as_str() { "month" => " / month", "year" => " / year", _ => "" };
    format!("{:.2} {}{}", jf(p, &["amount_cents"]) / 100.0, js(p, &["currency"]).to_uppercase(), per)
}

fn store_eval_lines(a: &serde_json::Value) -> (String, String, String) {
    let bt = a.get("evaluation").and_then(|e| e.get("backtest")).cloned().unwrap_or_default();
    let wf = a.get("evaluation").and_then(|e| e.get("walkforward")).cloned().unwrap_or_default();
    let fw = a.get("forward").and_then(|f| f.get("stats")).cloned().unwrap_or_default();
    let b = if bt.is_null() { String::new() } else { format!("Backtest {}d · {} · {} trades · DD {}%", js(&bt, &["days"]), money(jf(&bt, &["pnl_usd"])), js(&bt, &["trades"]), js(&bt, &["max_drawdown_pct"])) };
    let w = if wf.is_null() { String::new() } else { format!("Walk-forward {}/{} folds positive", js(&wf, &["positive_folds"]), js(&wf, &["folds"])) };
    let f = if fw.is_null() || fw.get("trades").is_none() { String::new() } else { format!("Forward {}d · {} · {} trades · DD {}%", js(&fw, &["days"]), money(jf(&fw, &["pnl_usd"])), js(&fw, &["trades"]), js(&fw, &["max_drawdown_pct"])) };
    (b, w, f)
}

#[component]
fn StoreView(mode: Signal<&'static str>) -> Element {
    let mut st = app_state();
    let mut data = use_signal(|| serde_json::Value::Null);
    let mut err = use_signal(String::new);
    let mut sel = use_signal(String::new);
    let mut detail = use_signal(|| serde_json::Value::Null);
    let mut busy = use_signal(String::new);
    let mut tick = use_signal(|| 0u32);
    let mut pending = use_signal(String::new);
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop {
            crate::sleep_ms(5000).await;
            if !pending.peek().is_empty() { tick += 1; }
        }
    });
    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            match get_store().await {
                Ok(t) => { data.set(serde_json::from_str(&t).unwrap_or_default()); err.set(String::new()); }
                Err(e) => err.set(e.to_string()),
            }
        });
    });
    use_effect(move || {
        let s = sel.read().clone();
        if s.is_empty() { return; }
        spawn(async move {
            detail.set(serde_json::Value::Null);
            if let Ok(t) = get_store_detail(s).await { detail.set(serde_json::from_str(&t).unwrap_or_default()); }
        });
    });
    let d = data.read().clone();
    let agents: Vec<serde_json::Value> = d.get("agents").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let cloud_err = js(&d, &["error"]);
    let my_platform = js(&d, &["platform"]);
    let cur_slug = sel.read().clone();
    let cur = agents.iter().find(|a| js(a, &["slug"]) == cur_slug).cloned().unwrap_or(serde_json::Value::Null);
    let det = detail.read().clone();
    let busy_now = busy.read().clone();
    use_effect(move || {
        let p = pending.read().clone();
        if p.is_empty() { return; }
        let owned = data.read().get("agents").and_then(|a| a.as_array())
            .and_then(|a| a.iter().find(|x| js(x, &["slug"]) == p).cloned())
            .and_then(|x| x.get("owned").and_then(|o| o.as_bool())).unwrap_or(false);
        if owned { pending.set(String::new()); st.toast.set(Some((true, format!("{p}: purchase complete — you can install it now")))); }
    });
    let mut buy = move |slug: String, price_id: String| {
        busy.set(format!("buy:{slug}"));
        spawn(async move {
            match billing_action("checkout".into(), price_id).await {
                Ok(_) => { pending.set(slug.clone()); st.toast.set(Some((true, "Complete the payment in your browser — access appears here automatically".to_string()))); }
                Err(e) => st.toast.set(Some((false, format!("Buy {slug}: {e}")))),
            }
            busy.set(String::new());
        });
    };
    let mut install = move |slug: String, version: String| {
        busy.set(slug.clone());
        spawn(async move {
            match store_install(slug.clone(), version).await {
                Ok(t) => {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    st.toast.set(Some((true, format!("{}@{} downloaded and verified — installing (checks, then backtest + walk-forward in Packages)", slug, js(&v, &["version"])))));
                }
                Err(e) => st.toast.set(Some((false, format!("Install {slug}: {e}")))),
            }
            busy.set(String::new());
            tick += 1;
        });
    };
    rsx! {
        div { class: "bt-grid strat store",
            div { class: "bt-list",
                div { class: "bt-head",
                    StratModeSwitch { mode }
                    button { class: "mini", style: "margin-left:auto", title: "Reload", onclick: move |_| tick += 1, "↻" }
                }
                if !err.read().is_empty() { p { class: "empty small", "{err}" } }
                if !cloud_err.is_empty() { p { class: "small text-down", "QOS Cloud: {cloud_err} — showing installed agents only" } }
                if agents.is_empty() && err.read().is_empty() { p { class: "empty small", "No agents in the QOS Store yet. Sign in to QOS Cloud from the Cloud menu." } }
                for a in agents.iter().cloned() {
                    {
                        let slug = js(&a, &["slug"]);
                        let s2 = slug.clone();
                        let installed: Vec<String> = a.get("installed").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                        let update = a.get("update").and_then(|x| x.as_bool()).unwrap_or(false);
                        let revoked = a.get("revoked").and_then(|x| x.as_bool()).unwrap_or(false);
                        let unlisted = a.get("unlisted").and_then(|x| x.as_bool()).unwrap_or(false);
                        let (_, _, fwd) = store_eval_lines(&a);
                        let cls = if slug == cur_slug { "bt-item store-item active" } else { "bt-item store-item" };
                        rsx! {
                            div { class: "{cls}", onclick: move |_| sel.set(s2.clone()),
                                div { class: "bt-item-top",
                                    span { class: "sym", "{js(&a, &[\"title\"])}" }
                                    span { class: "muted small", "{slug}@{js(&a, &[\"version\"])}" }
                                }
                                div { class: "small store-badges",
                                    if revoked { span { class: "pill bad", "Withdrawn" } }
                                    else if unlisted { span { class: "pill warn", "No longer listed" } }
                                    if update { span { class: "pill warn", "Update" } }
                                    if !installed.is_empty() { span { class: "pill ok", "Installed {installed.join(\", \")}" } }
                                }
                                if !fwd.is_empty() { div { class: "muted small pkg-line", "{fwd}" } }
                            }
                        }
                    }
                }
            }
            div { class: "bt-detail",
                if cur.is_null() { p { class: "empty", "Select an agent" } } else {
                    {
                        let slug = js(&cur, &["slug"]);
                        let ver = js(&cur, &["version"]);
                        let (s1, v1) = (slug.clone(), ver.clone());
                        let installed: Vec<String> = cur.get("installed").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                        let have = installed.contains(&ver);
                        let revoked = cur.get("revoked").and_then(|x| x.as_bool()).unwrap_or(false);
                        let unlisted = cur.get("unlisted").and_then(|x| x.as_bool()).unwrap_or(false);
                        let meta = cur.get("meta").cloned().unwrap_or_default();
                        let insts: Vec<String> = meta.get("instruments").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                        let (bt, wf, fw) = store_eval_lines(&cur);
                        let fwd = det.clone();
                        let curve: Vec<serde_json::Value> = fwd.get("equity").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                        let (pts, up) = store_spark(&curve);
                        let trades: Vec<serde_json::Value> = fwd.get("recent_trades").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                        let is_busy = busy_now == slug;
                        let platforms: Vec<String> = cur.get("platforms").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                        let compiled = js(&cur, &["distribution"]) != "source";
                        let no_build = compiled && !platforms.contains(&my_platform);
                        let prices: Vec<serde_json::Value> = cur.get("prices").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                        let owned = cur.get("owned").and_then(|x| x.as_bool()).unwrap_or(false);
                        let must_buy = !prices.is_empty() && !owned;
                        let buying = busy_now == format!("buy:{slug}") || *pending.read() == slug;
                        rsx! {
                            div { class: "store-detail",
                                div { class: "bt-title",
                                    span { class: "panel-title", style: "margin:0", "{js(&cur, &[\"title\"])}" }
                                    span { class: "muted small", "{slug}@{ver}" }
                                }
                                if !js(&cur, &["description"]).is_empty() { p { class: "small", "{js(&cur, &[\"description\"])}" } }
                                if !prices.is_empty() {
                                    div { class: "small", if owned { span { class: "pill ok", "Purchased" } } else { span { class: "muted", "{prices.iter().map(price_label).collect::<Vec<_>>().join(\" · \")}" } } }
                                }
                                div { class: "muted small", "{insts.join(\" · \")}" if !js(&meta, &["timeframe"]).is_empty() { " · {js(&meta, &[\"timeframe\"])}" }
                                    if compiled { " · compiled build" } else { " · source" } }
                                div { class: "store-lines small",
                                    if !bt.is_empty() { div { "{bt}" } }
                                    if !wf.is_empty() { div { "{wf}" } }
                                    if !fw.is_empty() { div { "{fw}" } }
                                }
                                if !fwd.is_null() {
                                    div { class: "store-fwd",
                                        div { class: "small",
                                            span { class: if js(&fwd, &["status"]) == "live" { "pill ok" } else if js(&fwd, &["status"]) == "failed" { "pill bad" } else { "pill" }, "Forward paper: {js(&fwd, &[\"status\"])}" }
                                            span { class: "muted", " since {jday(&js(&fwd, &[\"started_at\"]))}" }
                                            if !js(&fwd, &["reason"]).is_empty() { span { class: "muted", " · {js(&fwd, &[\"reason\"])}" } }
                                        }
                                        if !pts.is_empty() {
                                            svg { class: if up { "store-spark up" } else { "store-spark down" }, view_box: "0 0 300 60", preserve_aspect_ratio: "none",
                                                polyline { points: "{pts}" }
                                            }
                                        }
                                        if !trades.is_empty() {
                                            table { class: "grid-tbl compact",
                                                thead { tr { th { "Closed" } th { "Side" } th { class: "r", "P&L" } } }
                                                tbody {
                                                    for t in trades.iter().rev().take(10) {
                                                        tr {
                                                            td { "{jdate(&js(t, &[\"closed\"]))}" }
                                                            td { "{js(t, &[\"side\"])}" }
                                                            td { class: if jf(t, &["pnl"]) >= 0.0 { "r num text-up" } else { "r num text-down" }, "{money(jf(t, &[\"pnl\"]))}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "cloud-btns",
                                    if revoked {
                                        span { class: "text-down small", "This version was withdrawn by QOS. Nodes using it are stopped." }
                                    } else if unlisted {
                                        span { class: "muted small", "No longer listed in the store." }
                                    } else if must_buy {
                                        for pr in prices.iter().cloned() {
                                            {
                                                let (s3, pid) = (slug.clone(), js(&pr, &["id"]));
                                                rsx! { button { class: "mini accent", disabled: buying, onclick: move |_| buy(s3.clone(), pid.clone()),
                                                    if buying { "Waiting for payment…" } else { "Buy · {price_label(&pr)}" } } }
                                            }
                                        }
                                    } else if have {
                                        span { class: "pill ok", "Installed — see Packages to backtest, soak and deploy" }
                                    } else if no_build {
                                        span { class: "pill warn", title: "Built for: {platforms.join(\", \")}", "Not built for this computer yet ({my_platform})" }
                                    } else {
                                        button { class: "mini accent", disabled: is_busy, onclick: move |_| install(s1.clone(), v1.clone()),
                                            if is_busy { "Downloading…" } else if installed.is_empty() { "Install" } else { "Install update" } }
                                    }
                                    button { class: "mini", onclick: move |_| mode.set("packages"), "Open Packages" }
                                }
                                p { class: "muted small", "Agents from the QOS Store are signed by QOS Cloud and checked before every node start. They install like any package: checks, then backtest + walk-forward, then paper soak before live. Results on this page come from QOS Cloud, not from your account." }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StrategiesTab() -> Element {
    let st = app_state();
    let mut mode = use_signal(|| "packages");
    use_effect(move || { if st.strat_pkg.read().is_some() { mode.set("packages"); } });
    let m = *mode.read();
    rsx! { if m == "packages" { PackagesView { mode } } else if m == "store" { StoreView { mode } } else { ConfigsView { mode } } }
}

#[component]
fn PackagesView(mode: Signal<&'static str>) -> Element {
    let mut st = app_state();
    let mut data = use_signal(|| serde_json::Value::Null);
    let mut err = use_signal(String::new);
    let mut tick = use_signal(|| 0u32);
    let mut sel = use_signal(String::new);
    let mut form = use_signal(PkgForm::default);
    let mut form_for = use_signal(String::new);
    let mut job = use_signal(|| serde_json::Value::Null);
    let mut days = use_signal(|| "90".to_string());
    let mut wf_days = use_signal(|| "365".to_string());
    let mut train = use_signal(|| "90".to_string());
    let mut test = use_signal(|| "30".to_string());
    let mut show_install = use_signal(|| false);
    let mut src = use_signal(String::new);
    let mut force = use_signal(|| false);
    let mut dep_name = use_signal(String::new);
    let mut dep_auto = use_signal(|| true);
    let mut dep_start = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut live_code = use_signal(String::new);

    // link from the agent detail page → select that package
    use_effect(move || {
        let want = st.strat_pkg.read().clone();
        if let Some(r) = want { sel.set(r); show_install.set(false); st.strat_pkg.set(None); }
    });
    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            match get_packages().await {
                Ok(t) => {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    if js(&job.peek(), &["status"]) != "running" {
                        if let Some(r) = v.get("running").filter(|r| !r.is_null()) { job.set(r.clone()); }
                    }
                    if sel.peek().is_empty() {
                        let first = v.get("packages").and_then(|a| a.as_array()).and_then(|a| a.iter().find(|p| js(p, &["kind"]) != "builtin").or(a.first()).cloned());
                        if let Some(p) = first { sel.set(js(&p, &["ref"])); }
                    }
                    data.set(v);
                    err.set(String::new());
                }
                Err(e) => err.set(e.to_string()),
            }
        });
    });
    // selected package changed → parameter form from the manifest (defaults)
    use_effect(move || {
        let r = sel.read().clone();
        let d = data.read().clone();
        if r.is_empty() || *form_for.peek() == r { return; }
        let catalog: Vec<String> = d.get("instruments").and_then(|c| c.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        if let Some(p) = d.get("packages").and_then(|a| a.as_array()).and_then(|a| a.iter().find(|p| js(p, &["ref"]) == r)) {
            form.set(pkg_form(p, &catalog));
            form_for.set(r.clone());
            let sym = pkg_instruments(p, &catalog).first().map(|i| i.split('.').next().unwrap_or_default().replace('/', "").to_lowercase()).unwrap_or_default();
            dep_name.set(format!("{}_{}", js(p, &["name"]), sym));
        }
    });
    // track jobs (install / evaluate) + refresh status during evaluation
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        let mut n = 0u32;
        loop {
            crate::sleep_ms(1500).await;
            n += 1;
            let j = job.peek().clone();
            let id = js(&j, &["id"]);
            if !id.is_empty() && js(&j, &["status"]) == "running" {
                if let Ok(t) = get_strategy_job(id).await {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    if js(&v, &["status"]) != "running" {
                        let what = if js(&v, &["kind"]) == "install" { format!("install {}", js(&v, &["source"])) } else { format!("evaluate {}", js(&v, &["package"])) };
                        st.toast.set(Some((js(&v, &["status"]) == "done", format!("{what} — {}", js(&v, &["status"])))));
                        if js(&v, &["kind"]) == "install" && js(&v, &["status"]) == "done" {
                            let r = js(&v, &["package"]);
                            if !r.is_empty() { sel.set(r); show_install.set(false); }
                        }
                        tick += 1;
                    }
                    job.set(v);
                }
            } else if n % 4 == 0 {
                let d = data.peek().clone();
                let waiting = d.get("queue").and_then(|q| q.as_array()).map(|q| !q.is_empty()).unwrap_or(false)
                    || d.get("packages").and_then(|a| a.as_array()).map(|a| a.iter().any(|p| p.get("lifecycle").and_then(|l| l.get("evaluating")).and_then(|x| x.as_bool()).unwrap_or(false))).unwrap_or(false);
                if waiting { tick += 1; }
            }
        }
    });

    let d = data.read().clone();
    let packages: Vec<serde_json::Value> = d.get("packages").and_then(|c| c.as_array()).cloned().unwrap_or_default();
    let catalog: Vec<String> = d.get("instruments").and_then(|c| c.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let live_gw = d.get("live_gateway").and_then(|x| x.as_bool()).unwrap_or(false);
    let queue: Vec<String> = d.get("queue").and_then(|q| q.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let cur_ref = sel.read().clone();
    let cur = packages.iter().find(|p| js(p, &["ref"]) == cur_ref).cloned().unwrap_or(serde_json::Value::Null);
    let j = job.read().clone();
    let running = js(&j, &["status"]) == "running";
    let f = form.read().clone();
    let busy_now = *busy.read();

    let run_eval = move |only: &'static str| {
        let r = sel.peek().clone();
        let p = |s: &Signal<String>| s.peek().trim().parse::<i64>().ok();
        let mut o = serde_json::json!({"only": only});
        if only == "backtest" { if let Some(x) = p(&days) { o["days"] = serde_json::json!(x); } }
        else {
            if let Some(x) = p(&wf_days) { o["wf_days"] = serde_json::json!(x); }
            if let Some(x) = p(&train) { o["train"] = serde_json::json!(x); }
            if let Some(x) = p(&test) { o["test"] = serde_json::json!(x); }
        }
        spawn(async move {
            match evaluate_package(r.clone(), o.to_string()).await {
                Ok(t) => {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    st.toast.set(Some((true, format!("{r}: {only} {}", js(&v, &["status"])))));
                    tick += 1;
                }
                Err(e) => st.toast.set(Some((false, format!("{r}: could not start — {e}")))),
            }
        });
    };
    let do_install = move |_| {
        let (s, fo) = (src.peek().trim().to_string(), *force.peek());
        busy.set(true);
        spawn(async move {
            match install_package(s.clone(), fo).await {
                Ok(t) => job.set(serde_json::from_str(&t).unwrap_or_default()),
                Err(e) => st.toast.set(Some((false, format!("Install failed — {e}")))),
            }
            busy.set(false);
        });
    };
    let do_deploy = move |_| {
        let r = sel.peek().clone();
        let dd = data.peek().clone();
        let p = dd.get("packages").and_then(|a| a.as_array()).and_then(|a| a.iter().find(|p| js(p, &["ref"]) == r)).cloned().unwrap_or_default();
        let fm = form.peek().clone();
        if let Err(e) = crate::calc::trade_size_str(&fm.size) { st.toast.set(Some((false, format!("Not deployed: {e}")))); return; }
        let body = serde_json::json!({"name": dep_name.peek().trim(), "instrument": fm.instrument, "overrides": pkg_overrides(&fm, &p),
                                      "auto_restart": *dep_auto.peek(), "start": *dep_start.peek()});
        busy.set(true);
        spawn(async move {
            match deploy_package(r.clone(), body.to_string()).await {
                Ok(t) => {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    st.toast.set(Some((true, format!("Node {} created from {r}", js(&v, &["node", "id"])))));
                    tick += 1;
                }
                Err(e) => st.toast.set(Some((false, format!("Deploy failed — {e}")))),
            }
            busy.set(false);
        });
    };
    let mut open_report = move |kind: String, file: String| { st.bt_open.set(Some((kind, file))); st.bottom_tab.set(9); };

    // ---- job panel (install / evaluate) ----
    let job_view = if !j.is_null() && (running || js(&j, &["package"]) == cur_ref || (js(&j, &["kind"]) == "install" && *show_install.read())) {
        let status = js(&j, &["status"]);
        let pill = match status.as_str() { "done" => "pill ok", "running" => "pill warn", _ => "pill bad" };
        let prog = j.get("progress").and_then(|p| p.as_f64());
        let width = format!("{:.0}%", prog.unwrap_or(0.0) * 100.0);
        let jid = js(&j, &["id"]);
        let what = if js(&j, &["kind"]) == "install" { format!("install · {}", js(&j, &["source"])) } else { { let o = js(&j, &["options", "only"]); format!("evaluate · {}{}", js(&j, &["package"]), if o.is_empty() { String::new() } else { format!(" · {o}") }) } };
        let log: Vec<String> = j.get("log").and_then(|l| l.as_array()).map(|a| a.iter().rev().take(14).rev().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let last = js(&j, &["last_line"]);
        rsx! {
            div { class: "strat-job",
                span { class: "{pill}", "{status}" }
                span { class: "small", "{what}" }
                if running {
                    div { class: if prog.is_some() { "strat-bar" } else { "strat-bar indet" }, div { class: "strat-bar-fill", style: "width:{width}" } }
                    button { class: "mini", onclick: move |_| { let id = jid.clone(); spawn(async move { let _ = cancel_strategy_job(id).await; }); }, "Cancel" }
                }
                if log.is_empty() { span { class: "muted small strat-last", "{last}" } }
            }
            if !log.is_empty() { pre { class: "pkg-log", for ln in log.iter() { "{ln}\n" } } }
        }
    } else { rsx! {} };

    rsx! {
        div { class: "bt-grid strat pkg",
            div { class: "bt-list",
                div { class: "bt-head",
                    StratModeSwitch { mode }
                    button { class: if *show_install.read() { "mini accent" } else { "mini" }, style: "margin-left:auto", title: "Install a strategy package (qos install)", onclick: move |_| { let v = !*show_install.peek(); show_install.set(v); }, "+ Install" }
                    button { class: "mini", title: "Reload", onclick: move |_| tick += 1, "↻" }
                }
                if !err.read().is_empty() { p { class: "empty small", "{err}" } }
                if !queue.is_empty() { div { class: "muted small pkg-queue", "Evaluation queue: {queue.join(\", \")}" } }
                for p in packages.iter().cloned() {
                    {
                        let r = js(&p, &["ref"]);
                        let ok = js(&p, &["status"]) == "ok";
                        let builtin = js(&p, &["kind"]) == "builtin";
                        let lc = p.get("lifecycle").cloned().unwrap_or(serde_json::Value::Null);
                        let state = js(&lc, &["state"]);
                        let nodes: Vec<String> = p.get("nodes").and_then(|u| u.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                        let active = r == cur_ref;
                        let cls = format!("bt-item pkg-item{}{}", if active { " active" } else { "" }, if ok { "" } else { " err" });
                        let res = pkg_result_line(&lc);
                        let r2 = r.clone();
                        let label = js(&p, &["manifest", "label"]);
                        let error = js(&p, &["error"]);
                        rsx! {
                            div { class: "{cls}", onclick: move |_| { sel.set(r2.clone()); show_install.set(false); },
                                div { class: "bt-item-top",
                                    span { class: "sym", "{r}" }
                                    if builtin { span { class: "tag lav xs", title: "Built into the bot (reference implementation)", "builtin" } }
                                }
                                div { class: "r small",
                                    if ok { span { class: "{pkg_badge_class(&state)}", title: "{js(&lc, &[\"note\"])}", "{pkg_state_label(&state)}" } }
                                    else { span { class: "pkg-badge err", "error" } }
                                }
                                if ok {
                                    div { class: "muted small pkg-line", title: "{label}", "{res}" }
                                    div { class: "small pkg-line", if nodes.is_empty() { span { class: "muted", "no nodes" } } else { span { class: "pkg-nodes", "▸ {nodes.join(\", \")}" } } }
                                } else {
                                    div { class: "small pkg-errtxt", title: "{error}", "{error}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "bt-detail",
                if *show_install.read() {
                    div { class: "node-form pkg-install",
                        div { class: "bt-title", span { class: "panel-title", style: "margin:0", "Install package" } span { class: "muted small", "Same checks as qos install: import · manifest · network · deterministic · look-ahead — then backtest + walk-forward run automatically" } }
                        div { class: "strat-run",
                            input { class: "pkg-src", placeholder: "~/dev/my_strategy  ·  …/strategy.py  ·  git+https://github.com/me/repo.git#subdir", value: "{src}", oninput: move |e| src.set(e.value()) }
                            label { class: "mini pkg-browse", title: "Choose a folder that contains strategy.py",
                                "Browse…"
                                input { r#type: "file", directory: true, onchange: move |e| {
                                    if let Some(fd) = e.files().first() {
                                        let mut path = fd.path();
                                        if path.extension().is_some() { if let Some(par) = path.parent() { path = par.to_path_buf(); } }
                                        src.set(path.display().to_string());
                                    }
                                } }
                            }
                            label { class: "chk", title: "Replace the same version if no node uses it", input { r#type: "checkbox", checked: *force.read(), oninput: move |e| force.set(e.checked()) } span { "Replace existing" } }
                            button { class: "mini accent", disabled: src.read().trim().is_empty() || running || busy_now, title: if running { "Another Strategy lab job is running" } else { "" }, onclick: do_install, "Install" }
                        }
                        if js(&j, &["kind"]) == "install" { {job_view.clone()} }
                    }
                }
                if cur.is_null() { if !*show_install.read() { p { class: "empty", "Select a package" } } } else {
                    {
                        let ok = js(&cur, &["status"]) == "ok";
                        let builtin = js(&cur, &["kind"]) == "builtin";
                        let lc = cur.get("lifecycle").cloned().unwrap_or(serde_json::Value::Null);
                        let state = js(&lc, &["state"]);
                        let note = js(&lc, &["note"]);
                        let man = cur.get("manifest").cloned().unwrap_or(serde_json::Value::Null);
                        let label = js(&man, &["label"]);
                        let desc = js(&man, &["description"]);
                        let step_i = PKG_STEPS.iter().position(|(k, _)| *k == state);
                        let params = pkg_params(&cur);
                        let grid = man.get("grid").cloned().unwrap_or(serde_json::Value::Null);
                        let insts = pkg_instruments(&cur, &catalog);
                        let tf = js(&man, &["timeframe"]);
                        let bt = lc.get("backtest").cloned().filter(|x| !x.is_null());
                        let wf = lc.get("walkforward").cloned().filter(|x| !x.is_null());
                        let evaluating = lc.get("evaluating").and_then(|x| x.as_bool()).unwrap_or(false);
                        let eval_err = js(&lc, &["eval_error"]);
                        let node_info: Vec<serde_json::Value> = cur.get("node_info").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                        let can_live = state == "soak_go" || state == "live";
                        let needs = pkg_needs(&state);
                        let eval_block = if !ok { "Package cannot be loaded" } else if builtin { "Builtin strategies are tested from their config files (Configs)" } else if running { "Another Strategy lab job is running" } else { "" };
                        let dep_block = if !ok { "Package cannot be loaded".to_string() } else if f.instrument.is_empty() { "No catalog data for the package's instruments".to_string() } else { String::new() };
                        let src_txt = js(&lc, &["source"]);
                        let path_txt = js(&cur, &["path"]);
                        rsx! {
                            // ---- package + lifecycle ----
                            div { class: "node-form",
                                div { class: "bt-title",
                                    span { class: "panel-title", style: "margin:0", "{cur_ref}" }
                                    if !label.is_empty() { span { class: "tag lav xs", title: "{desc}", "{label}" } }
                                    if ok { span { class: "{pkg_badge_class(&state)}", "{pkg_state_label(&state)}" } } else { span { class: "pkg-badge err", "error" } }
                                    if evaluating || queue.contains(&cur_ref) { span { class: "pill warn", "evaluating" } }
                                }
                                if !ok { div { class: "pkg-errbox", "{js(&cur, &[\"error\"])}" } }
                                if ok {
                                    div { class: "pkg-steps",
                                        for (i, (_k, lbl)) in PKG_STEPS.iter().enumerate() {
                                            span { class: if step_i.map(|s| i <= s).unwrap_or(false) { "pkg-step done" } else { "pkg-step" }, "{lbl}" }
                                        }
                                    }
                                    div { class: "muted small", "{note}" }
                                    if !eval_err.is_empty() { div { class: "small text-down", "Last evaluation failed: {eval_err}" } }
                                    div { class: "muted small pkg-meta",
                                        "{tf} · {insts.join(\", \")}"
                                        if !path_txt.is_empty() { " · {path_txt}" }
                                        if !src_txt.is_empty() { " · from {src_txt}" }
                                    }
                                }
                            }
                            // ---- evaluate ----
                            if ok {
                                div { class: "node-form",
                                    div { class: "bt-title", span { class: "panel-title", style: "margin:0", "Backtest & walk-forward" } span { class: "muted small", "Results are tied to {cur_ref} · re-run any time" } }
                                    div { class: "strat-run",
                                        label { class: "strat-inl", span { "Backtest days" } input { class: "num short", value: "{days}", oninput: move |e| days.set(e.value()) } }
                                        button { class: "mini accent", disabled: !eval_block.is_empty(), title: "{eval_block}", onclick: move |_| run_eval("backtest"), "Backtest" }
                                        span { class: "pkg-sep" }
                                        label { class: "strat-inl", span { "WF days" } input { class: "num short", value: "{wf_days}", oninput: move |e| wf_days.set(e.value()) } }
                                        label { class: "strat-inl", span { "Train" } input { class: "num short", value: "{train}", oninput: move |e| train.set(e.value()) } }
                                        label { class: "strat-inl", span { "Test" } input { class: "num short", value: "{test}", oninput: move |e| test.set(e.value()) } }
                                        button { class: "mini accent", disabled: !eval_block.is_empty(), title: if eval_block.is_empty() { "Grid from the manifest".to_string() } else { eval_block.to_string() }, onclick: move |_| run_eval("walkforward"), "Walk-forward" }
                                    }
                                    if js(&j, &["kind"]) != "install" || !*show_install.read() { {job_view.clone()} }
                                    div { class: "strat-latest pkg-results",
                                        div { class: "scard",
                                            div { class: "lbl", "Last backtest" }
                                            match bt.clone() {
                                                Some(b) => {
                                                    let pnl = jf(&b, &["pnl_usd"]);
                                                    let file = js(&b, &["report"]);
                                                    let pass = b.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
                                                    rsx! {
                                                        div { class: "val num {pnl_class(pnl)}", "{money(pnl)}" }
                                                        div { class: "muted small", "DD {jf(&b, &[\"max_drawdown_pct\"]):.2}% (cap {js(&b, &[\"dd_cap_pct\"])}%) · {js(&b, &[\"trades\"])} trades · {js(&b, &[\"instrument\"])}" }
                                                        div { class: "small", span { class: if pass { "pill ok" } else { "pill bad" }, if pass { "pass" } else { "fail" } } }
                                                        if !file.is_empty() { button { class: "linkbtn", onclick: move |_| open_report("backtest".to_string(), file.clone()), "Open" } }
                                                    }
                                                }
                                                None => rsx! { div { class: "muted small", "Not run yet" } },
                                            }
                                        }
                                        div { class: "scard",
                                            div { class: "lbl", "Last walk-forward" }
                                            match wf.clone() {
                                                Some(w) if w.get("folds").is_some() => {
                                                    let pnl = jf(&w, &["pnl_usd"]);
                                                    let file = js(&w, &["report"]);
                                                    let pass = w.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
                                                    rsx! {
                                                        div { class: "val num {pnl_class(pnl)}", "{money(pnl)}" }
                                                        div { class: "muted small", "out-of-sample · {js(&w, &[\"positive_folds\"])}/{js(&w, &[\"folds\"])} positive folds (need 60%) · {js(&w, &[\"trades\"])} trades" }
                                                        div { class: "small", span { class: if pass { "pill ok" } else { "pill bad" }, if pass { "pass" } else { "fail" } } }
                                                        if !file.is_empty() { button { class: "linkbtn", onclick: move |_| open_report("walkforward".to_string(), file.clone()), "Open" } }
                                                    }
                                                }
                                                Some(w) => rsx! { div { class: "small text-down", "{js(&w, &[\"error\"])}" } },
                                                None => rsx! { div { class: "muted small", "Not run yet" } },
                                            }
                                        }
                                    }
                                }
                                // ---- parameters (manifest) → overrides of the new node ----
                                div { class: "node-form",
                                    div { class: "bt-title",
                                        span { class: "panel-title", style: "margin:0", "Parameters" }
                                        span { class: "muted small", "From the manifest · changes become overrides of the node you deploy — the package itself is not changed" }
                                    }
                                    div { class: "node-form-grid",
                                        label { span { "Instrument" }
                                            Dropdown { value: "{f.instrument}", onchange: move |v| form.write().instrument = v,
                                                options: insts.iter().map(|i| (i.to_string(), i.to_string())).collect::<DdOpts>() }
                                        }
                                        label { title: "Fixed by the package", span { "Timeframe" } input { value: "{tf}", disabled: true } }
                                        label { span { "Trade size (units)" } input { class: "num", value: "{f.size}", oninput: move |e| form.write().size = e.value() } }
                                        for q in params.iter() {
                                            {
                                                let key = js(q, &["key"]);
                                                let k2 = key.clone();
                                                let val = f.params.get(&key).cloned().unwrap_or_default();
                                                let def = pkg_default(q);
                                                let changed = val.trim() != def;
                                                let g = grid.get(&key).map(|x| x.to_string()).unwrap_or_default();
                                                let (lo, hi) = (js(q, &["min"]), js(q, &["max"]));
                                                let mut tip = vec![];
                                                if !lo.is_empty() || !hi.is_empty() { tip.push(format!("range {lo} – {hi}")); }
                                                tip.push(format!("default {def}"));
                                                if !g.is_empty() { tip.push(format!("walk-forward grid {g}")); }
                                                let help = js(q, &["help"]);
                                                if !help.is_empty() { tip.push(help); }
                                                let tip = tip.join(" · ");
                                                let is_bool = js(q, &["type"]) == "bool";
                                                let lbl = { let l = js(q, &["label"]); if l.is_empty() { key.clone() } else { l } };
                                                rsx! {
                                                    label { class: if changed { "pkg-changed" } else { "" }, title: "{tip}", span { "{lbl}" }
                                                        if is_bool {
                                                            Dropdown { value: if val == "true" { "true" } else { "false" }, onchange: move |v| { form.write().params.insert(k2.clone(), v); },
                                                                options: dd_opts([("true", "true"), ("false", "false")]) }
                                                        } else {
                                                            input { class: if js(q, &["type"]) == "str" { "" } else { "num" }, value: "{val}", oninput: move |e| { form.write().params.insert(k2.clone(), e.value()); } }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        label { span { "Stop loss (pips, 0 = off)" } input { class: "num", value: "{f.sl}", oninput: move |e| form.write().sl = e.value() } }
                                        label { span { "Take profit (pips, 0 = off)" } input { class: "num", value: "{f.tp}", oninput: move |e| form.write().tp = e.value() } }
                                        div { class: "node-form-checks",
                                            label { class: "chk", input { r#type: "checkbox", checked: f.allow_short, oninput: move |e| form.write().allow_short = e.checked() } span { "Allow short" } }
                                        }
                                    }
                                    div { class: "node-form-btns",
                                        button { class: "mini", onclick: move |_| { form_for.set(String::new()); let r = sel.peek().clone(); sel.set(r); }, "Reset to defaults" }
                                        span { class: "muted small pkg-ov", "Overrides: {pkg_overrides(&f, &cur)}" }
                                    }
                                }
                                // ---- deploy ----
                                div { class: "node-form",
                                    div { class: "bt-title",
                                        span { class: "panel-title", style: "margin:0", "Deploy as Nautilus node" }
                                        span { class: "muted small", if live_gw { "Live gateway: the node can start only after Allow live (soak GO)" } else { "Paper — allowed from installed" } }
                                    }
                                    div { class: "strat-run",
                                        label { class: "strat-inl", span { "Node name" } input { value: "{dep_name}", oninput: move |e| dep_name.set(e.value()) } }
                                        label { class: "chk", input { r#type: "checkbox", checked: *dep_auto.read(), oninput: move |e| dep_auto.set(e.checked()) } span { "Auto-restart" } }
                                        label { class: "chk", input { r#type: "checkbox", checked: *dep_start.read(), oninput: move |e| dep_start.set(e.checked()) } span { "Start now" } }
                                        button { class: "mini accent", disabled: !dep_block.is_empty() || busy_now, title: "{dep_block}", onclick: do_deploy, "Deploy" }
                                    }
                                }
                            }
                            // ---- nodes using this version + Allow live ----
                            div { class: "node-form",
                                div { class: "bt-title",
                                    span { class: "panel-title", style: "margin:0", "Nodes using {cur_ref}" }
                                    input { class: "pkg-ramp-code", r#type: "password", placeholder: "Live code (to allow live)", value: "{live_code}", oninput: move |e| live_code.set(e.value()) }
                                    span { class: "muted small", "Live starts at 10% size and steps up as real results pass the ramp checks" }
                                }
                                if node_info.is_empty() { p { class: "muted small", "No node uses this version yet." } }
                                for ni in node_info.iter().cloned() {
                                    {
                                        let nid = js(&ni, &["id"]);
                                        let nm_ = js(&ni, &["name"]);
                                        let run = ni.get("running").and_then(|x| x.as_bool()).unwrap_or(false);
                                        let allowed = ni.get("allow_live").and_then(|x| x.as_bool()).unwrap_or(false);
                                        let ov = ni.get("overrides").cloned().unwrap_or(serde_json::Value::Null);
                                        let ov_txt = if ov.as_object().map(|o| o.is_empty()).unwrap_or(true) { "defaults".to_string() } else { ov.to_string() };
                                        let (nid_open, nid_live) = (nid.clone(), nid.clone());
                                        let tip = if allowed { "Revoke: the node can no longer start on a live gateway".to_string() } else if can_live { "Allow this node to run on a live gateway".to_string() } else { needs.clone() };
                                        rsx! {
                                            div { class: "pkg-node",
                                                span { class: if run { "nd-dot run" } else { "nd-dot" } }
                                                button { class: "linkbtn", title: "Open agent details", onclick: move |_| st.node_detail.set(Some(nid_open.clone())), "{nm_}" }
                                                span { class: "muted small", "{nid}" }
                                                span { class: "muted small pkg-ov", "{ov_txt}" }
                                                if allowed { span { class: "pkg-badge live", "live allowed" } } else { span { class: "muted small", "paper only" } }
                                                button { class: if allowed { "mini danger" } else { "mini accent" }, style: "margin-left:auto", disabled: (!can_live && !allowed) || busy_now, title: "{tip}",
                                                    onclick: move |_| {
                                                        let id = nid_live.clone();
                                                        busy.set(true);
                                                        spawn(async move {
                                                            let c = live_code.peek().clone();
                                                            match node_allow_live(id.clone(), !allowed, c).await {
                                                                Ok(_) => st.toast.set(Some((true, format!("{id}: live {}", if allowed { "revoked" } else { "allowed" })))),
                                                                Err(e) => st.toast.set(Some((false, format!("{id}: {e}")))),
                                                            }
                                                            busy.set(false);
                                                            tick += 1;
                                                        });
                                                    },
                                                    if allowed { "Revoke live" } else { "Allow live" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ConfigsView(mode: Signal<&'static str>) -> Element {
    let mut st = app_state();
    let mut data = use_signal(|| serde_json::Value::Null);
    let mut err = use_signal(String::new);
    let mut tick = use_signal(|| 0u32);
    let mut sel = use_signal(String::new);
    let mut form = use_signal(StratForm::default);
    let mut loaded = use_signal(StratForm::default);
    let mut save_as = use_signal(String::new);
    let mut confirm_del = use_signal(|| false);
    let mut kind = use_signal(|| "backtest".to_string());
    let mut days = use_signal(|| "90".to_string());
    let mut train = use_signal(|| "90".to_string());
    let mut test = use_signal(|| "30".to_string());
    let mut job = use_signal(|| serde_json::Value::Null);
    let mut warnings = use_signal(Vec::<String>::new);
    let mut dep_name = use_signal(String::new);
    let mut dep_auto = use_signal(|| true);
    let mut dep_start = use_signal(|| false);
    let mut busy = use_signal(|| false);

    // load the list (and select the first one if nothing is selected)
    use_effect(move || {
        let _ = tick.read();
        spawn(async move {
            match get_strategies().await {
                Ok(t) => {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    if job.peek().is_null() { if let Some(r) = v.get("running").filter(|r| !r.is_null()) { job.set(r.clone()); } }
                    if sel.peek().is_empty() {
                        if let Some(f) = v.get("configs").and_then(|c| c.as_array()).and_then(|a| a.first()).and_then(|c| c.get("file")).and_then(|f| f.as_str()) { sel.set(f.to_string()); }
                    }
                    data.set(v);
                    err.set(String::new());
                }
                Err(e) => err.set(e.to_string()),
            }
        });
    });
    // selected config changed → fill the form + pre-deploy warnings
    use_effect(move || {
        let f = sel.read().clone();
        let d = data.read().clone();
        if f.is_empty() { return; }
        if let Some(c) = d.get("configs").and_then(|c| c.as_array()).and_then(|a| a.iter().find(|c| js(c, &["file"]) == f)) {
            let sf = strat_form(c.get("config").unwrap_or(&serde_json::Value::Null), d.get("specs").unwrap_or(&serde_json::Value::Null));
            if *loaded.peek() != sf || form.peek().instrument.is_empty() { form.set(sf.clone()); loaded.set(sf); }
            if dep_name.peek().is_empty() { dep_name.set(f.trim_end_matches(".json").to_string()); }
        }
        spawn(async move { warnings.set(get_deploy_check(f).await.unwrap_or_default()); });
    });
    // track running jobs
    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    use_future(move || async move {
        loop {
            crate::sleep_ms(1500).await;
            let j = job.peek().clone();
            let id = js(&j, &["id"]);
            if id.is_empty() || js(&j, &["status"]) != "running" { continue; }
            if let Ok(t) = get_strategy_job(id).await {
                let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                let done = js(&v, &["status"]) != "running";
                if done {
                    let msg = format!("{} {} — {}", js(&v, &["kind"]), js(&v, &["status"]), js(&v, &["summary"]));
                    st.toast.set(Some((js(&v, &["status"]) == "done", msg)));
                    tick += 1;
                }
                job.set(v);
            }
        }
    });

    let d = data.read().clone();
    let configs: Vec<serde_json::Value> = d.get("configs").and_then(|c| c.as_array()).cloned().unwrap_or_default();
    let instruments: Vec<String> = d.get("instruments").and_then(|c| c.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let cur_file = sel.read().clone();
    let cur = configs.iter().find(|c| js(c, &["file"]) == cur_file).cloned().unwrap_or(serde_json::Value::Null);
    let f = form.read().clone();
    let dirty = *loaded.read() != f;
    let j = job.read().clone();
    let running = js(&j, &["status"]) == "running";
    let job_here = js(&j, &["config_file"]) == cur_file;
    let used_by: Vec<String> = cur.get("used_by").and_then(|u| u.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let latest = cur.get("latest").cloned().unwrap_or(serde_json::Value::Null);
    let warns = warnings.read().clone();
    let k = kind.read().clone();

    let specs = d.get("specs").cloned().unwrap_or(serde_json::Value::Null);
    let spec_list = spec_params(&specs, &f.strategy);
    let type_label = js(&specs, &[f.strategy.as_str(), "label"]);
    let type_doc = js(&specs, &[f.strategy.as_str(), "doc"]);
    let timeframes: Vec<i64> = d.get("timeframes").and_then(|t| t.as_array()).map(|a| a.iter().filter_map(|x| x.as_i64()).collect()).unwrap_or(vec![1, 5]);
    let mut do_save = move |file: String, overwrite: bool| {
        if let Err(e) = crate::calc::trade_size_str(&form.peek().size) { st.toast.set(Some((false, format!("Not saved: {e}")))); return; }
        let body = strat_json(&form.peek(), &data.peek().get("specs").cloned().unwrap_or_default());
        busy.set(true);
        spawn(async move {
            match save_strategy(file.clone(), body, overwrite).await {
                Ok(_) => { st.toast.set(Some((true, format!("Saved {file}")))); sel.set(file); loaded.set(StratForm::default()); save_as.set(String::new()); tick += 1; }
                Err(e) => st.toast.set(Some((false, format!("Save failed — {e}")))),
            }
            busy.set(false);
        });
    };
    let run = move |_| {
        let file = sel.peek().clone();
        let kd = kind.peek().clone();
        let mut opts = serde_json::json!({"days": days.peek().trim().parse::<i64>().unwrap_or(90)});
        if kd == "walkforward" {
            opts["train"] = serde_json::json!(train.peek().trim().parse::<i64>().unwrap_or(90));
            opts["test"] = serde_json::json!(test.peek().trim().parse::<i64>().unwrap_or(30));
        }
        spawn(async move {
            match run_strategy_job(file, kd, opts.to_string()).await {
                Ok(t) => job.set(serde_json::from_str(&t).unwrap_or_default()),
                Err(e) => st.toast.set(Some((false, format!("Could not start — {e}")))),
            }
        });
    };
    let mut deploy = move |force: bool| {
        let (file, name, auto, start) = (sel.peek().clone(), dep_name.peek().clone(), *dep_auto.peek(), *dep_start.peek());
        busy.set(true);
        spawn(async move {
            match deploy_strategy(file, name, auto, start, force).await {
                Ok(t) => {
                    let v: serde_json::Value = serde_json::from_str(&t).unwrap_or_default();
                    st.toast.set(Some((true, format!("Node {} created{}", js(&v, &["node", "id"]), if start { " and starting" } else { "" }))));
                    tick += 1;
                }
                Err(e) => st.toast.set(Some((false, format!("Deploy failed — {e}")))),
            }
            busy.set(false);
        });
    };
    let mut open_report = move |kind: String, file: String| { st.bt_open.set(Some((kind, file))); st.bottom_tab.set(9); };

    rsx! {
        div { class: "bt-grid strat",
            div { class: "bt-list",
                div { class: "bt-head",
                    StratModeSwitch { mode }
                    button { class: "mini", style: "margin-left:auto", title: "Reload", onclick: move |_| tick += 1, "↻" }
                }
                if !err.read().is_empty() { p { class: "empty small", "{err}" } }
                for c in configs.iter().cloned() {
                    {
                        let file = js(&c, &["file"]);
                        let cfg = c.get("config").cloned().unwrap_or_default();
                        let active = file == cur_file;
                        let bt = c.get("latest").and_then(|l| l.get("backtest")).cloned();
                        let par = c.get("latest").and_then(|l| l.get("parity")).cloned();
                        let users = c.get("used_by").and_then(|u| u.as_array()).map(|a| a.len()).unwrap_or(0);
                        let f2 = file.clone();
                        rsx! {
                            div { class: if active { "bt-item active" } else { "bt-item" }, onclick: move |_| { sel.set(f2.clone()); confirm_del.set(false); },
                                div { class: "bt-item-top", span { class: "sym", "{file.trim_end_matches(\".json\")}" } }
                                div { class: "r small",
                                    if users > 0 { span { class: "tag mint xs", title: "Deployed as node", "{users} node" } }
                                }
                                div { class: "muted small", "{js(&cfg, &[\"instrument\"])} {js(&cfg, &[\"tf_minutes\"])}m · {js(&c, &[\"summary\"])}" }
                                div { class: "r small",
                                    if let Some(b) = bt.clone() { span { class: "num {pnl_class(jf(&b, &[\"pnl\"]))}", title: "Last backtest", "{money(jf(&b, &[\"pnl\"]))}" } }
                                    if let Some(p) = par.clone() { span { class: if p.get("passed").and_then(|x| x.as_bool()).unwrap_or(false) { "tag mint xs" } else { "tag rose xs" }, title: "Last parity", style: "margin-left:4px", "{js(&p, &[\"label\"])}" } }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "bt-detail",
                if cur.is_null() { p { class: "empty", "Select a config" } } else {
                    // ---- parameters ----
                    div { class: "node-form",
                        div { class: "bt-title",
                            span { class: "panel-title", style: "margin:0", "Parameters · {cur_file}" }
                            span { class: "tag lav xs", title: "{type_doc}", "{type_label}" }
                            if dirty { span { class: "pill warn", "Unsaved changes" } }
                            if !used_by.is_empty() { span { class: "muted small", "Running as node: {used_by.join(\", \")} (restart the node after saving)" } }
                        }
                        div { class: "node-form-grid",
                            label { span { "Strategy ID" } input { value: "{f.strategy_id}", oninput: move |e| form.write().strategy_id = e.value() } }
                            label { span { "Instrument" }
                                Dropdown { value: "{f.instrument}", onchange: move |v| form.write().instrument = v,
                                    options: instruments.iter().map(|i| (i.to_string(), i.to_string())).collect::<DdOpts>() }
                            }
                            label { span { "Timeframe" }
                                Dropdown { value: "{f.tf}", onchange: move |v| form.write().tf = v,
                                    options: timeframes.iter().map(|t| (t.to_string(), if *t == 1 { "1 minute".to_string() } else { format!("{t} minutes") })).collect::<DdOpts>() }
                            }
                            label { span { "Trade size (units)" } input { class: "num", value: "{f.size}", oninput: move |e| form.write().size = e.value() } }
                            for p in spec_list.iter() {
                                {
                                    let key = js(p, &["key"]);
                                    let k2 = key.clone();
                                    let val = f.params.get(&key).cloned().unwrap_or_default();
                                    let is_num = js(p, &["type"]) != "str";
                                    let tip = { let (lo, hi) = (js(p, &["min"]), js(p, &["max"])); if lo.is_empty() && hi.is_empty() { js(p, &["help"]) } else { format!("{lo} – {hi}") } };
                                    rsx! { label { title: "{tip}", span { "{js(p, &[\"label\"])}" }
                                        input { class: if is_num { "num" } else { "" }, value: "{val}", oninput: move |e| { form.write().params.insert(k2.clone(), e.value()); } } } }
                                }
                            }
                            label { span { "Stop loss (pips, 0 = off)" } input { class: "num", value: "{f.sl}", oninput: move |e| form.write().sl = e.value() } }
                            label { span { "Take profit (pips, 0 = off)" } input { class: "num", value: "{f.tp}", oninput: move |e| form.write().tp = e.value() } }
                            div { class: "node-form-checks",
                                label { class: "chk", input { r#type: "checkbox", checked: f.allow_short, oninput: move |e| form.write().allow_short = e.checked() } span { "Allow short" } }
                            }
                        }
                        div { class: "node-form-btns",
                            button { class: "mini accent", disabled: !dirty || *busy.read(), onclick: move |_| do_save(sel.peek().clone(), true), "Save" }
                            if dirty { button { class: "mini", onclick: move |_| form.set(loaded.peek().clone()), "Revert" } }
                            input { class: "strat-saveas", placeholder: "new_file_name", value: "{save_as}", oninput: move |e| save_as.set(e.value()) }
                            button { class: "mini", disabled: file_slug(&save_as.read()).is_empty() || *busy.read(), onclick: move |_| { let fl = file_slug(&save_as.peek()); do_save(fl, false); }, "Save as new" }
                            div { class: "tabs-right",
                                if *confirm_del.read() {
                                    span { class: "muted small", "Delete {cur_file}?" }
                                    button { class: "mini danger", onclick: move |_| {
                                        let fl = sel.peek().clone();
                                        confirm_del.set(false);
                                        spawn(async move {
                                            match delete_strategy(fl.clone()).await {
                                                Ok(_) => { st.toast.set(Some((true, format!("Deleted {fl}")))); sel.set(String::new()); tick += 1; }
                                                Err(e) => st.toast.set(Some((false, format!("Delete failed — {e}")))),
                                            }
                                        });
                                    }, "Yes, delete" }
                                    button { class: "mini", onclick: move |_| confirm_del.set(false), "No" }
                                } else {
                                    button { class: "mini", disabled: !used_by.is_empty(), title: if used_by.is_empty() { "Delete this config" } else { "Used by a node" }, onclick: move |_| confirm_del.set(true), "Delete" }
                                }
                            }
                        }
                    }
                    // ---- test ----
                    div { class: "node-form",
                        div { class: "bt-title", span { class: "panel-title", style: "margin:0", "Test on historical data" } span { class: "muted small", "Data catalog · fills at bar close · 0.2 bps fees" } }
                        div { class: "strat-run",
                            div { class: "tfs",
                                for (kd, lbl, dd) in [("backtest", "Backtest", "90"), ("parity", "Parity vs legacy agent", "60"), ("walkforward", "Walk-forward", "365")] {
                                    button { class: if k == kd { "tf active" } else { "tf" }, onclick: move |_| { kind.set(kd.to_string()); days.set(dd.to_string()); }, "{lbl}" }
                                }
                            }
                            label { class: "strat-inl", span { "Days" } input { class: "num short", value: "{days}", oninput: move |e| days.set(e.value()) } }
                            if k == "walkforward" {
                                label { class: "strat-inl", span { "Train" } input { class: "num short", value: "{train}", oninput: move |e| train.set(e.value()) } }
                                label { class: "strat-inl", span { "Test" } input { class: "num short", value: "{test}", oninput: move |e| test.set(e.value()) } }
                            }
                            button { class: "mini accent", disabled: running || dirty, title: if dirty { "Save changes first" } else { "" }, onclick: run, if running { "Running…" } else { "Run" } }
                            if dirty { span { class: "muted small", "Save changes first" } }
                        }
                        if !j.is_null() && (job_here || running) {
                            {
                                let status = js(&j, &["status"]);
                                let prog = j.get("progress").and_then(|p| p.as_f64());
                                let width = format!("{:.0}%", prog.unwrap_or(0.0) * 100.0);
                                let jid = js(&j, &["id"]);
                                let rep = j.get("report").cloned().filter(|r| !r.is_null());
                                let pill = match status.as_str() { "done" => "pill ok", "running" => "pill warn", _ => "pill bad" };
                                rsx! {
                                    div { class: "strat-job",
                                        span { class: "{pill}", "{status}" }
                                        span { class: "small", "{js(&j, &[\"kind\"])} · {js(&j, &[\"config_file\"])}" }
                                        if running {
                                            div { class: if prog.is_some() { "strat-bar" } else { "strat-bar indet" }, div { class: "strat-bar-fill", style: "width:{width}" } }
                                            button { class: "mini", onclick: move |_| { let id = jid.clone(); spawn(async move { let _ = cancel_strategy_job(id).await; }); }, "Cancel" }
                                        }
                                        span { class: "muted small strat-last", if running { "{js(&j, &[\"last_line\"])}" } else { "{js(&j, &[\"summary\"])}" } }
                                        if let Some(r) = rep {
                                            button { class: "mini accent", onclick: move |_| open_report(js(&r, &["kind"]), js(&r, &["file"])), "Open report" }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "strat-latest",
                            for (kd, lbl) in [("backtest", "Last backtest"), ("parity", "Last parity"), ("walkforward", "Last walk-forward")] {
                                {
                                    let r = latest.get(kd).cloned().filter(|r| !r.is_null());
                                    rsx! {
                                        div { class: "scard",
                                            div { class: "lbl", "{lbl}" }
                                            match r {
                                                Some(r) => {
                                                    let pnl = jf(&r, &["pnl"]);
                                                    let file = js(&r, &["file"]);
                                                    rsx! {
                                                        div { class: "val num {pnl_class(pnl)}", "{money(pnl)}" }
                                                        div { class: "muted small", "{js(&r, &[\"label\"])} · {jdate(&js(&r, &[\"created\"]))}" }
                                                        button { class: "linkbtn", onclick: move |_| open_report(kd.to_string(), file.clone()), "Open" }
                                                    }
                                                }
                                                None => rsx! { div { class: "muted small", "Not run yet" } },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // ---- deploy ----
                    div { class: "node-form",
                        div { class: "bt-title", span { class: "panel-title", style: "margin:0", "Deploy as Nautilus node" } span { class: "muted small", "Trades on the gateway selected at sign-in, with the account's risk limits" } }
                        if !warns.is_empty() {
                            div { class: "strat-warn", for w in warns.iter() { div { "⚠ {w}" } } }
                        }
                        div { class: "strat-run",
                            label { class: "strat-inl", span { "Node name" } input { value: "{dep_name}", oninput: move |e| dep_name.set(e.value()) } }
                            label { class: "chk", input { r#type: "checkbox", checked: *dep_auto.read(), oninput: move |e| dep_auto.set(e.checked()) } span { "Auto-restart" } }
                            label { class: "chk", input { r#type: "checkbox", checked: *dep_start.read(), oninput: move |e| dep_start.set(e.checked()) } span { "Start now" } }
                            button { class: if warns.is_empty() { "mini accent" } else { "mini danger" }, disabled: dirty || *busy.read(), onclick: move |_| deploy(!warnings.peek().is_empty()),
                                if warns.is_empty() { "Deploy" } else { "Deploy anyway" } }
                            if dirty { span { class: "muted small", "Save changes first" } }
                        }
                    }
                }
            }
        }
    }
}

/// Display name of a QOS Cloud plan ("free" → "Community", "pro" → "Researcher", "team" → "Enterprise").
fn plan_label(plan: &str) -> String {
    match plan {
        "free" => "Community".into(),
        "pro" => "Researcher".into(),
        "team" => "Enterprise".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod qa_tests {

    #[test]
    fn dropdown_label_and_placement() {
        let o = super::dd_opts([("ALL", "All markets"), ("FX", "Forex")]);
        assert_eq!(super::dd_label(&o, "FX"), "Forex");
        assert_eq!(super::dd_label(&o, "XX"), "XX");
        assert_eq!(super::dd_place(100.0, 124.0, 900.0, 5), (false, 320.0));
        let (up, h) = super::dd_place(800.0, 824.0, 900.0, 10);
        assert!(up && h > 200.0);
        let (up, h) = super::dd_place(20.0, 44.0, 120.0, 10);
        assert!(!up && h >= 60.0);
    }
    use super::*;

    fn snap_with(acct: &str, issues: i64, nodes: &[&str]) -> Snapshot {
        Snapshot { account: Account { account_id: acct.into(), net_liquidation: 100000.0, ..Default::default() },
                   reconcile: Reconcile { status: "mismatch".into(), issues, node_keys: 1, ..Default::default() },
                   running_nodes: nodes.iter().map(|s| s.to_string()).collect(), ..Default::default() }
    }

    #[test] fn ws_push_keeps_reconcile_nodes_and_account() {
        let prev = snap_with("DU1", 1, &["ema1"]);
        // websocket push: no reconcile / running_nodes keys, account failed ({})
        let push: Snapshot = serde_json::from_value(serde_json::json!({"account": {}, "positions": [], "orders": [], "fills": [], "ib_connected": true, "mode": "agent"})).unwrap();
        let (m, acct_ok) = merge_snapshot(Some(&prev), push, false, false);
        assert_eq!(m.reconcile.issues, 1);
        assert_eq!(m.running_nodes, vec!["ema1".to_string()]);
        assert_eq!(m.account.account_id, "DU1");
        assert!(!acct_ok);
        // a full HTTP snapshot replaces everything
        let (m2, ok2) = merge_snapshot(Some(&prev), snap_with("DU1", 0, &[]), true, true);
        assert_eq!((m2.reconcile.issues, m2.running_nodes.len(), ok2), (0, 0, true));
    }

    #[test] fn positions_rows_with_local_currency_and_unpriced_cash() {
        // shapes from server.py _position_row / cash-balance fallback
        let p: Position = serde_json::from_value(serde_json::json!({"key": "FX:EURGBP", "symbol": "EURGBP", "market": "FX", "position": 100000.0,
            "market_value": 84000.0, "unrealized_pnl": 120.0, "realized_pnl": 0.0, "currency": "GBP", "usd": false, "pnl_ccy": "GBP"})).unwrap();
        assert!(!p.usd);
        assert_eq!(money_in(p.unrealized_pnl, p.usd, &p.currency), "120.00 GBP");
        let legacy: Position = serde_json::from_value(serde_json::json!({"symbol": "EURUSD", "unrealized_pnl": 5.0})).unwrap();
        assert!(legacy.usd);                                   // older API without the field: values were USD
        let cash: Position = serde_json::from_value(serde_json::json!({"key": "FX:USDJPY", "symbol": "USDJPY", "position": 0.0, "usd": false, "currency": "JPY",
            "unpriced": true, "cash_balance": -1500000.0, "cash_currency": "JPY"})).unwrap();
        assert!(cash.unpriced && cash.cash_balance < 0.0);
        let (t, skipped) = crate::calc::usd_total([&p, &legacy].iter().map(|x| (x.unrealized_pnl, x.usd)));
        assert_eq!((t, skipped), (5.0, 1));
    }

    #[test] fn reconcile_drift_and_stats_breakeven_parse() {
        let r: Reconcile = serde_json::from_value(serde_json::json!({"status": "warn", "rows": [{"key": "FX:EURUSD", "status": "ok", "drift": true}]})).unwrap();
        assert!(r.rows[0].drift);
        let s: Stats = serde_json::from_value(serde_json::json!({"trades": 3, "wins": 1, "losses": 1, "breakeven": 1})).unwrap();
        assert_eq!(s.breakeven, 1);
    }

    #[test] fn nulls_never_reject_a_snapshot_or_trade_list() {
        // /fills adjustment row with price null, reconcile null, null numbers in positions / orders
        let snap: Snapshot = serde_json::from_value(serde_json::json!({"account": null, "positions": [{"symbol": "EURUSD", "position": null, "unrealized_pnl": null}],
            "orders": [{"order_id": 3, "symbol": "EURUSD", "side": "BUY", "type": "LMT", "qty": null, "limit_price": null}],
            "fills": [{"time": null, "key": "FX:EURUSD", "symbol": "EURUSD", "side": "BOT", "qty": 1000.0, "price": null}],
            "ib_connected": true, "reconcile": null, "running_nodes": null})).unwrap();
        assert_eq!(snap.fills[0].price, None);
        assert_eq!(snap.positions[0].position, 0.0);
        assert!(snap.account.account_id.is_empty());
        let t: Vec<Trade> = serde_json::from_value(serde_json::json!([{"key": "FX:EURGBP", "symbol": "EURGBP", "entry_price": null, "pnl": null, "net_pnl": null, "pnl_unknown": 1, "commission": null, "open": 0},
                                                                     {"key": "FX:EURUSD", "symbol": "EURUSD", "entry_price": 1.08, "pnl": 12.0, "net_pnl": 10.0}])).unwrap();
        assert_eq!((t[0].net_pnl, t[0].pnl_unknown, t[1].net_pnl), (None, 1, Some(10.0)));
        let c: Calendar = serde_json::from_value(serde_json::json!({"items": [], "news_pause_minutes": 5, "calendar_stale": true})).unwrap();
        assert!(c.calendar_stale);
        let d: DailyPnl = serde_json::from_value(serde_json::json!({"date": "2026-10-30", "baseline": "unconfirmed", "baseline_confirmed": false})).unwrap();
        assert!(!d.baseline_confirmed);
        let old: DailyPnl = serde_json::from_value(serde_json::json!({"date": "2026-10-30"})).unwrap();
        assert!(old.baseline_confirmed);                       // older API: no baseline concept → no warning
    }

    #[test] fn replaced_move_confirmation_reverts_its_line() {
        let mv = ConfirmReq { title: "Move order?".into(), body: String::new(), ok_label: "Move order".into(), action: ConfirmAction::MoveOrder { id: 7, from: 1.25, to: 1.29, pane: 1 } };
        assert_eq!(revert_on_cancel(Some(&mv)), Some((1, 7, 1.25)));
        let k = ConfirmReq { title: String::new(), body: String::new(), ok_label: KILL_CONFIRM_LABEL.into(), action: ConfirmAction::Kill };
        assert_eq!(revert_on_cancel(Some(&k)), None);
        assert_eq!(revert_on_cancel(None), None);
        assert!(KILL_CONFIRM_LABEL.contains("ALL accounts"));
    }

    #[test] fn all_accounts_equity_partial_flag() {
        let pts: Vec<EquityPoint> = serde_json::from_value(serde_json::json!([{"ts": "2026-10-01T00:00:00+00:00", "netliq": 1.0, "partial": null},
            {"ts": "2026-10-02T00:00:00+00:00", "netliq": 2.0, "partial": true}, {"ts": "2026-10-03T00:00:00+00:00", "netliq": 3.0}])).unwrap();
        assert!(equity_partial(&pts));
        assert!(!equity_partial(&pts[..1]));
        let s: Stats = serde_json::from_value(serde_json::json!({"equity_partial": true})).unwrap();
        assert_eq!(s.equity_partial, Some(true));
        let s2: Stats = serde_json::from_value(serde_json::json!({"equity_partial": null})).unwrap();
        assert_eq!(s2.equity_partial, None);
        assert!(PARTIAL_TIP.contains("held flat"));
    }

    #[test] fn recon_warn_rows_and_cash_warn() {
        // manual-only book (no node keys, no issues) with a "warn" row → strip visible, row listed amber with its note
        let r: Reconcile = serde_json::from_value(serde_json::json!({"status": "warn", "node_keys": 0, "issues": 0,
            "rows": [{"key": "FX:USDJPY", "status": "warn", "note": "cash balance implies +200,000"}, {"key": "FX:EURUSD", "status": "ok"}]})).unwrap();
        assert!(recon_visible(&r));
        let rows = recon_issue_rows(&r);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "recon-item warn");
        assert!(rows[0].2.contains("USDJPY") && rows[0].2.contains("+200,000"));
        let ok: Reconcile = serde_json::from_value(serde_json::json!({"status": "ok", "rows": [{"key": "FX:EURUSD", "status": "ok"}]})).unwrap();
        assert!(!recon_visible(&ok));
        let p: Position = serde_json::from_value(serde_json::json!({"symbol": "USDJPY", "cash_warn": "cash balance implies +200,000; possible trade outside QOS"})).unwrap();
        assert!(p.cash_warn.unwrap().contains("outside QOS"));
        let q: Position = serde_json::from_value(serde_json::json!({"symbol": "USDJPY", "cash_warn": null})).unwrap();
        assert_eq!(q.cash_warn, None);
    }

    #[test] fn only_manual_orders_are_manual() {
        let mut o = Order { order_id: 1, symbol: "EURUSD".into(), side: "BUY".into(), order_type: "LMT".into(), ..Default::default() };
        assert!(order_is_manual(&o));
        o.source = "node:ema1".into();
        assert!(!order_is_manual(&o));
        o.source = String::new(); o.order_ref = "agent:x".into();
        assert!(!order_is_manual(&o));
        o.order_ref = String::new(); o.source = "manual:close_all".into();
        assert!(order_is_manual(&o));
    }
}
