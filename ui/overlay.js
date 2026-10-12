import { invoke, listen, currentWindow, t, setLang, getLang, fmt, pct, clock, esc, icon, jobColor, capacity } from "./common.js";

const widget = new URLSearchParams(location.search).get("w") || "dps";
const root = document.getElementById("w");
const win = currentWindow();
let settings = null;
let live = null;
let lastFight = null; // the last finished fight, for the summary card

// ---------- edit mode: move and resize ----------
root.addEventListener("mousedown", (e) => {
  if (document.body.classList.contains("edit") && e.button === 0) win.startDragging();
});
document.getElementById("grip").addEventListener("mousedown", (e) => {
  e.stopPropagation();
  win.startResizeDragging("SouthEast");
});

function applySettings(s) {
  settings = s;
  setLang(s.language);
  const cfg = s.overlays[widget] || {};
  document.documentElement.style.setProperty("--alpha", (cfg.opacity ?? 70) / 100);
  document.body.style.zoom = (cfg.scale ?? 100) / 100;
  document.body.classList.toggle("edit", !s.locked);
  document.querySelector(".edit-hint").textContent = t("editHint");
  render();
}

// ---------- widgets ----------
const header = (title, right = "") =>
  `<div class="whead"><span class="title">${esc(title)}</span><span class="spacer"></span>${right}</div>`;

function phasePill(state) {
  const f = state?.fight;
  if (state?.phase === "fight") return `<span class="pill live num">${clock(f?.durationS)}</span>`;
  if (state?.phase === "result") return `<span class="pill num">${t("result")} · ${clock(f?.durationS)}</span>`;
  return `<span class="pill">${t(state?.gameSeen ? "idle" : "waiting")}</span>`;
}

// DPS: keyed rows so the bars animate instead of being rebuilt.
const rowEls = new Map();
function renderDps() {
  const f = live?.fight;
  if (!root.querySelector(".rows")) {
    root.innerHTML = `<div class="head"></div><div class="rows"></div>`;
    rowEls.clear();
  }
  const total = f ? `<span class="num muted">${t("total")}</span> <b class="num">${fmt(f.totalDps)}</b>` : "";
  root.querySelector(".head").innerHTML =
    header("DPS", phasePill(live)) +
    (f?.me ? `<div style="display:flex;align-items:baseline;gap:8px"><span class="big num">${fmt(f.me.dps)}</span><span class="muted num">${fmt(f.me.damage)} · ${t("crit")} ${pct(f.me.critRate)}</span><span class="spacer" style="flex:1"></span>${total}</div>` : "");
  root.classList.toggle("faded", live?.phase === "result");
  const rows = root.querySelector(".rows");
  const players = f?.players ?? [];
  if (!players.length) {
    rows.innerHTML = `<div class="empty">${t(live?.gameSeen ? "noData" : "waiting")}</div>`;
    rowEls.clear();
    return;
  }
  rows.querySelector(".empty")?.remove();
  const top = players[0].dps || 1;
  const seen = new Set();
  const fit = capacity(rows, 22, 3);
  let shown = players.slice(0, fit);
  const meIdx = players.findIndex((p) => p.isMe);
  if (meIdx >= fit) shown[fit - 1] = players[meIdx];
  shown.forEach((p) => {
    const i = players.indexOf(p);
    const slot = shown.indexOf(p);
    let el = rowEls.get(p.id);
    if (!el) {
      el = document.createElement("div");
      el.className = "row";
      el.innerHTML = `<div class="fill"></div><span class="rank num"></span><span class="name"></span><span class="val num"></span><span class="sub num"></span>`;
      rowEls.set(p.id, el);
    }
    el.classList.toggle("me", p.isMe);
    const fill = el.querySelector(".fill");
    fill.style.width = `${Math.max(2, (p.dps / top) * 100)}%`;
    fill.style.background = jobColor(p.job);
    el.querySelector(".rank").textContent = i + 1;
    el.querySelector(".name").textContent = p.name;
    el.querySelector(".val").textContent = fmt(p.dps);
    el.querySelector(".sub").textContent = pct(p.share);
    if (rows.children[slot] !== el) rows.insertBefore(el, rows.children[slot] || null);
    seen.add(p.id);
  });
  for (const [id, el] of rowEls) if (!seen.has(id)) { el.remove(); rowEls.delete(id); }
}

