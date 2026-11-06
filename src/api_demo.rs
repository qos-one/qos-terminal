// Generated from api.rs by scripts/gen_api_demo.py — do not edit by hand.
// Web demo only: every function POSTs {argument name: value} to the same path as the server function and reads JSON back.

/// error type for the web demo (stands in for the fullstack ServerFnError)
#[derive(Debug, Clone, PartialEq)]
pub struct ServerFnError(pub String);
impl ServerFnError { pub fn new(m: impl std::fmt::Display) -> Self { ServerFnError(m.to_string()) } }
impl std::fmt::Display for ServerFnError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }
impl std::error::Error for ServerFnError {}

async fn demo_call<T: serde::de::DeserializeOwned>(path: &str, args: serde_json::Value) -> Result<T, ServerFnError> {
    let req = gloo_net::http::Request::post(path).header("x-qos-demo", "1").json(&args).map_err(ServerFnError::new)?;
    let res = req.send().await.map_err(|e| ServerFnError::new(format!("demo server not responding: {e}")))?;
    let ok = res.ok();
    let text = res.text().await.map_err(ServerFnError::new)?;
    if !ok {
        let msg = serde_json::from_str::<serde_json::Value>(&text).ok().and_then(|v| v.get("message").and_then(|m| m.as_str().map(String::from))).unwrap_or(text);
        return Err(ServerFnError(msg));
    }
    serde_json::from_str(if text.is_empty() { "null" } else { &text }).map_err(|e| ServerFnError::new(format!("{path}: {e}")))
}

