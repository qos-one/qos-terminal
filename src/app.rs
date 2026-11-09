//! Main Terminal screen (App) — split from main.rs so the web demo (../demo) can share it
use std::collections::HashMap;

use dioxus::prelude::*;

use crate::api::*;
use crate::ui::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const LWC_JS: Asset = asset!("/assets/lightweight-charts.js");
const CHART_JS: Asset = asset!("/assets/chart.js");

#[component]
pub fn App() -> Element {
    let st = State {
        quotes: use_signal(Vec::new),
        prev_mid: use_signal(HashMap::new),
        snapshot: use_signal(|| None),
        api_error: use_signal(|| None),
        selected: use_signal(|| "FX:EURUSD".to_string()),
        pane_key: [use_signal(|| "FX:EURUSD".to_string()), use_signal(|| "FX:GBPUSD".to_string()), use_signal(|| "STK:SPY".to_string()), use_signal(|| "CRYPTO:BTC".to_string())],
        pane_tf: [use_signal(|| "1m".to_string()), use_signal(|| "5m".to_string()), use_signal(|| "15m".to_string()), use_signal(|| "1h".to_string())],
        pane_bars: [use_signal(Vec::new), use_signal(Vec::new), use_signal(Vec::new), use_signal(Vec::new)],
        pane_loading: [use_signal(|| false), use_signal(|| false), use_signal(|| false), use_signal(|| false)],
        layout: use_signal(|| 1usize),
        active_pane: use_signal(|| 0usize),
        toast: use_signal(|| None),
        refresh_tick: use_signal(|| 0u32),
        market: use_signal(Vec::new),
        markets: use_signal(Vec::new),
        watchlist: use_signal(Vec::new),
        wl_market: use_signal(|| "ALL".to_string()),
        wl_filter: use_signal(String::new),
        browser_open: use_signal(|| false),
        mode: use_signal(|| "manual".to_string()),
        heat_group: use_signal(|| None),
        node_detail: use_signal(|| None),
        strat_pkg: use_signal(|| None),
        perf_full: use_signal(|| false),
        chart_full: use_signal(|| None),
        risk: use_signal(|| None),
        indicators: use_signal(IndCfg::default),
        ticket_prefill: use_signal(|| None),
        ctx_menu: use_signal(|| None),
        crosshair: use_signal(|| None),
        bottom_h: use_signal(|| BOTTOM_DEFAULT),
        bottom_drag: use_signal(|| None),
        bottom_tab: use_signal(|| 0usize),
        conn: use_signal(|| None),
        ws_open: use_signal(|| false),
        notifs: use_signal(Vec::new),
        notif_open: use_signal(|| false),
        help_open: use_signal(|| false),
        wl_w: use_signal(|| 300.0),
        ticket_w: use_signal(|| 280.0),
        side_drag: use_signal(|| None),
        alerts: use_signal(Vec::new),
        alert_dialog: use_signal(|| None),
        sound: use_signal(|| true),
        preset: use_signal(|| "Trading".to_string()),
        ui_loaded: use_signal(|| false),
        submit_req: use_signal(|| 0u32),
        news_cache: use_signal(|| None),
        node_events: use_signal(Vec::new),
        node_live: use_signal(std::collections::HashMap::new),
        bt_open: use_signal(|| None),
        session: use_signal(|| None),
        login_busy: use_signal(|| false),
        op: use_signal(|| None),
        hist_scope: use_signal(String::new),
        confirm: use_signal(|| None),
        err_src: use_signal(std::collections::BTreeMap::new),
        ws_quote_seq: use_signal(|| 0u32),
        ws_snap_seq: use_signal(|| 0u32),
        acct_missing: use_signal(|| false),
        bar_dec: use_signal(HashMap::new),
        tz: use_signal(String::new),
    };
    use_context_provider(|| st);
    use_hook(|| crate::ui::ui_zoom_restore());

    #[cfg(any(feature = "desktop", feature = "mobile", feature = "demo", feature = "web"))]
    {
        let mut st = st;

        // browser builds: copy the page clock for calc::now_secs (std::time is not available in wasm)
        #[cfg(target_arch = "wasm32")]
        use_future(move || async move {
            loop {
                let mut ev = document::eval("dioxus.send(Date.now());");
                if let Ok(ms) = ev.recv::<f64>().await { crate::calc::set_web_now_ms(ms); }
                crate::sleep_ms(1000).await;
            }
        });

        // viewer's time zone: history "today" / "week" start at local midnight
        use_future(move || async move {
            let mut ev = document::eval("dioxus.send((Intl.DateTimeFormat().resolvedOptions().timeZone) || '');");
            if let Ok(z) = ev.recv::<String>().await { if !z.is_empty() { st.tz.set(z); } }
        });

        // realtime stream: connect to the bot API configured on the server side (BOT_API), not a hard-coded address
        use_future(move || async move {
            if let Ok(u) = get_ws_url().await {
                let js = serde_json::to_string(&u).unwrap_or_default();
                let _ = document::eval(&format!("window.QX_WS_URL = {js}; if (window.qxWs) window.qxWs.setUrl({js});"));
            }
        });

        // markets and watchlist at startup (retry until the API is ready)
        use_future(move || async move {
            loop {
                let ok_m = match get_markets().await { Ok(m) => { st.markets.set(m); true } Err(_) => false };
                let ok_w = match get_watchlist().await { Ok(w) => { st.watchlist.set(w); true } Err(_) => false };
                if ok_m && ok_w { break; }
                crate::sleep_ms(3000).await;
            }
        });

        // operator session: check every 20 s (token expires when idle → back to the lock screen)
        use_future(move || async move {
            loop {
                if let Ok(o) = op_status().await {
                    let had = st.op.peek().as_ref().and_then(|x| x.operator.clone()).is_some();
                    if had && o.configured && o.operator.is_none() { st.notify("info", "Operator session expired — terminal locked"); }
                    st.op.set(Some(o));
                }
                crate::sleep_ms(20000).await;
            }
        });

        // login/account status: check every 3 s (more often while the gateway is starting)
        use_future(move || async move {
            loop {
                match get_setup_status().await {
                    Ok(s) => { let starting = s.setup.state == "starting"; st.session.set(Some(s)); st.set_err("setup", None);
                        crate::sleep_ms((if starting { 2 } else { 5 }) * 1000).await; }
                    Err(e) => { st.set_err("setup", Some(e.to_string())); crate::sleep_ms(3000).await; }
                }
            }
        });

        // load the saved layout + alerts at startup
        use_future(move || async move {
            loop {
                match get_ui().await {
                    Ok(u) => { st.apply_ui(&u); st.ui_loaded.set(true); break; }
                    Err(_) => crate::sleep_ms(3000).await,
                }
            }
            if let Ok(a) = get_alerts().await { st.alerts.set(a); }
        });
        // save the layout on change (1.5 s debounce)
        let mut save_ver = use_signal(|| 0u32);
        use_effect(move || {
            let _ = (*st.wl_w.read(), *st.ticket_w.read(), *st.bottom_h.read(), *st.layout.read(), *st.sound.read(), st.preset.read().clone());
            for k in st.pane_key.iter() { let _ = k.read(); }
            for t in st.pane_tf.iter() { let _ = t.read(); }
            if !*st.ui_loaded.peek() || st.bottom_drag.peek().is_some() || st.side_drag.peek().is_some() { return; }
            save_ver += 1;
            let my = *save_ver.peek();
            spawn(async move {
                crate::sleep_ms(1500).await;
                if *save_ver.peek() == my { let _ = put_ui(st.ui_settings()).await; }
            });
        });

        // quotes every 1 s (fallback when the websocket is down, or open but silent for 5 s — the API pushes nothing while IB is disconnected)
        use_future(move || async move {
            let (mut last_seq, mut silent) = (0u32, 0u32);
            loop {
                let seq = *st.ws_quote_seq.peek();
                if seq != last_seq { last_seq = seq; silent = 0; } else { silent += 1; }
                if *st.ws_open.peek() && silent < 5 { crate::sleep_ms(1000).await; continue; }
                match get_quotes().await {
                    Ok(qs) => {
                        let prev: HashMap<String, f64> = st.quotes.read().iter().map(|q| (q.key.clone(), q.mid)).collect();
                        st.prev_mid.set(prev);
                        st.quotes.set(qs);
                        st.set_err("quotes", None);
                    }
                    Err(e) => st.set_err("quotes", Some(e.to_string())),
                }
                crate::sleep_ms(1000).await;
            }
        });

        // account / positions / orders / fills every 3 s
        use_future(move || async move {
            let (mut last_seq, mut silent) = (0u32, 0u32);
            loop {
                let seq = *st.ws_snap_seq.peek();
                if seq != last_seq { last_seq = seq; silent = 0; } else { silent += 1; }
                if !*st.ws_open.peek() || silent >= 2 {
                    match get_snapshot().await {
                        Ok(s) => { st.apply_snapshot(s, true, true); st.set_err("snapshot", None); }
                        Err(e) => st.set_err("snapshot", Some(e.to_string())),
                    }
                } else {
                    st.set_err("snapshot", None);
                    match get_health().await { Ok(h) => { st.conn.set(Some(h.conn)); st.set_err("health", None); } Err(e) => st.set_err("health", Some(e.to_string())) }
                }
                if let Ok(r) = get_risk().await { st.risk.set(Some(r)); }
                crate::sleep_ms(3000).await;
            }
        });
        use_effect(move || {
            let _ = st.refresh_tick.read();
            spawn(async move {
                crate::sleep_ms(600).await;
                if let Ok(s) = get_snapshot().await { st.apply_snapshot(s, true, true); }
            });
        });

        // market overview every 5 s
        use_future(move || async move {
            loop {
                if let Ok(m) = get_market().await { st.market.set(m); }
                crate::sleep_ms(5000).await;
            }
        });

        // toasts disappear after 7 s
        use_effect(move || {
            if st.toast.read().is_some() {
                spawn(async move {
                    crate::sleep_ms(7000).await;
                    st.toast.set(None);
                });
            }
        });
    }

    let err = st.api_error.read().clone();
    let op_ok = st.op.read().as_ref().map(|o| o.operator.is_some() || !o.configured).unwrap_or(false);
    let logged_in = op_ok && st.session.read().as_ref().map(|s| s.logged_in).unwrap_or(false);
    let bottom_h = *st.bottom_h.read();
    let dragging = st.bottom_drag.read().is_some() || st.side_drag.read().is_some();
    let mut st2 = st;

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Script { src: LWC_JS }
        document::Script { src: CHART_JS }
        if !logged_in {
            LoginScreen {}
        } else {
        div { class: if dragging { "app dragging" } else { "app" },
            style: "grid-template-rows: 48px auto minmax(200px, 1fr) auto min({bottom_h}px, calc(100% - 360px));",
            onmousemove: move |e| {
                if let Some((y0, h0)) = *st2.bottom_drag.peek() {
                    let dy = y0 - e.client_coordinates().y;
                    st2.bottom_h.set((h0 + dy).clamp(BOTTOM_MIN, 900.0));
                }
                if let Some((side, x0, w0)) = *st2.side_drag.peek() {
                    let dx = e.client_coordinates().x - x0;
                    if side == 0 { st2.wl_w.set((w0 + dx).clamp(SIDE_MIN, SIDE_MAX)); } else { st2.ticket_w.set((w0 - dx).clamp(SIDE_MIN, SIDE_MAX)); }
                }
            },
            onmouseup: move |_| { if st2.bottom_drag.peek().is_some() { st2.bottom_drag.set(None); } if st2.side_drag.peek().is_some() { st2.side_drag.set(None); } },
            onmouseleave: move |_| { if st2.bottom_drag.peek().is_some() { st2.bottom_drag.set(None); } if st2.side_drag.peek().is_some() { st2.side_drag.set(None); } },
            Header {}
            if let Some(e) = err {
                div { class: "banner", "Cannot reach bot API: {e} — run  uv run uvicorn api.server:app --port 8000  in the bot folder" }
            }
            div { class: "workspace",
                Watchlist {}
                div { class: "vsplit", title: "Drag to resize watchlist",
                    onmousedown: move |e| { e.prevent_default(); st2.side_drag.set(Some((0, e.client_coordinates().x, *st2.wl_w.peek()))); } }
                ChartGrid {}
                div { class: "vsplit", title: "Drag to resize side panel",
                    onmousedown: move |e| { e.prevent_default(); st2.side_drag.set(Some((1, e.client_coordinates().x, *st2.ticket_w.peek()))); } }
                if *st.mode.read() == "agent" { NodesSidePanel {} } else { OrderTicket {} }
            }
            ChartEvents {}
            MarketOverview {}
            BottomPanel {}
            Toast {}
            if cfg!(feature = "demo") { DemoBadge {} }
            InstrumentBrowser {}
            HeatmapModal {}
            AgentDetailModal {}
        }
        }
    }
}

/// web demo corner badge (shown only in builds with the "demo" feature)
#[component]
fn DemoBadge() -> Element {
    let site = option_env!("QOS_SITE_URL").unwrap_or("http://localhost:8090");
    rsx! {
        div { class: "demo-badge",
            span { class: "demo-dot" }
            span { b { "Live demo" } " · simulated market data · no real orders" }
            button { class: "demo-fs", title: "Full screen (Esc to exit)", onclick: move |_| { let _ = document::eval(crate::ui::WEB_FULLSCREEN_JS); }, "⤢ Full screen" }
            a { class: "demo-dl", href: "{site}/download", target: "_blank", "Install QOS" }
            a { class: "demo-gh", href: "https://github.com/qos-one", target: "_blank", rel: "noopener", "GitHub" }
        }
    }
}