function renderTarget() {
  const tg = live?.fight?.target;
  root.classList.toggle("faded", live?.phase === "result");
  if (!tg) { root.innerHTML = `<div class="empty">${t("noData")}</div>`; return; }
  const p = tg.pct;
  const label = p != null ? `${(p * 100).toFixed(1)}%${tg.hp != null ? ` · ${fmt(tg.hp)}` : ""}` : fmt(tg.hp);
  const ttk = tg.dead ? "✝" : tg.ttkS != null && isFinite(tg.ttkS) ? `${t("ttk")} ${clock(tg.ttkS)}` : "";
  root.innerHTML = `
    <div class="tname">${tg.isBoss ? `<span class="badge">${t("boss")}</span>` : ""}<span class="n">${esc(tg.name)}</span><span class="spacer" style="flex:1"></span><span class="muted num" style="font-size:11px">${ttk}</span></div>
    <div class="hpbar"><div class="hp" style="width:${((p ?? 0) * 100).toFixed(2)}%"></div><div class="lbl num">${label}</div></div>`;
}

const alarms = new Map(); // watch key -> until ms
function renderTracker() {
  const list = live?.tracker ?? [];
  if (!settings?.watches?.length) { root.innerHTML = `<div class="empty">${t("trackerEmpty")}</div>`; return; }
  const now = Date.now();
  root.innerHTML = `<div class="list"></div>`;
  const box = root.firstChild;
  box.innerHTML = list.slice(0, capacity(box, 28, 2)).map((w) => {
    const key = `${w.on}|${w.name}`;
    const alarm = (alarms.get(key) ?? 0) > now && !w.active;
    let right = `<span class="up num">${pct(w.uptime)}</span>`;
    if (w.showStacks) {
      const max = Math.max(w.maxStacks, 1);
      const ready = w.stacks >= Math.min(4, max);
      right = `<span class="pips ${ready ? "ready" : ""}">${Array.from({ length: max }, (_, i) => `<span class="pip ${i < w.stacks ? "on" : ""}"></span>`).join("")}</span>`;
    }
    return `<div class="watch ${w.active ? "" : "off"} ${alarm ? "alarm" : ""}">${icon(w.icon)}<span class="wn">${esc(w.name)}</span>${right}</div>`;
  }).join("");
}

function renderSkills() {
  const f = live?.fight;
  root.classList.toggle("faded", live?.phase === "result");
  const skills = f?.skills ?? [];
  if (!skills.length) { root.innerHTML = header(t("damage"), phasePill(live)) + `<div class="empty">${t("noData")}</div>`; return; }
  const top = skills[0].damage || 1;
  root.innerHTML = header(t("damage"), phasePill(live)) + `<div class="list"></div>`;
  const box = root.querySelector(".list");
  box.innerHTML = skills.slice(0, capacity(box, 24, 2)).map((s) => `
    <div class="skill">${icon(s.icon)}
      <span class="sn">${esc(s.name)}${s.isDot ? `<span class="tag">${t("dot")}</span>` : ""}${s.summon ? `<span class="tag">${t("summon")}</span>` : ""}<span class="sbar" style="width:${(s.damage / top) * 100}%"></span></span>
      <span class="num">${fmt(s.dps)}</span><span class="num muted" style="text-align:right">${pct(s.share)}</span>
    </div>`).join("");
}

function renderSummary(animate = false) {
  const f = lastFight;
  if (!f) { root.innerHTML = header(t("result")) + `<div class="empty">${t("noData")}</div>`; return; }
  const m = f.me ?? {};
  const stat = (k, v) => `<div class="stat"><div class="k">${k}</div><div class="v num">${v}</div></div>`;
  root.innerHTML = header(`${t("result")} · ${esc(f.target?.name ?? "")}`, `<span class="pill num">${clock(f.durationS)}</span>`) +
    `<div class="sumgrid">
      ${stat(t("myDps"), fmt(m.dps))}${stat(t("groupDps"), fmt(f.totalDps))}${stat(t("damage"), fmt(m.damage))}
      ${stat(t("bestHit"), fmt(m.bestHit))}${stat(t("critRate"), pct(m.critRate))}${stat(t("kills"), f.kills ?? 0)}
    </div>`;
  if (animate) { root.classList.remove("pop"); void root.offsetWidth; root.classList.add("pop"); }
  root.classList.toggle("faded", live?.phase === "fight");
}

