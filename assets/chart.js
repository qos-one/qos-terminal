/* quantX chart bridge over TradingView Lightweight Charts v4
   Rust → JS: window.qxChart.<fn>(...)   JS → Rust: window.__qxSend({type, ...}) */
(function () {
  const INSTANCES = {};
  function makeInstance(paneId) {
  const C = { pane: paneId,
    chart: null, sub: null, candles: null, volume: null, subSeries: {}, overlays: {},
    bars: [], decimals: 5, tfSec: 60, key: "", intraday: true,
    orderLines: new Map(), draggable: new Set(), pending: new Map(), posLine: null, hLines: new Map(), markers: [],
    ind: { ema: [20], sma: [], bb: null, vwap: false, volume: true, sub: "none" },
    drag: null, el: null, subEl: null, ready: false,
  };
  const send = (m) => { try { m.pane = C.pane; if (window.__qxSend) window.__qxSend(m); } catch (e) {} };
  const css = (v) => getComputedStyle(document.documentElement).getPropertyValue(v).trim();
  const fmt = (p) => (p == null || isNaN(p)) ? "" : Number(p).toFixed(C.decimals);
  const toSec = (t) => {
    if (typeof t === "number") return t;
    let s = String(t);
    if (s.length === 10) return Math.floor(Date.parse(s + "T00:00:00Z") / 1000);
    if (!/[zZ]|[+-]\d\d:?\d\d$/.test(s)) s += "Z";
    return Math.floor(Date.parse(s) / 1000);
  };

  function theme() {
    return {
      layout: { background: { type: "solid", color: css("--panel") || "#101723" }, textColor: css("--muted") || "#8a9bb3",
                fontFamily: "SF Mono, JetBrains Mono, Menlo, monospace", fontSize: 11 },
      grid: { vertLines: { color: "rgba(34,48,74,.55)" }, horzLines: { color: "rgba(34,48,74,.55)" } },
      crosshair: { mode: 0, vertLine: { color: "rgba(56,189,248,.5)", labelBackgroundColor: "#38bdf8" },
                   horzLine: { color: "rgba(56,189,248,.5)", labelBackgroundColor: "#38bdf8" } },
      rightPriceScale: { borderColor: "#22304a", scaleMargins: { top: 0.08, bottom: 0.22 } },
      timeScale: { borderColor: "#22304a", timeVisible: C.intraday, secondsVisible: false, rightOffset: 4, barSpacing: 8 },
      handleScroll: true, handleScale: true,
    };
  }

  function init(id, subId) {
    const el = document.getElementById(id);
    if (!el || !window.LightweightCharts) {               // web: the chart library or the pane may not be loaded yet → retry for up to 15 s
      if ((C.initTries = (C.initTries || 0) + 1) <= 150) setTimeout(() => init(id, subId), 100);
      return;
    }
    C.initTries = 0;
    if (C.chart && C.el === el) return;
    if (C.chart) destroy();
    C.el = el; C.subEl = document.getElementById(subId);
    C.chart = LightweightCharts.createChart(C.el, theme());
    C.candles = C.chart.addCandlestickSeries({
      upColor: "#2ee57a", downColor: "#ff4d5e", borderUpColor: "#2ee57a", borderDownColor: "#ff4d5e",
      wickUpColor: "#2ee57a", wickDownColor: "#ff4d5e", priceFormat: { type: "price", precision: C.decimals, minMove: Math.pow(10, -C.decimals) },
    });
    C.volume = C.chart.addHistogramSeries({ priceFormat: { type: "volume" }, priceScaleId: "", color: "rgba(56,189,248,.35)" });
    C.volume.priceScale().applyOptions({ scaleMargins: { top: 0.82, bottom: 0 } });

    C.chart.subscribeCrosshairMove((p) => {
      if (!p || !p.time || !p.seriesData) { send({ type: "crosshair", visible: false }); return; }
      const d = p.seriesData.get(C.candles);
      if (!d) return;
      const vol = C.bars.find((b) => b.time === p.time);
      const o = { type: "crosshair", visible: true, time: p.time, o: d.open, h: d.high, l: d.low, c: d.close, v: vol && vol.v > 0 ? vol.v : 0, ind: {} };
      for (const [name, s] of Object.entries(C.overlays)) { const v = p.seriesData.get(s); if (v && v.value != null) o.ind[name] = v.value; }
      send(o);
    });
    const ro = new ResizeObserver(() => { if (C.chart) C.chart.applyOptions({ width: C.el.clientWidth, height: C.el.clientHeight }); if (C.sub) C.sub.applyOptions({ width: C.subEl.clientWidth, height: C.subEl.clientHeight }); });
    ro.observe(C.el); if (C.subEl) ro.observe(C.subEl);
    // when the window is resized (e.g. fullscreen) force a re-measure after layout completes
    window.addEventListener("resize", () => setTimeout(resize, 50));

    // --- right-click: price at cursor ---
    C.el.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      const r = C.el.getBoundingClientRect();
      const raw = C.candles.coordinateToPrice(e.clientY - r.top);
      const price = raw == null || !isFinite(raw) || raw <= 0 ? null : roundPx(raw);
      const near = nearestLine(e.clientY - r.top, false);
      send({ type: "ctx", price, x: e.clientX, y: e.clientY, line: near });
    });
    // --- drag order / horizontal lines ---
    C.el.addEventListener("mousedown", (e) => {
      if (e.button !== 0) return;
      const r = C.el.getBoundingClientRect();
      const near = nearestLine(e.clientY - r.top, true);   // only draggable order lines (manual orders, Manual mode) and h-lines
      if (!near) return;
      C.drag = Object.assign(near, { from: near.price, y0: e.clientY, moved: false });
      C.chart.applyOptions({ handleScroll: false, handleScale: false });
      e.preventDefault();
    });
    window.addEventListener("mousemove", (e) => {
      if (!C.drag) return;
      const r = C.el.getBoundingClientRect();
      if (!C.drag.moved && Math.abs(e.clientY - C.drag.y0) < 3) return;   // a click is not a drag
      const raw = C.candles.coordinateToPrice(e.clientY - r.top);
      if (raw == null || !isFinite(raw) || raw <= 0) return;
      const price = roundPx(raw);
      C.drag.moved = true;
      const line = C.drag.kind === "order" ? C.orderLines.get(C.drag.id) : C.hLines.get(C.drag.id);
      if (line) { line.applyOptions({ price, title: C.drag.kind === "order" ? `${C.drag.title} ${fmt(price)}` : fmt(price) }); C.drag.price = price; }
    });
    window.addEventListener("mouseup", () => {
      if (!C.drag) return;
      const d = C.drag; C.drag = null;
      C.chart.applyOptions({ handleScroll: true, handleScale: true });
      if (!d.moved || d.price === d.from) return;              // nothing moved → nothing to send
      if (d.kind === "order") { C.pending.set(d.id, d.price); send({ type: "order_move", order_id: d.id, price: d.price, from: d.from }); }   // Rust asks for confirmation; the line stays where it was dropped meanwhile
      else { saveHLines(); }
    });
    C.ready = true;
    flushReady();
  }

  // price rounded to the instrument's decimals (the grid shown on the axis)
  function roundPx(p) { const f = Math.pow(10, C.decimals); return Math.round(p * f) / f; }

  function nearestLine(y, forDrag) {
    let best = null, bestD = 7;
    const check = (kind, id, price, title) => {
      const yy = C.candles.priceToCoordinate(price);
      if (yy == null) return;
      const d = Math.abs(yy - y);
      if (d < bestD) { bestD = d; best = { kind, id, price, title }; }
    };
    for (const [id, l] of C.orderLines) { if (forDrag && !C.draggable.has(id)) continue; check("order", id, l.options().price, l.options().title.split(" ").slice(0, -1).join(" ")); }
    for (const [id, l] of C.hLines) check("hline", id, l.options().price, "");
    return best;
  }

  // ---------- data ----------
  function setData(d) {
    if (!C.chart) return;
    const newKey = d.key !== C.key || d.decimals !== C.decimals;
    C.key = d.key; C.decimals = d.decimals; C.tfSec = d.tfSec; C.intraday = d.tfSec < 86400;
    C.bars = (d.bars || []).map((b) => ({ time: toSec(b.t), open: b.o, high: b.h, low: b.l, close: b.c, v: b.v || 0 }))
      .filter((b, i, a) => i === 0 || b.time > a[i - 1].time);
    C.candles.applyOptions({ priceFormat: { type: "price", precision: C.decimals, minMove: Math.pow(10, -C.decimals) } });
    C.chart.applyOptions({ timeScale: { timeVisible: C.intraday } });
    C.candles.setData(C.bars.map(({ time, open, high, low, close }) => ({ time, open, high, low, close })));
    C.volume.setData(C.ind.volume ? C.bars.map((b) => ({ time: b.time, value: Math.max(b.v, 0), color: b.close >= b.open ? "rgba(46,229,122,.28)" : "rgba(255,77,94,.28)" })) : []);
    recomputeIndicators();
    applyMarkers();
    if (newKey) { C.chart.timeScale().setVisibleLogicalRange({ from: Math.max(0, C.bars.length - 150), to: C.bars.length + 3 }); loadHLines(); }
  }

  function tick(price, tsSec) {
    // tsSec is the quote's own timestamp: a quote older than the last bar (market closed, stale feed) changes nothing
    if (!C.chart || !C.bars.length || !price || !isFinite(price) || !tsSec) return;
    const last = C.bars[C.bars.length - 1];
    if (tsSec < last.time) return;
    const bucket = C.intraday ? Math.floor(tsSec / C.tfSec) * C.tfSec : last.time;
    if (bucket > last.time && C.intraday) {
      const nb = { time: bucket, open: price, high: price, low: price, close: price, v: 0 };
      C.bars.push(nb); C.candles.update({ time: nb.time, open: nb.open, high: nb.high, low: nb.low, close: nb.close });
    } else {
      last.close = price; last.high = Math.max(last.high, price); last.low = Math.min(last.low, price);
      C.candles.update({ time: last.time, open: last.open, high: last.high, low: last.low, close: last.close });
    }
    recomputeIndicators(true);
  }

  // ---------- indicators ----------
  // EMA seeded with the SMA of the first n values (TA-Lib / TradingView); null during warm-up and for null inputs
  const ema = (vals, n) => { const k = 2 / (n + 1); const out = new Array(vals.length).fill(null); let e = null, sum = 0, cnt = 0;
    for (let i = 0; i < vals.length; i++) { const v = vals[i]; if (v == null) continue;
      if (e == null) { sum += v; cnt++; if (cnt === n) { e = sum / n; out[i] = e; } } else { e = v * k + e * (1 - k); out[i] = e; } }
    return out; };
  const sma = (vals, n) => vals.map((_, i) => i + 1 < n ? null : vals.slice(i + 1 - n, i + 1).reduce((a, b) => a + b, 0) / n);
  const std = (vals, n) => vals.map((_, i) => { if (i + 1 < n) return null; const w = vals.slice(i + 1 - n, i + 1); const m = w.reduce((a, b) => a + b, 0) / n; return Math.sqrt(w.reduce((a, b) => a + (b - m) ** 2, 0) / n); });
  // Wilder RSI: 100 with gains and no losses, 50 when the window is flat
  const rsiVal = (g, l) => (l === 0 ? (g === 0 ? 50 : 100) : 100 - 100 / (1 + g / l));
  const rsi = (vals, n) => { const out = []; let g = 0, l = 0; for (let i = 0; i < vals.length; i++) { if (i === 0) { out.push(null); continue; } const ch = vals[i] - vals[i - 1]; const up = Math.max(ch, 0), dn = Math.max(-ch, 0); if (i <= n) { g += up / n; l += dn / n; out.push(i === n ? rsiVal(g, l) : null); } else { g = (g * (n - 1) + up) / n; l = (l * (n - 1) + dn) / n; out.push(rsiVal(g, l)); } } return out; };
  const COLORS = ["#38bdf8", "#c4b5fd", "#fdba74", "#6ee7b7", "#f9a8d4"];

  function lineSeries(name, color, opts) {
    if (!C.overlays[name]) C.overlays[name] = C.chart.addLineSeries(Object.assign({ color, lineWidth: 1.5, priceLineVisible: false, lastValueVisible: false, crosshairMarkerVisible: false }, opts || {}));
    return C.overlays[name];
  }
  function clearOverlays() { for (const s of Object.values(C.overlays)) C.chart.removeSeries(s); C.overlays = {}; }
  const pts = (arr) => C.bars.map((b, i) => ({ time: b.time, value: arr[i] })).filter((p) => p.value != null && isFinite(p.value));

  function recomputeIndicators(light) {
    if (!C.chart) return;
    const closes = C.bars.map((b) => b.close);
    if (!light) clearOverlays();
    let ci = 0;
    for (const n of C.ind.ema) { lineSeries(`EMA${n}`, COLORS[ci++ % COLORS.length]).setData(pts(ema(closes, n))); }
    for (const n of C.ind.sma) { lineSeries(`SMA${n}`, COLORS[ci++ % COLORS.length], { lineStyle: 2 }).setData(pts(sma(closes, n))); }
    if (C.ind.bb) {
      const m = sma(closes, C.ind.bb.period), s = std(closes, C.ind.bb.period);
      lineSeries("BB mid", "rgba(196,181,253,.6)", { lineWidth: 1 }).setData(pts(m));
      lineSeries("BB up", "rgba(196,181,253,.35)", { lineWidth: 1 }).setData(pts(m.map((v, i) => v == null ? null : v + C.ind.bb.k * s[i])));
      lineSeries("BB dn", "rgba(196,181,253,.35)", { lineWidth: 1 }).setData(pts(m.map((v, i) => v == null ? null : v - C.ind.bb.k * s[i])));
    }
    if (C.ind.vwap) {
      let pv = 0, vv = 0, day = null; const out = [];
      // only bars with real (positive) volume count; IB FX midpoint bars carry -1 and live bars 0 → no VWAP for them
      for (const b of C.bars) { const d = Math.floor(b.time / 86400); if (d !== day) { day = d; pv = 0; vv = 0; } const tp = (b.high + b.low + b.close) / 3; const vol = b.v > 0 ? b.v : 0; pv += tp * vol; vv += vol; out.push(vv > 0 ? pv / vv : null); }
      lineSeries("VWAP", "#fcd34d", { lineWidth: 1, lineStyle: 1 }).setData(pts(out));
    }
    C.volume.applyOptions({ visible: !!C.ind.volume });
    renderSub(closes, light);
  }

  function renderSub(closes, light) {
    if (!C.subEl) return;
    const want = C.ind.sub;
    if (want === "none") { if (C.sub) { C.sub.remove(); C.sub = null; C.subSeries = {}; } C.subEl.style.display = "none"; return; }
    C.subEl.style.display = "block";
    if (!C.sub) {
      C.sub = LightweightCharts.createChart(C.subEl, Object.assign(theme(), { rightPriceScale: { borderColor: "#22304a", scaleMargins: { top: 0.1, bottom: 0.1 } }, timeScale: { visible: false } }));
      C.chart.timeScale().subscribeVisibleLogicalRangeChange((r) => { if (r && C.sub) C.sub.timeScale().setVisibleLogicalRange(r); });
      C.sub.timeScale().subscribeVisibleLogicalRangeChange((r) => { if (r) C.chart.timeScale().setVisibleLogicalRange(r); });
      C.subSeries = {};
    }
    if (!light || C.subSeries.kind !== want) { for (const k of Object.keys(C.subSeries)) if (k !== "kind") C.sub.removeSeries(C.subSeries[k]); C.subSeries = { kind: want }; }
    if (want === "rsi") {
      if (!C.subSeries.rsi) { C.subSeries.rsi = C.sub.addLineSeries({ color: "#c4b5fd", lineWidth: 1.5, priceLineVisible: false, lastValueVisible: true });
        C.subSeries.rsi.createPriceLine({ price: 70, color: "rgba(255,77,94,.5)", lineWidth: 1, lineStyle: 2, axisLabelVisible: false });
        C.subSeries.rsi.createPriceLine({ price: 30, color: "rgba(46,229,122,.5)", lineWidth: 1, lineStyle: 2, axisLabelVisible: false }); }
      C.subSeries.rsi.setData(pts(rsi(closes, 14)));
    } else if (want === "macd") {
      const f = ema(closes, 12), s = ema(closes, 26); const macd = f.map((v, i) => (v == null || s[i] == null ? null : v - s[i])); const sig = ema(macd, 9); const hist = macd.map((v, i) => (v == null || sig[i] == null ? null : v - sig[i]));
      if (!C.subSeries.hist) { C.subSeries.hist = C.sub.addHistogramSeries({ priceLineVisible: false, lastValueVisible: false });
        C.subSeries.macd = C.sub.addLineSeries({ color: "#38bdf8", lineWidth: 1.5, priceLineVisible: false });
        C.subSeries.sig = C.sub.addLineSeries({ color: "#fdba74", lineWidth: 1, priceLineVisible: false }); }
      C.subSeries.hist.setData(C.bars.map((b, i) => ({ time: b.time, value: hist[i], color: hist[i] >= 0 ? "rgba(46,229,122,.5)" : "rgba(255,77,94,.5)" })).filter((p) => p.value != null));
      C.subSeries.macd.setData(pts(macd)); C.subSeries.sig.setData(pts(sig));
    }
    const r = C.chart.timeScale().getVisibleLogicalRange(); if (r) C.sub.timeScale().setVisibleLogicalRange(r);
  }

  function setIndicators(cfg) { C.ind = Object.assign(C.ind, cfg); recomputeIndicators(false); }

  // ---------- markers / lines ----------
  function applyMarkers() {
    if (!C.candles) return;
    const snap = (t) => { let best = null; for (const b of C.bars) { if (b.time <= t) best = b.time; else break; } return best; };
    const ms = C.markers.map((m) => { const t = snap(toSec(m.t)); if (t == null) return null;
      return { time: t, position: m.side === "BUY" ? "belowBar" : "aboveBar", color: m.side === "BUY" ? "#2ee57a" : "#ff4d5e",
               shape: m.side === "BUY" ? "arrowUp" : "arrowDown", text: `${m.side} ${m.qty} @ ${fmt(m.price)}` }; }).filter(Boolean)
      .sort((a, b) => a.time - b.time);
    C.candles.setMarkers(ms);
  }
  function setMarkers(list) { C.markers = list || []; applyMarkers(); }

  function setLines(d) {
    if (!C.candles) return;
    if (C.posLine) { C.candles.removePriceLine(C.posLine); C.posLine = null; }
    if (d.position && d.position.qty) {
      const p = d.position; const up = p.pnl >= 0;
      C.posLine = C.candles.createPriceLine({ price: p.price, color: up ? "#2ee57a" : "#ff4d5e", lineWidth: 1, lineStyle: 0, axisLabelVisible: true,
        title: `POS ${p.qty > 0 ? "+" : ""}${p.qty} · ${up ? "+" : ""}${p.pnl.toFixed(2)}${p.ccy ? " " + p.ccy : ""}` });
    }
    const keep = new Set();
    C.draggable = new Set();
    for (const o of d.orders || []) {
      keep.add(o.id);
      if (o.draggable) C.draggable.add(o.id);
      const col = o.kind === "TP" ? "#6ee7b7" : o.kind === "SL" ? "#fda4af" : o.side === "BUY" ? "#38bdf8" : "#fdba74";
      const title = `${o.kind} ${o.side} ${o.qty}`;
      // a line waiting for the "Move order?" answer keeps its dropped price until the order really moves (or moveLine restores it)
      if (C.pending.has(o.id) && Math.abs(C.pending.get(o.id) - o.price) < 1e-12) C.pending.delete(o.id);
      if (C.orderLines.has(o.id)) { if ((!C.drag || C.drag.id !== o.id) && !C.pending.has(o.id)) C.orderLines.get(o.id).applyOptions({ price: o.price, title: `${title} ${fmt(o.price)}` }); }
      else C.orderLines.set(o.id, C.candles.createPriceLine({ price: o.price, color: col, lineWidth: 1, lineStyle: 2, axisLabelVisible: true, title: `${title} ${fmt(o.price)}` }));
    }
    for (const [id, l] of C.orderLines) if (!keep.has(id)) { C.candles.removePriceLine(l); C.orderLines.delete(id); C.pending.delete(id); }
  }

  // ---------- horizontal lines (per instrument, localStorage) ----------
  const hkey = () => `qx.hlines.${C.key}`;
  function addHLine(price) { const id = "h" + Date.now(); C.hLines.set(id, C.candles.createPriceLine({ price, color: "#fcd34d", lineWidth: 1, lineStyle: 3, axisLabelVisible: true, title: fmt(price) })); saveHLines(); }
  function removeHLine(id) { const l = C.hLines.get(id); if (l) { C.candles.removePriceLine(l); C.hLines.delete(id); saveHLines(); } }
  function saveHLines() { try { localStorage.setItem(hkey(), JSON.stringify([...C.hLines.values()].map((l) => l.options().price))); } catch (e) {} }
  function loadHLines() { for (const l of C.hLines.values()) C.candles.removePriceLine(l); C.hLines.clear();
    try { for (const p of JSON.parse(localStorage.getItem(hkey()) || "[]")) { const id = "h" + Math.random().toString(36).slice(2); C.hLines.set(id, C.candles.createPriceLine({ price: p, color: "#fcd34d", lineWidth: 1, lineStyle: 3, axisLabelVisible: true, title: fmt(p) })); } } catch (e) {} }

  // put an order line back (e.g. the user cancelled the move confirmation)
  function moveLine(id, price) { C.pending.delete(id); const l = C.orderLines.get(id); if (l) { const t = l.options().title.split(" "); t.pop(); l.applyOptions({ price, title: `${t.join(" ")} ${fmt(price)}` }); } }

  function zoom(f) { const ts = C.chart.timeScale(); const r = ts.getVisibleLogicalRange(); if (!r) return; const mid = (r.from + r.to) / 2, half = (r.to - r.from) / 2 * f; ts.setVisibleLogicalRange({ from: mid - half, to: mid + half }); }
  function fit() { C.chart.timeScale().setVisibleLogicalRange({ from: Math.max(0, C.bars.length - 150), to: C.bars.length + 3 }); }
  function resize() { if (C.chart) C.chart.applyOptions({ width: C.el.clientWidth, height: C.el.clientHeight }); if (C.sub) C.sub.applyOptions({ width: C.subEl.clientWidth, height: C.subEl.clientHeight }); }

  function destroy() { if (C.sub) { C.sub.remove(); C.sub = null; } if (C.chart) { C.chart.remove(); C.chart = null; }
    C.overlays = {}; C.subSeries = {}; C.orderLines = new Map(); C.hLines = new Map(); C.posLine = null; C.candles = null; C.volume = null; C.ready = false; }
  // the move was confirmed and sent: follow the order's real price from the next snapshot on
  function releaseLine(id) { C.pending.delete(id); }
  return { init, setData, tick, setIndicators, setMarkers, setLines, moveLine, releaseLine, addHLine, removeHLine, zoom, fit, resize, destroy, _C: C };
  }

  // run calls queued by js_call before the script or the pane's chart was ready (entries: {p: pane, n: needs ready, run})
  function flushReady() {
    const q = window.__qxOnReady || []; window.__qxOnReady = [];
    q.forEach((e) => {
      const f = typeof e === "function" ? e : e.run;
      const ok = typeof e === "function" || !e.n || (INSTANCES[e.p] && INSTANCES[e.p]._C.ready);
      if (!ok) { window.__qxOnReady.push(e); return; }
      try { f(); } catch (x) {}
    });
    const w = window.__qxOnReady; if (w.length > 200) w.splice(0, w.length - 200);
  }
  window.qxChart = {
    get(pane) { return INSTANCES[pane] || (INSTANCES[pane] = makeInstance(pane)); },
    ready(pane) { const i = INSTANCES[pane]; return !!(i && i._C.ready); },
    destroy(pane) { if (INSTANCES[pane]) { INSTANCES[pane].destroy(); delete INSTANCES[pane]; } },
  };

  // ---------- round 6: websocket client, hotkeys, sounds ----------
  const send0 = (m) => { try { if (window.__qxSend) window.__qxSend(m); else { const q = (window.__qxQueue = window.__qxQueue || []); q.push(m); if (q.length > 50) q.shift(); } } catch (e) {} };
  const WS = { sock: null, url: window.QX_WS_URL || "ws://127.0.0.1:8000/ws", alive: false, timer: null, ping: null, backoff: 1000 };
  function wsConnect() {
    if (WS.sock && (WS.sock.readyState === 0 || WS.sock.readyState === 1)) return;
    try { WS.sock = new WebSocket(WS.url); } catch (e) { return wsRetry(); }
    WS.sock.onopen = () => { WS.alive = true; WS.backoff = 1000; send0({ type: "ws_state", open: true });
      clearInterval(WS.ping); WS.ping = setInterval(() => { try { WS.sock.send("ping"); } catch (e) {} }, 15000); };
    WS.sock.onmessage = (ev) => { try { const m = JSON.parse(ev.data); send0({ type: "ws", msg: m }); } catch (e) {} };
    WS.sock.onclose = () => { WS.alive = false; clearInterval(WS.ping); send0({ type: "ws_state", open: false }); wsRetry(); };
    WS.sock.onerror = () => { try { WS.sock.close(); } catch (e) {} };
  }
  function wsRetry() { clearTimeout(WS.timer); WS.timer = setTimeout(wsConnect, WS.backoff); WS.backoff = Math.min(WS.backoff * 1.6, 15000); }
  // the Terminal passes the address derived from BOT_API; reconnect when it differs from the default
  function wsSetUrl(u) { if (!u || u === WS.url) return; WS.url = u; try { if (WS.sock) WS.sock.close(); } catch (e) {} if (!window.QX_NO_WS) wsConnect(); }
  window.qxWs = { connect: wsConnect, setUrl: wsSetUrl, alive: () => WS.alive };
  if (!window.QX_NO_WS) wsConnect(); // web demo: no local bot, poll the demo backend instead

  // hotkeys: ignored while typing in input/textarea/select
  window.addEventListener("keydown", (e) => {
    const t = e.target, tag = (t && t.tagName || "").toLowerCase();
    const typing = tag === "input" || tag === "textarea" || tag === "select" || (t && t.isContentEditable) || !!(t && t.closest && t.closest(".dd"));
    const combo = (e.ctrlKey || e.metaKey) && e.shiftKey;
    let name = null;
    const zoomMod = !window.QX_BROWSER_ZOOM && (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey; // web: leave Ctrl/Cmd +/-/0 to the browser
    if (zoomMod && (e.key === "=" || e.key === "+")) name = "ui_zoom_in";
    else if (zoomMod && (e.key === "-" || e.key === "_")) name = "ui_zoom_out";
    else if (zoomMod && e.key === "0") name = "ui_zoom_reset";
    else if (e.key === "F1") name = "buy"; else if (e.key === "F2") name = "sell"; else if (e.key === "Escape") name = "escape";
    else if (combo && e.key.toUpperCase() === "X") name = "close_all"; else if (combo && e.key.toUpperCase() === "K") name = "kill";
    else if (combo && e.key.toUpperCase() === "F") name = "fullscreen";
    else if (!typing) {
      if (e.key === "1" || e.key === "2" || e.key === "4") name = "layout" + e.key;
      else if (e.key === "ArrowUp") name = "wl_up"; else if (e.key === "ArrowDown") name = "wl_down";
      else if (e.key === "?" || (e.shiftKey && e.key === "/")) name = "help";
      else if (e.key === "b" || e.key === "B") name = "buy"; else if (e.key === "s" || e.key === "S") name = "sell";
      else if (e.key === "+" || e.key === "=") name = "zoom_in"; else if (e.key === "-") name = "zoom_out";
      else if (e.key === "a" || e.key === "A") name = "alert";
    }
    if (e.key === "Enter" && !typing) name = "enter";
    if (!name) return;
    if (name !== "enter" || !typing) e.preventDefault();
    send0({ type: "hotkey", name });
  }, true);

  // scrollbars are hidden: a plain mouse wheel over a box that only scrolls sideways (e.g. the market cards) scrolls it left/right
  window.addEventListener("wheel", (e) => {
    if (e.shiftKey || e.ctrlKey || Math.abs(e.deltaY) <= Math.abs(e.deltaX)) return;
    for (let el = e.target; el && el !== document.body; el = el.parentElement) {
      if (el.classList && (el.classList.contains("lwc-main") || el.classList.contains("lwc-sub"))) return;   // charts zoom with the wheel
      const cs = getComputedStyle(el);
      const canY = /(auto|scroll)/.test(cs.overflowY) && el.scrollHeight > el.clientHeight + 1;
      if (canY) return;                                                                                     // normal vertical scroll
      const canX = /(auto|scroll)/.test(cs.overflowX) && el.scrollWidth > el.clientWidth + 1;
      if (canX) { el.scrollLeft += e.deltaY; e.preventDefault(); return; }
    }
  }, { passive: false });

  // sounds (WebAudio, no files needed)
  let AC = null;
  function beep(freq, ms, type, gain) {
    try { AC = AC || new (window.AudioContext || window.webkitAudioContext)(); const o = AC.createOscillator(), g = AC.createGain();
      o.type = type || "sine"; o.frequency.value = freq; g.gain.value = gain || 0.08; o.connect(g); g.connect(AC.destination);
      o.start(); g.gain.exponentialRampToValueAtTime(0.0001, AC.currentTime + ms / 1000); o.stop(AC.currentTime + ms / 1000); } catch (e) {}
  }
  window.qxSound = {
    play(kind) {
      if (kind === "fill") { beep(880, 120); setTimeout(() => beep(1320, 160), 120); }
      else if (kind === "alert") { beep(660, 180, "triangle"); setTimeout(() => beep(660, 180, "triangle"), 240); }
      else if (kind === "error") { beep(220, 260, "sawtooth", 0.06); }
      else if (kind === "info") { beep(1040, 90); }
      else if (kind === "disconnect") { beep(330, 200, "square", 0.05); setTimeout(() => beep(260, 300, "square", 0.05), 220); }
    },
  };

  flushReady();
})();