pub async fn get_ws_url() -> Result<String, ServerFnError> { demo_call("/api/ws_url", serde_json::json!({})).await }
pub async fn get_markets() -> Result<Vec<Market>, ServerFnError> { demo_call("/api/markets", serde_json::json!({})).await }
pub async fn get_instruments(market: String, q: String) -> Result<Vec<Instrument>, ServerFnError> { demo_call("/api/instruments", serde_json::json!({"market": market, "q": q})).await }
pub async fn get_watchlist() -> Result<Vec<Instrument>, ServerFnError> { demo_call("/api/watchlist", serde_json::json!({})).await }
pub async fn add_watch(key: String) -> Result<Vec<Instrument>, ServerFnError> { demo_call("/api/watchlist/add", serde_json::json!({"key": key})).await }
pub async fn remove_watch(key: String) -> Result<Vec<Instrument>, ServerFnError> { demo_call("/api/watchlist/remove", serde_json::json!({"key": key})).await }
pub async fn get_quotes() -> Result<Vec<Quote>, ServerFnError> { demo_call("/api/quotes", serde_json::json!({})).await }
pub async fn get_market() -> Result<Vec<MarketItem>, ServerFnError> { demo_call("/api/market", serde_json::json!({})).await }
pub async fn get_bars(key: String, bar_size: String, duration: String) -> Result<BarsResp, ServerFnError> { demo_call("/api/bars", serde_json::json!({"key": key, "bar_size": bar_size, "duration": duration})).await }
pub async fn get_snapshot() -> Result<Snapshot, ServerFnError> { demo_call("/api/snapshot", serde_json::json!({})).await }
pub async fn place_order(req: OrderRequest) -> Result<OrderResult, ServerFnError> { demo_call("/api/order", serde_json::json!({"req": req})).await }
pub async fn cancel_order(order_id: i64) -> Result<OrderResult, ServerFnError> { demo_call("/api/order/cancel", serde_json::json!({"order_id": order_id})).await }
pub async fn close_position(key: String, force: bool) -> Result<OrderResult, ServerFnError> { demo_call("/api/position/close", serde_json::json!({"key": key, "force": force})).await }
pub async fn set_mode(mode: String) -> Result<String, ServerFnError> { demo_call("/api/mode", serde_json::json!({"mode": mode})).await }
pub async fn get_heatmap(group: String) -> Result<Heatmap, ServerFnError> { demo_call("/api/heatmap", serde_json::json!({"group": group})).await }
pub async fn modify_order(order_id: i64, qty: Option<f64>, price: Option<f64>) -> Result<OrderResult, ServerFnError> { demo_call("/api/order/modify", serde_json::json!({"order_id": order_id, "qty": qty, "price": price})).await }
pub async fn close_all_positions() -> Result<Vec<ClosedItem>, ServerFnError> { demo_call("/api/positions/close_all", serde_json::json!({})).await }
pub async fn reverse_position(key: String) -> Result<OrderResult, ServerFnError> { demo_call("/api/position/reverse", serde_json::json!({"key": key})).await }
pub async fn get_risk() -> Result<RiskInfo, ServerFnError> { demo_call("/api/risk", serde_json::json!({})).await }
pub async fn put_risk(settings: RiskSettings) -> Result<RiskInfo, ServerFnError> { demo_call("/api/risk/settings", serde_json::json!({"settings": settings})).await }
pub async fn kill_switch() -> Result<bool, ServerFnError> { demo_call("/api/risk/kill", serde_json::json!({})).await }
pub async fn confirm_daily_baseline(netliq: Option<f64>) -> Result<DailyPnl, ServerFnError> { demo_call("/api/risk/daily_baseline", serde_json::json!({"netliq": netliq})).await }
pub async fn resume_trading() -> Result<bool, ServerFnError> { demo_call("/api/risk/resume", serde_json::json!({})).await }
pub async fn get_audit(limit: i64) -> Result<Vec<AuditItem>, ServerFnError> { demo_call("/api/audit", serde_json::json!({"limit": limit})).await }
pub async fn get_trades(range: String, key: String, source: String, account: String, tz: String) -> Result<Vec<Trade>, ServerFnError> { demo_call("/api/history/trades", serde_json::json!({"range": range, "key": key, "source": source, "account": account, "tz": tz})).await }
pub async fn get_equity(range: String, account: String, tz: String) -> Result<Vec<EquityPoint>, ServerFnError> { demo_call("/api/history/equity", serde_json::json!({"range": range, "account": account, "tz": tz})).await }
pub async fn get_stats(range: String, account: String, source: String, tz: String) -> Result<Stats, ServerFnError> { demo_call("/api/history/stats", serde_json::json!({"range": range, "account": account, "source": source, "tz": tz})).await }
pub async fn set_journal(first_exec: String, note: String, tags: String, rating: i64) -> Result<bool, ServerFnError> { demo_call("/api/history/journal", serde_json::json!({"first_exec": first_exec, "note": note, "tags": tags, "rating": rating})).await }
pub async fn get_export(what: String, range: String, account: String, tz: String) -> Result<String, ServerFnError> { demo_call("/api/history/export", serde_json::json!({"what": what, "range": range, "account": account, "tz": tz})).await }
pub async fn sync_databricks() -> Result<serde_json::Value, ServerFnError> { demo_call("/api/history/sync", serde_json::json!({})).await }
pub async fn get_notify() -> Result<NotifyCfg, ServerFnError> { demo_call("/api/notify", serde_json::json!({})).await }
pub async fn put_notify(cfg: NotifyCfg) -> Result<NotifyCfg, ServerFnError> { demo_call("/api/notify", serde_json::json!({"cfg": cfg})).await }
pub async fn notify_test() -> Result<bool, ServerFnError> { demo_call("/api/notify/test", serde_json::json!({})).await }
pub async fn get_alerts() -> Result<Vec<Alert>, ServerFnError> { demo_call("/api/alerts", serde_json::json!({})).await }
pub async fn add_alert(key: String, kind: String, level: f64, note: String, repeat: bool) -> Result<Alert, ServerFnError> { demo_call("/api/alerts/add", serde_json::json!({"key": key, "kind": kind, "level": level, "note": note, "repeat": repeat})).await }
pub async fn delete_alert(id: String) -> Result<bool, ServerFnError> { demo_call("/api/alerts/delete", serde_json::json!({"id": id})).await }
pub async fn get_news(key: String) -> Result<Vec<NewsItem>, ServerFnError> { demo_call("/api/news", serde_json::json!({"key": key})).await }
pub async fn get_article(provider: String, article_id: String) -> Result<String, ServerFnError> { demo_call("/api/news/article", serde_json::json!({"provider": provider, "article_id": article_id})).await }
pub async fn get_calendar() -> Result<Calendar, ServerFnError> { demo_call("/api/calendar", serde_json::json!({})).await }
pub async fn set_news_pause(minutes: i64) -> Result<i64, ServerFnError> { demo_call("/api/calendar/pause", serde_json::json!({"minutes": minutes})).await }
pub async fn get_ui() -> Result<UiSettings, ServerFnError> { demo_call("/api/ui", serde_json::json!({})).await }
pub async fn put_ui(s: UiSettings) -> Result<UiSettings, ServerFnError> { demo_call("/api/ui", serde_json::json!({"s": s})).await }
pub async fn get_health() -> Result<Health, ServerFnError> { demo_call("/api/health", serde_json::json!({})).await }
pub async fn op_status() -> Result<OpStatus, ServerFnError> { demo_call("/api/op/status", serde_json::json!({})).await }
pub async fn op_login(id: String, pin: String) -> Result<Operator, ServerFnError> { demo_call("/api/op/login", serde_json::json!({"id": id, "pin": pin})).await }
pub async fn op_bootstrap(name: String, pin: String) -> Result<Operator, ServerFnError> { demo_call("/api/op/bootstrap", serde_json::json!({"name": name, "pin": pin})).await }
pub async fn op_logout() -> Result<bool, ServerFnError> { demo_call("/api/op/logout", serde_json::json!({})).await }
pub async fn op_list() -> Result<Vec<Operator>, ServerFnError> { demo_call("/api/op/list", serde_json::json!({})).await }
pub async fn op_create(name: String, role: String, pin: String) -> Result<Operator, ServerFnError> { demo_call("/api/op/create", serde_json::json!({"name": name, "role": role, "pin": pin})).await }
pub async fn op_update(id: String, role: Option<String>, active: Option<bool>, pin: Option<String>) -> Result<Operator, ServerFnError> { demo_call("/api/op/update", serde_json::json!({"id": id, "role": role, "active": active, "pin": pin})).await }
pub async fn set_live_code(code: String) -> Result<bool, ServerFnError> { demo_call("/api/security/live_code", serde_json::json!({"code": code})).await }
pub async fn setup_connect(gateway_id: String, live_code: String) -> Result<bool, ServerFnError> { demo_call("/api/setup/connect", serde_json::json!({"gateway_id": gateway_id, "live_code": live_code})).await }
pub async fn gateway_stop(gateway_id: String) -> Result<bool, ServerFnError> { demo_call("/api/setup/gateway_stop", serde_json::json!({"gateway_id": gateway_id})).await }
pub async fn gateway_remove(gateway_id: String) -> Result<bool, ServerFnError> { demo_call("/api/setup/gateway_remove", serde_json::json!({"gateway_id": gateway_id})).await }
pub async fn get_setup_status() -> Result<SetupStatus, ServerFnError> { demo_call("/api/setup/status", serde_json::json!({})).await }
pub async fn setup_login(username: String, password: String, mode: String, live_code: String) -> Result<bool, ServerFnError> { demo_call("/api/setup/login", serde_json::json!({"username": username, "password": password, "mode": mode, "live_code": live_code})).await }
pub async fn setup_account(account_id: String) -> Result<bool, ServerFnError> { demo_call("/api/setup/account", serde_json::json!({"account_id": account_id})).await }
pub async fn setup_logout(stop_gateway: bool) -> Result<bool, ServerFnError> { demo_call("/api/setup/logout", serde_json::json!({"stop_gateway": stop_gateway})).await }
pub async fn put_assistant(write_enabled: Option<bool>, live: Option<bool>, max_order_notional_usd: Option<f64>, max_orders_per_hour: Option<i64>, allow_short: Option<bool>, allow_agents: Option<bool>) -> Result<AssistantCfg, ServerFnError> { demo_call("/api/assistant", serde_json::json!({"write_enabled": write_enabled, "live": live, "max_order_notional_usd": max_order_notional_usd, "max_orders_per_hour": max_orders_per_hour, "allow_short": allow_short, "allow_agents": allow_agents})).await }
pub async fn probe_markets() -> Result<Vec<ProbeResult>, ServerFnError> { demo_call("/api/risk/markets/probe", serde_json::json!({})).await }
pub async fn set_market_tradable(market: String, tradable: bool) -> Result<bool, ServerFnError> { demo_call("/api/risk/markets", serde_json::json!({"market": market, "tradable": tradable})).await }
pub async fn get_bt_reports() -> Result<Vec<BtReport>, ServerFnError> { demo_call("/api/backtests", serde_json::json!({})).await }
pub async fn get_bt_report(kind: String, file: String) -> Result<String, ServerFnError> { demo_call("/api/backtests/report", serde_json::json!({"kind": kind, "file": file})).await }
pub async fn get_nodes() -> Result<NodesResp, ServerFnError> { demo_call("/api/nodes", serde_json::json!({})).await }
pub async fn save_node(id: String, name: String, config_file: String, overrides: String, auto_restart: bool) -> Result<NodeInfo, ServerFnError> { demo_call("/api/nodes/save", serde_json::json!({"id": id, "name": name, "config_file": config_file, "overrides": overrides, "auto_restart": auto_restart})).await }
pub async fn node_action(id: String, action: String) -> Result<String, ServerFnError> { demo_call("/api/nodes/action", serde_json::json!({"id": id, "action": action})).await }
pub async fn get_node_log(id: String, level: String) -> Result<Vec<String>, ServerFnError> { demo_call("/api/nodes/log", serde_json::json!({"id": id, "level": level})).await }
pub async fn get_node_events(node: String, types: String) -> Result<String, ServerFnError> { demo_call("/api/events", serde_json::json!({"node": node, "types": types})).await }
pub async fn get_strategies() -> Result<String, ServerFnError> { demo_call("/api/strategies", serde_json::json!({})).await }
pub async fn save_strategy(file: String, config: String, overwrite: bool) -> Result<String, ServerFnError> { demo_call("/api/strategies/save", serde_json::json!({"file": file, "config": config, "overwrite": overwrite})).await }
pub async fn delete_strategy(file: String) -> Result<bool, ServerFnError> { demo_call("/api/strategies/delete", serde_json::json!({"file": file})).await }
pub async fn run_strategy_job(file: String, kind: String, opts: String) -> Result<String, ServerFnError> { demo_call("/api/strategies/run", serde_json::json!({"file": file, "kind": kind, "opts": opts})).await }
pub async fn get_strategy_job(id: String) -> Result<String, ServerFnError> { demo_call("/api/strategies/job", serde_json::json!({"id": id})).await }
pub async fn cancel_strategy_job(id: String) -> Result<bool, ServerFnError> { demo_call("/api/strategies/job/cancel", serde_json::json!({"id": id})).await }
pub async fn get_deploy_check(file: String) -> Result<Vec<String>, ServerFnError> { demo_call("/api/strategies/deploy_check", serde_json::json!({"file": file})).await }
pub async fn deploy_strategy(file: String, name: String, auto_restart: bool, start: bool, force: bool) -> Result<String, ServerFnError> { demo_call("/api/strategies/deploy", serde_json::json!({"file": file, "name": name, "auto_restart": auto_restart, "start": start, "force": force})).await }
pub async fn get_packages() -> Result<String, ServerFnError> { demo_call("/api/packages", serde_json::json!({})).await }
pub async fn install_package(source: String, force: bool) -> Result<String, ServerFnError> { demo_call("/api/packages/install", serde_json::json!({"source": source, "force": force})).await }
pub async fn evaluate_package(pkg: String, opts: String) -> Result<String, ServerFnError> { demo_call("/api/packages/evaluate", serde_json::json!({"pkg": pkg, "opts": opts})).await }
pub async fn deploy_package(pkg: String, body: String) -> Result<String, ServerFnError> { demo_call("/api/packages/deploy", serde_json::json!({"pkg": pkg, "body": body})).await }
pub async fn node_allow_live(id: String, enabled: bool, live_code: String) -> Result<String, ServerFnError> { demo_call("/api/nodes/allow_live", serde_json::json!({"id": id, "enabled": enabled, "live_code": live_code})).await }
pub async fn node_ramp_action(id: String, action: String, live_code: String, force: bool) -> Result<String, ServerFnError> { demo_call("/api/nodes/ramp", serde_json::json!({"id": id, "action": action, "live_code": live_code, "force": force})).await }
pub async fn get_portfolio_risk() -> Result<String, ServerFnError> { demo_call("/api/portfolio/risk", serde_json::json!({})).await }
pub async fn put_portfolio_limits(body: String) -> Result<String, ServerFnError> { demo_call("/api/portfolio/limits", serde_json::json!({"body": body})).await }
pub async fn get_system_health() -> Result<String, ServerFnError> { demo_call("/api/system/health", serde_json::json!({})).await }
pub async fn run_backup() -> Result<String, ServerFnError> { demo_call("/api/system/backup", serde_json::json!({})).await }
pub async fn release_hold(id: String) -> Result<String, ServerFnError> { demo_call("/api/nodes/release_hold", serde_json::json!({"id": id})).await }
pub async fn get_cloud_status() -> Result<String, ServerFnError> { demo_call("/api/cloud/status", serde_json::json!({})).await }
pub async fn cloud_action(what: String, arg: String) -> Result<String, ServerFnError> { demo_call("/api/cloud/action", serde_json::json!({"what": what, "arg": arg})).await }
pub async fn get_store() -> Result<String, ServerFnError> { demo_call("/api/cloud/store", serde_json::json!({})).await }
pub async fn get_store_detail(slug: String) -> Result<String, ServerFnError> { demo_call("/api/cloud/store/detail", serde_json::json!({"slug": slug})).await }
pub async fn store_install(slug: String, version: String) -> Result<String, ServerFnError> { demo_call("/api/cloud/store/install", serde_json::json!({"slug": slug, "version": version})).await }
pub async fn billing_action(what: String, arg: String) -> Result<String, ServerFnError> { demo_call("/api/cloud/billing", serde_json::json!({"what": what, "arg": arg})).await }
pub async fn get_live_readiness(node: String) -> Result<String, ServerFnError> { demo_call("/api/live/readiness", serde_json::json!({"node": node})).await }
pub async fn get_soak() -> Result<SoakResp, ServerFnError> { demo_call("/api/soak", serde_json::json!({})).await }
pub async fn get_soak_detail(id: String) -> Result<SoakNode, ServerFnError> { demo_call("/api/soak/detail", serde_json::json!({"id": id})).await }
pub async fn soak_parity(id: String) -> Result<serde_json::Value, ServerFnError> { demo_call("/api/soak/parity", serde_json::json!({"id": id})).await }
pub async fn soak_reset(id: String) -> Result<serde_json::Value, ServerFnError> { demo_call("/api/soak/reset", serde_json::json!({"id": id})).await }
pub async fn get_node_summary(id: String, range: String, tz: String) -> Result<NodeSummary, ServerFnError> { demo_call("/api/nodes/summary", serde_json::json!({"id": id, "range": range, "tz": tz})).await }