let upcoming = [];
function countdown(ms) {
  const s = Math.max(0, Math.round(ms / 1000));
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = s % 60;
  return h ? `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}` : `${m}:${String(sec).padStart(2, "0")}`;
}
function renderEvents() {
  if (!upcoming.length) { root.innerHTML = header(t("upcoming")) + `<div class="empty">${t("noEvents")}</div>`; return; }
  const now = Date.now();
  root.innerHTML = header(t("upcoming")) + `<div class="list"></div>`;
  const box = root.querySelector(".list");
  box.innerHTML = upcoming.slice(0, capacity(box, 26, 2)).map((u) => {
    const left = u.startMs - now;
    const soon = !u.open && left < 5 * 60_000;
    // Over a day away a countdown reads badly ("97:12:40"): show the day and time.
    const far = left > 24 * 3600_000
      ? new Date(u.startMs).toLocaleString(getLang(), { weekday: "short", hour: "2-digit", minute: "2-digit" })
      : countdown(left);
    const right = u.open ? `<span class="pill live">${t("openNow")} ${countdown(u.endMs - now)}</span>` : `<span class="num ${soon ? "soon" : "muted"}">${far}</span>`;
    return `<div class="evt ${u.open ? "open" : ""}"><span class="en">${esc(u.name)}${u.confirmed ? "" : ' <span class="unc">?</span>'}</span>${right}</div>`;
  }).join("");
}

function renderInfo() {
  const f = live?.fight;
  root.innerHTML = `<span class="dot ${live?.gameSeen ? "ok" : ""}"></span>
    <span>${t(live?.phase ?? "idle")}</span>
    ${f && live.phase === "fight" ? `<b class="num">${clock(f.durationS)}</b>` : ""}
    <span class="spacer" style="flex:1"></span>
    <span class="muted num">${t("ping")} ${live?.pingMs ?? "–"} ms</span>`;
}

let audio = null;
function beep(kind) {
  if (!settings?.alertSound) return;
  try {
    audio ??= new AudioContext();
    const tones = kind === "death" ? [440, 330] : kind === "event" ? [660, 880, 990] : [880, 660];
    tones.forEach((freq, i) => {
      const o = audio.createOscillator(), g = audio.createGain();
      o.frequency.value = freq; o.type = "sine";
      g.gain.setValueAtTime(0.0001, audio.currentTime + i * 0.14);
      g.gain.exponentialRampToValueAtTime(0.18, audio.currentTime + i * 0.14 + 0.02);
      g.gain.exponentialRampToValueAtTime(0.0001, audio.currentTime + i * 0.14 + 0.13);
      o.connect(g).connect(audio.destination);
      o.start(audio.currentTime + i * 0.14); o.stop(audio.currentTime + i * 0.14 + 0.15);
    });
  } catch { /* no audio device */ }
}

function showAlert(a) {
  if (widget === "tracker" && a.kind === "dropped") {
    const w = (live?.tracker ?? []).find((x) => x.name === a.text);
    if (w) alarms.set(`${w.on}|${w.name}`, Date.now() + 4000);
    return;
  }
  if (widget !== "alerts") return;
  const el = document.createElement("div");
  el.className = `alert ${a.kind}`;
  const text = a.kind === "event"
    ? (a.minutes > 0 ? t("eventSoon", { name: a.text, n: a.minutes }) : t("eventNow", { name: a.text }))
    : t(a.kind === "death" ? "died" : "dropped", { name: a.text });
  el.innerHTML = `${a.icon ? icon(a.icon) : a.kind === "event" ? '<span class="aicon">⏰</span>' : ""}<span>${esc(text)}</span>`;
  root.appendChild(el);
  while (root.children.length > 3) root.firstChild.remove();
  setTimeout(() => el.remove(), 3100);
  beep(a.kind);
}

function render() {
  if (!settings) return;
  switch (widget) {
    case "dps": return renderDps();
    case "target": return renderTarget();
    case "tracker": return renderTracker();
    case "skills": return renderSkills();
    case "summary": return renderSummary();
    case "info": return renderInfo();
    case "events": return renderEvents();
    case "alerts": return; // event driven
  }
}

// ---------- wiring ----------
root.classList.add(widget);
if (widget === "alerts") root.classList.add("alerts", "bare");
if (widget === "info") root.classList.add("info");

applySettings(await invoke("get_settings"));
live = await invoke("get_live");
if (widget === "summary") {
  const [latest] = await invoke("list_history");
  if (latest) lastFight = await invoke("get_fight", { id: latest.id });
}
render();
listen("settings-changed", applySettings);
// Out of combat nothing moves: skip identical states instead of repainting.
let lastKey = "";
listen("live", (state) => {
  live = state;
  const key = state.phase === "fight" ? String(state.nowMs) : JSON.stringify([state.phase, state.fight?.id, state.tracker, state.gameSeen, state.pingMs]);
  if (key === lastKey) return;
  lastKey = key;
  render();
});
listen("fight-ended", (f) => { lastFight = f; if (widget === "summary") renderSummary(true); });
listen("alert", showAlert);
if (widget === "events") {
  upcoming = await invoke("get_upcoming");
  renderEvents();
  listen("events", (list) => { upcoming = list; renderEvents(); });
}
