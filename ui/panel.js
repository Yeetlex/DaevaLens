import { invoke, listen, setLang, getLang, fmt, pct, clock, esc, icon, jobColor, jobName } from "./common.js";

const S = {
  es: {
    navOverlays: "Overlays", navHistory: "Historial", navTracker: "Rastreador", navSettings: "Ajustes", navAbout: "Acerca de", navEvents: "Eventos",
    evTitle: "Eventos", evLead: "Grietas, asedios y jefes de mundo: un aviso unos minutos antes en el overlay de avisos, y la cuenta atrás en «Próximos».",
    region: "Región del servidor", regionHint: "Los eventos en hora del servidor van con su reloj.",
    rEu: "Europa", rNaE: "Norteamérica Este", rNaW: "Norteamérica Oeste", rSa: "Sudamérica", rAsia: "Asia",
    follow: "Seguir", lead: "Avisar antes", leadMin: "min", times: "Horas", timesHint: "HH:MM separadas por comas", daysL: "Días", everyDay: "todos los días",
    clockL: "Reloj", clockServer: "servidor", clockUtc: "UTC", durationL: "Abierto", unconfirmed: "Hora sin confirmar: compruébala en el juego y corrígela aquí si hace falta.",
    restore: "Restaurar", addEvent: "Añadir evento", newEvent: "Evento nuevo", nextIn: "Próximo: {when}", openNowP: "abierto ahora", del2: "Borrar", quit: "Salir de DaevaLens", quitAsk: "Pulsa otra vez para salir",
    ovTitle: "Overlays", ovLead: "Cada overlay es una ventana independiente encima del juego. Actívalos, ajusta su aspecto y colócalos en modo edición.",
    edit: "Modo edición", editOn: "Salir del modo edición", resetPos: "Recolocar todos", opacity: "Opacidad", scale: "Tamaño",
    w_dps: "DPS", w_dps_d: "Tu DPS y el de tu grupo, con barras por jugador.",
    w_target: "Objetivo", w_target_d: "Vida del objetivo y tiempo estimado hasta que muera.",
    w_tracker: "Rastreador", w_tracker_d: "Buffs y debuffs que sigues: activo, cargas y tiempo activo.",
    w_skills: "Habilidades", w_skills_d: "Tu daño por habilidad en el combate actual.",
    w_summary: "Resumen", w_summary_d: "Tarjeta con los números del último combate.",
    w_alerts: "Avisos", w_alerts_d: "Aviso grande cuando se cae un debuff o muere alguien.",
    w_info: "Estado", w_info_d: "Barrita con conexión, ping y tiempo de combate.",
    w_events: "Próximos", w_events_d: "Cuenta atrás de los eventos que sigues (grietas, asedios, jefes).",
    hTitle: "Historial", hLead: "Todos tus combates de más de 3 segundos. Pulsa uno para ver el análisis.",
    when: "Cuándo", target: "Objetivo", duration: "Duración", myDps: "Tu DPS", groupDps: "DPS grupo", players: "Jugadores",
    none: "Todavía no hay combates guardados.", back: "← Volver", del: "Borrar",
    damage: "Daño", hits: "Golpes", crit: "Crít.", backHit: "Espalda", maxHit: "Mejor golpe", share: "Parte", dps: "DPS",
    kills: "Enemigos muertos", hps: "CPS", heal: "Curación", myHps: "Tu curación/s",
    learnHint: "Tu clase no tiene lista predefinida: tras tu primer combate de más de 10 s, DaevaLens añadirá solo los buffs y debuffs que más mantuviste.", deaths: "Caídos", bestHit: "Mejor golpe", critRate: "Críticos", summonShare: "Invocaciones",
    timeline: "DPS durante el combate", you: "Tú", group: "Grupo", skillsT: "Tus habilidades", playersT: "Jugadores", uptimes: "Rastreador",
    uptime: "Tiempo activo", onSelf: "en ti", onTarget: "en el objetivo",
    tTitle: "Rastreador", tLead: "Elige qué buffs (en ti) y debuffs (en tu objetivo, puestos por ti) quieres seguir.",
    name: "Nombre", on: "Dónde", alert: "Avisar si cae", stacks: "Mostrar cargas", add: "Añadir", remove: "Quitar",
    seen: "Vistos en los últimos 10 minutos (pulsa para añadir)", seenNone: "Lucha un poco y aparecerán aquí.",
    preset: "Cargar la lista de mi clase", manual: "Añadir a mano",
    sTitle: "Ajustes", language: "Idioma", restart: "Se aplica al reiniciar DaevaLens.",
    idle: "Fin del combate tras", idleHint: "Segundos sin daño tuyo ni de tu grupo.", keep: "Mantener el resultado", keepHint: "Segundos que el último combate sigue en pantalla.",
    onlyMine: "Solo mis objetivos", onlyMineHint: "Cuenta el daño de los demás solo en los enemigos que golpeas tú.",
    hide: "Ocultar sin el juego delante", alerts: "Avisos",
    withGame: "Overlays solo con el juego abierto", withGameHint: "Aparecen al abrir AION 2 y se esconden al cerrarlo.",
    login: "Abrir al iniciar sesión", loginHint: "Se queda en segundo plano hasta que abras el juego.",
    noCapture: "No se pueden leer los paquetes de red.", npcap: "En Windows hace falta Npcap (gratuito).", getNpcap: "Descargar Npcap",
    linuxCap: "En Linux, el programa necesita permiso de captura. Instálalo con el paquete o ejecuta:", sound: "Sonido en los avisos", limit: "Combates guardados", seconds: "s",
    aTitle: "Acerca de DaevaLens", capture: "Captura", game: "Juego", dataFiles: "Datos", ok: "correcto", missing: "falta", seenGame: "detectado", notSeen: "esperando",
  },
  en: {
    navOverlays: "Overlays", navHistory: "History", navTracker: "Tracker", navSettings: "Settings", navAbout: "About", navEvents: "Events",
    evTitle: "Events", evLead: "Rifts, sieges and world bosses: an alert a few minutes before in the alerts overlay, and a countdown in “Upcoming”.",
    region: "Server region", regionHint: "Events on server time follow its clock.",
    rEu: "Europe", rNaE: "North America East", rNaW: "North America West", rSa: "South America", rAsia: "Asia",
    follow: "Follow", lead: "Warn before", leadMin: "min", times: "Times", timesHint: "HH:MM separated by commas", daysL: "Days", everyDay: "every day",
    clockL: "Clock", clockServer: "server", clockUtc: "UTC", durationL: "Open", unconfirmed: "Unconfirmed time: check it in game and fix it here if needed.",
    restore: "Restore", addEvent: "Add event", newEvent: "New event", nextIn: "Next: {when}", openNowP: "open now", del2: "Delete", quit: "Quit DaevaLens", quitAsk: "Click again to quit",
    ovTitle: "Overlays", ovLead: "Each overlay is its own window on top of the game. Turn them on, style them, and place them in edit mode.",
    edit: "Edit mode", editOn: "Leave edit mode", resetPos: "Re-stack all", opacity: "Opacity", scale: "Size",
    w_dps: "DPS", w_dps_d: "Your DPS and your group's, with a bar per player.",
    w_target: "Target", w_target_d: "Target health and estimated time to kill.",
    w_tracker: "Tracker", w_tracker_d: "Buffs and debuffs you follow: up, stacks and uptime.",
    w_skills: "Skills", w_skills_d: "Your damage per skill in the current fight.",
    w_summary: "Summary", w_summary_d: "A card with the numbers of the last fight.",
    w_alerts: "Alerts", w_alerts_d: "Big alert when a debuff drops or someone dies.",
    w_info: "Status", w_info_d: "Small bar with connection, ping and fight time.",
    w_events: "Upcoming", w_events_d: "Countdown to the events you follow (rifts, sieges, bosses).",
    hTitle: "History", hLead: "Every fight longer than 3 seconds. Click one for the analysis.",
    when: "When", target: "Target", duration: "Duration", myDps: "Your DPS", groupDps: "Group DPS", players: "Players",
    none: "No fights saved yet.", back: "← Back", del: "Delete",
    damage: "Damage", hits: "Hits", crit: "Crit", backHit: "Back", maxHit: "Best hit", share: "Share", dps: "DPS",
    kills: "Kills", hps: "HPS", heal: "Healing", myHps: "Your HPS",
    learnHint: "Your class has no preset: after your first fight longer than 10 s, DaevaLens adds the buffs and debuffs you kept up the most.", deaths: "Deaths", bestHit: "Best hit", critRate: "Crit rate", summonShare: "Summons",
    timeline: "DPS over the fight", you: "You", group: "Group", skillsT: "Your skills", playersT: "Players", uptimes: "Tracker",
    uptime: "Uptime", onSelf: "on you", onTarget: "on target",
    tTitle: "Tracker", tLead: "Pick the buffs (on you) and debuffs (on your target, applied by you) to follow.",
    name: "Name", on: "Where", alert: "Alert when it drops", stacks: "Show stacks", add: "Add", remove: "Remove",
    seen: "Seen in the last 10 minutes (click to add)", seenNone: "Fight a bit and they will show up here.",
    preset: "Load my class list", manual: "Add by hand",
    sTitle: "Settings", language: "Language", restart: "Applies after restarting DaevaLens.",
    idle: "End the fight after", idleHint: "Seconds without damage from you or your group.", keep: "Keep the result", keepHint: "Seconds the last fight stays on screen.",
    onlyMine: "Only my targets", onlyMineHint: "Count others' damage only on enemies you hit yourself.",
    hide: "Hide when the game is not in front", alerts: "Alerts",
    withGame: "Overlays only while the game runs", withGameHint: "They appear when AION 2 starts and hide when it closes.",
    login: "Start when I log in", loginHint: "Stays in the background until you open the game.",
    noCapture: "Network packets cannot be read.", npcap: "Windows needs Npcap (free).", getNpcap: "Download Npcap",
    linuxCap: "On Linux the program needs the capture permission. Install the package, or run:", sound: "Alert sound", limit: "Fights kept", seconds: "s",
    aTitle: "About DaevaLens", capture: "Capture", game: "Game", dataFiles: "Data", ok: "ok", missing: "missing", seenGame: "detected", notSeen: "waiting",
  },
};
const L = (k) => S[getLang()]?.[k] ?? S.en[k] ?? k;
const WIDGETS = ["dps", "target", "tracker", "skills", "summary", "alerts", "info", "events"];

let settings = await invoke("get_settings");
setLang(settings.language);
let page = "overlays";
let detailId = null; // a fight open in the history page
const main = document.getElementById("main");

async function save(mutate) {
  mutate(settings);
  settings = await invoke("save_settings", { settings });
}

const sw = (id, on) => `<label class="switch"><input type="checkbox" id="${id}" ${on ? "checked" : ""}><span></span></label>`;

// ---------- pages ----------
function pageOverlays() {
  main.innerHTML = `<h1>${L("ovTitle")}</h1><p class="lead">${L("ovLead")}</p>
    <div class="toolbar">
      <button class="btn primary" id="edit">${settings.locked ? L("edit") : L("editOn")}</button>
      <button class="btn" id="restack">${L("resetPos")}</button>
    </div>
    <div class="cards">${WIDGETS.map((w) => {
      const c = settings.overlays[w];
      return `<div class="card">
        <h3>${L("w_" + w)} ${sw("en-" + w, c.enabled)}</h3><p>${L("w_" + w + "_d")}</p>
        <div class="field"><label>${L("opacity")}</label><input type="range" min="0" max="100" value="${c.opacity}" id="op-${w}"></div>
        <div class="field"><label>${L("scale")}</label><input type="range" min="60" max="180" step="5" value="${c.scale}" id="sc-${w}"></div>
      </div>`;
    }).join("")}</div>`;
  main.querySelector("#edit").onclick = async () => { await invoke("toggle_edit"); settings = await invoke("get_settings"); render(); };
  main.querySelector("#restack").onclick = () => save((s) => { for (const w of WIDGETS) { s.overlays[w].x = null; s.overlays[w].y = null; } })
    .then(async () => { for (const w of WIDGETS) if (settings.overlays[w].enabled) { await invoke("toggle_widget", { widget: w }); await invoke("toggle_widget", { widget: w }); } });
  for (const w of WIDGETS) {
    main.querySelector(`#en-${w}`).onchange = (e) => save((s) => { s.overlays[w].enabled = e.target.checked; });
    main.querySelector(`#op-${w}`).oninput = (e) => save((s) => { s.overlays[w].opacity = +e.target.value; });
    main.querySelector(`#sc-${w}`).oninput = (e) => save((s) => { s.overlays[w].scale = +e.target.value; });
  }
}

async function pageHistory() {
  detailId = null;
  const list = await invoke("list_history");
  const date = (ms) => new Date(ms).toLocaleString(getLang(), { day: "2-digit", month: "2-digit", hour: "2-digit", minute: "2-digit" });
  main.innerHTML = `<h1>${L("hTitle")}</h1><p class="lead">${L("hLead")}</p>` + (list.length ? `
    <table><thead><tr><th>${L("when")}</th><th>${L("target")}</th><th class="r">${L("duration")}</th><th class="r">${L("myDps")}</th><th class="r">${L("groupDps")}</th><th class="r">${L("players")}</th></tr></thead>
    <tbody>${list.map((h) => `<tr class="click" data-id="${h.id}"><td class="num">${date(h.startMs)}</td>
      <td>${h.isBoss ? `<span class="badge">BOSS</span> ` : ""}${esc(h.target ?? "–")}</td><td class="r num">${clock(h.durationS)}</td>
      <td class="r num"><b>${fmt(h.myDps)}</b></td><td class="r num">${fmt(h.totalDps)}</td><td class="r num">${h.players}</td></tr>`).join("")}</tbody></table>`
    : `<div class="card">${L("none")}</div>`);
  main.querySelectorAll("tr.click").forEach((tr) => (tr.onclick = () => fightDetail(+tr.dataset.id)));
}

function chartSvg(timeline, duration) {
  // 5-second rolling DPS for you and the group.
  const n = Math.max(1, Math.ceil(duration));
  const me = new Array(n + 1).fill(0), all = new Array(n + 1).fill(0);
  for (const [s, m, a] of timeline) if (s <= n) { me[s] += m; all[s] += a; }
  const roll = (arr) => arr.map((_, i) => { let sum = 0, c = 0; for (let j = Math.max(0, i - 4); j <= i; j++) { sum += arr[j]; c++; } return sum / c; });
  const rm = roll(me), ra = roll(all);
  const max = Math.max(1, ...ra, ...rm);
  const W = Math.max(400, Math.round(main.clientWidth - 60)), H = 200, pad = 34;
  const x = (i) => pad + (i / n) * (W - pad * 2), y = (v) => H - pad - (v / max) * (H - pad * 2);
  const path = (arr) => arr.map((v, i) => `${i ? "L" : "M"}${x(i).toFixed(1)},${y(v).toFixed(1)}`).join("");
  const area = (arr) => `${path(arr)}L${x(n)},${H - pad}L${x(0)},${H - pad}Z`;
  const grid = [0.25, 0.5, 0.75, 1].map((f) => `<line x1="${pad}" x2="${W - pad}" y1="${y(max * f)}" y2="${y(max * f)}" stroke="#2a3158" stroke-dasharray="3 4"/><text x="${pad - 4}" y="${y(max * f) + 4}" fill="#5d6588" font-size="10" text-anchor="end">${fmt(max * f)}</text>`).join("");
  return `<svg class="chart" viewBox="0 0 ${W} ${H}" width="${W}" height="${H}">
    <defs><linearGradient id="g" x1="0" x2="1"><stop offset="0" stop-color="#7ee0ff"/><stop offset=".55" stop-color="#8b7bff"/><stop offset="1" stop-color="#ff7bd5"/></linearGradient>
    <linearGradient id="ga" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#8b7bff" stop-opacity=".35"/><stop offset="1" stop-color="#8b7bff" stop-opacity="0"/></linearGradient></defs>
    ${grid}<path d="${area(ra)}" fill="url(#ga)"/><path d="${path(ra)}" fill="none" stroke="#5d6588" stroke-width="1.5"/>
    <path d="${path(rm)}" fill="none" stroke="url(#g)" stroke-width="2.5"/>
    <g font-size="11" fill="#8a93b8" text-anchor="end"><rect x="${W - pad - 118}" y="10" width="14" height="3" rx="1.5" fill="url(#g)"/><text x="${W - pad - 70}" y="15">${L("you")}</text>
    <rect x="${W - pad - 58}" y="10" width="14" height="3" rx="1.5" fill="#5d6588"/><text x="${W - pad}" y="15">${L("group")}</text></g></svg>`;
}

async function fightDetail(id) {
  detailId = id;
  const f = await invoke("get_fight", { id });
  if (!f) return pageHistory();
  const m = f.me ?? {};
  const kpi = (k, v) => `<div class="kpi"><div class="k">${k}</div><div class="v num">${v}</div></div>`;
  const top = f.skills?.[0]?.damage || 1;
  main.innerHTML = `
    <div class="toolbar"><button class="btn" id="back">${L("back")}</button><span style="flex:1"></span><button class="btn danger small" id="del">${L("del")}</button></div>
    <h1>${f.target?.isBoss ? `<span class="badge">BOSS</span> ` : ""}${esc(f.target?.name ?? "–")}</h1>
    <p class="lead num">${new Date(f.startMs).toLocaleString(getLang())} · ${clock(f.durationS)}</p>
    <div class="kpis">${kpi(L("myDps"), fmt(m.dps))}${kpi(L("groupDps"), fmt(f.totalDps))}${kpi(L("damage"), fmt(m.damage))}
      ${kpi(L("bestHit"), fmt(m.bestHit))}${kpi(L("critRate"), pct(m.critRate, 1))}${kpi(L("summonShare"), pct(m.summonShare))}
      ${m.hps > 0 ? kpi(L("myHps"), fmt(m.hps)) : ""}${kpi(L("kills"), f.kills ?? 0)}${kpi(L("deaths"), (f.deaths ?? []).length)}</div>
    <div class="section-title">${L("timeline")}</div>${chartSvg(f.timeline ?? [], f.durationS)}
    ${(f.watches ?? []).length ? `<div class="section-title">${L("uptimes")}</div><table><tbody>${f.watches.map((w) => `
      <tr><td style="width:30px">${icon(w.icon)}</td><td>${esc(w.name)} <span class="muted">· ${L(w.on === "target" ? "onTarget" : "onSelf")}</span></td>
      <td class="r num cellbar" style="width:40%"><b>${pct(w.uptime)}</b><i style="width:${w.uptime * 100}%"></i></td></tr>`).join("")}</tbody></table>` : ""}
    <div class="section-title">${L("skillsT")}</div>
    <table><thead><tr><th></th><th>${L("name")}</th><th class="r">${L("damage")}</th><th class="r">${L("share")}</th><th class="r">${L("dps")}</th><th class="r">${L("hits")}</th><th class="r">${L("crit")}</th><th class="r">${L("backHit")}</th><th class="r">${L("maxHit")}</th></tr></thead>
    <tbody>${(f.skills ?? []).map((s) => `<tr><td style="width:30px">${icon(s.icon)}</td><td class="cellbar">${esc(s.name)}${s.isDot ? ' <span class="muted">DoT</span>' : ""}${s.summon ? ' <span class="muted">⟡</span>' : ""}<i style="width:${(s.damage / top) * 100}%"></i></td>
      <td class="r num">${fmt(s.damage)}</td><td class="r num">${pct(s.share, 1)}</td><td class="r num">${fmt(s.dps)}</td><td class="r num">${s.hits}</td>
      <td class="r num">${pct(s.critRate)}</td><td class="r num">${pct(s.backRate)}</td><td class="r num">${fmt(s.maxHit)}</td></tr>`).join("")}</tbody></table>
    <div class="section-title">${L("playersT")}</div>
    <table><thead><tr><th>${L("name")}</th><th class="r">${L("damage")}</th><th class="r">${L("dps")}</th><th class="r">${L("share")}</th><th class="r">${L("crit")}</th><th class="r">${L("hps")}</th></tr></thead>
    <tbody>${(f.players ?? []).map((p) => `<tr><td><span style="color:${jobColor(p.job)}">●</span> ${esc(p.name)} <span class="muted">${esc(jobName(p.job))}</span></td>
      <td class="r num">${fmt(p.damage)}</td><td class="r num"><b>${fmt(p.dps)}</b></td><td class="r num">${pct(p.share, 1)}</td><td class="r num">${pct(p.critRate)}</td><td class="r num heal">${p.hps > 0 ? fmt(p.hps) : ""}</td></tr>`).join("")}</tbody></table>`;
  main.querySelector("#back").onclick = pageHistory;
  main.querySelector("#del").onclick = async () => { await invoke("delete_fight", { id }); pageHistory(); };
}

async function pageTracker() {
  const seen = await invoke("seen_abnormals");
  const live = await invoke("get_live");
  const has = (n, on) => settings.watches.some((w) => w.name.toLowerCase() === n.toLowerCase() && w.on === on);
  main.innerHTML = `<h1>${L("tTitle")}</h1><p class="lead">${L("tLead")}</p>
    ${!settings.watches.length && live.meJob && live.meJob !== "Elementalist" ? `<div class="banner">${L("learnHint")}</div>` : ""}
    <div class="toolbar">${live.meJob ? `<button class="btn" id="preset">${L("preset")}</button>` : ""}</div>
    <table><thead><tr><th>${L("name")}</th><th>${L("on")}</th><th>${L("alert")}</th><th>${L("stacks")}</th><th></th></tr></thead>
    <tbody>${settings.watches.map((w, i) => `<tr>
      <td>${esc(w.name)}</td>
      <td><select data-on="${i}"><option value="self" ${w.on === "self" ? "selected" : ""}>${L("onSelf")}</option><option value="target" ${w.on === "target" ? "selected" : ""}>${L("onTarget")}</option></select></td>
      <td>${sw("al-" + i, w.alert)}</td><td>${sw("st-" + i, w.stacks)}</td>
      <td class="r"><button class="btn small danger" data-rm="${i}">${L("remove")}</button></td></tr>`).join("")}
      <tr><td><input type="text" id="newName" placeholder="${L("manual")}"></td><td><select id="newOn"><option value="self">${L("onSelf")}</option><option value="target">${L("onTarget")}</option></select></td><td></td><td></td><td class="r"><button class="btn small" id="addManual">${L("add")}</button></td></tr>
    </tbody></table>
    <div class="section-title">${L("seen")}</div>
    <div>${seen.filter((a) => !has(a.name, a.on)).map((a, i) => `<span class="chip" data-seen="${i}">${icon(a.icon)}${esc(a.name)} <span class="muted">· ${L(a.on === "target" ? "onTarget" : "onSelf")}</span></span>`).join("") || `<span class="muted">${L("seenNone")}</span>`}</div>`;
  const visibleSeen = seen.filter((a) => !has(a.name, a.on));
  main.querySelectorAll("[data-seen]").forEach((el) => (el.onclick = () => {
    const a = visibleSeen[+el.dataset.seen];
    save((s) => s.watches.push({ name: a.name, on: a.on, alert: a.on === "target", stacks: false })).then(pageTracker);
  }));
  main.querySelectorAll("[data-rm]").forEach((el) => (el.onclick = () => save((s) => s.watches.splice(+el.dataset.rm, 1)).then(pageTracker)));
  main.querySelectorAll("[data-on]").forEach((el) => (el.onchange = () => save((s) => { s.watches[+el.dataset.on].on = el.value; })));
  settings.watches.forEach((_, i) => {
    main.querySelector(`#al-${i}`).onchange = (e) => save((s) => { s.watches[i].alert = e.target.checked; });
    main.querySelector(`#st-${i}`).onchange = (e) => save((s) => { s.watches[i].stacks = e.target.checked; });
  });
  main.querySelector("#addManual").onclick = () => {
    const name = main.querySelector("#newName").value.trim();
    if (name) save((s) => s.watches.push({ name, on: main.querySelector("#newOn").value, alert: false, stacks: false })).then(pageTracker);
  };
  main.querySelector("#preset")?.addEventListener("click", async () => { settings = await invoke("load_class_preset"); pageTracker(); });
}

async function pageEvents() {
  const DAYS = getLang() === "es" ? ["L", "M", "X", "J", "V", "S", "D"] : ["M", "T", "W", "T", "F", "S", "S"];
  const up = await invoke("get_upcoming");
  const when = (id) => {
    const u = up.find((x) => x.id === id);
    if (!u) return "";
    if (u.open) return L("openNowP");
    const d = new Date(u.startMs);
    const sameDay = d.toDateString() === new Date().toDateString();
    return L("nextIn").replace("{when}", d.toLocaleString(getLang(), sameDay ? { hour: "2-digit", minute: "2-digit" } : { weekday: "short", hour: "2-digit", minute: "2-digit" }));
  };
  const regions = [["eu", "rEu"], ["na-east", "rNaE"], ["na-west", "rNaW"], ["sa", "rSa"], ["asia", "rAsia"]];
  main.innerHTML = `<h1>${L("evTitle")}</h1><p class="lead">${L("evLead")}</p>
    <div class="toolbar" style="align-items:center">
      <label class="muted">${L("region")}</label>
      <select id="region">${regions.map(([v, k]) => `<option value="${v}" ${settings.region === v ? "selected" : ""}>${L(k)}</option>`).join("")}</select>
      <span class="muted" style="font-size:12px">${L("regionHint")}</span>
      <span style="flex:1"></span><button class="btn" id="addEv">${L("addEvent")}</button>
    </div>
    <div class="cards evcards">${settings.events.map((e, i) => `
      <div class="card ev ${e.enabled ? "" : "off"}">
        <h3><span class="evname">${e.builtin ? esc(e.name) : `<input type="text" data-k="name" data-i="${i}" value="${esc(e.name)}">`}${e.confirmed ? "" : ` <span class="unc" title="${L("unconfirmed")}">?</span>`}</span>${sw("en-ev-" + i, e.enabled)}</h3>
        <p class="next">${when(e.id)}</p>
        <div class="field"><label>${L("times")}<span class="hint">${L("timesHint")}</span></label><input type="text" class="times" data-k="times" data-i="${i}" value="${esc(e.times.join(", "))}"></div>
        <div class="field"><label>${L("daysL")}</label><span class="days">${DAYS.map((d, j) => `<button class="day ${e.days.includes(j) ? "on" : ""}" data-day="${j}" data-i="${i}">${d}</button>`).join("")}</span></div>
        <div class="field"><label>${L("clockL")}</label><select data-k="tz" data-i="${i}"><option value="server" ${e.tz === "server" ? "selected" : ""}>${L("clockServer")}</option><option value="utc" ${e.tz === "utc" ? "selected" : ""}>${L("clockUtc")}</option></select></div>
        <div class="field"><label>${L("lead")}</label><span><input type="number" min="0" max="240" data-k="leadMin" data-i="${i}" value="${e.leadMin}"> ${L("leadMin")}</span></div>
        <div class="field"><label>${L("durationL")}</label><span><input type="number" min="0" max="600" data-k="durationMin" data-i="${i}" value="${e.durationMin}"> ${L("leadMin")}</span></div>
        <div class="evfoot">${e.builtin ? `<button class="btn small" data-restore="${i}">${L("restore")}</button>` : `<button class="btn small danger" data-delev="${i}">${L("del2")}</button>`}<span class="muted" style="font-size:11px">${e.days.length ? "" : L("everyDay")}</span></div>
      </div>`).join("")}</div>`;
  const redraw = () => pageEvents();
  main.querySelector("#region").onchange = (el) => save((s) => { s.region = el.target.value; }).then(redraw);
  main.querySelector("#addEv").onclick = () => save((s) => s.events.push({ id: "custom-" + Date.now(), name: L("newEvent"), enabled: true, leadMin: 5, times: ["20:00"], days: [], tz: "server", durationMin: 10, confirmed: true, builtin: false })).then(redraw);
  settings.events.forEach((_, i) => {
    main.querySelector(`#en-ev-${i}`).onchange = (el) => save((s) => { s.events[i].enabled = el.target.checked; }).then(redraw);
  });
  main.querySelectorAll("[data-k]").forEach((el) => (el.onchange = () => {
    const i = +el.dataset.i, k = el.dataset.k;
    save((s) => {
      if (k === "times") s.events[i].times = el.value.split(/[,\s]+/).map((x) => x.trim()).filter((x) => /^\d{1,2}:\d{2}$/.test(x)).map((x) => x.padStart(5, "0"));
      else if (k === "leadMin" || k === "durationMin") s.events[i][k] = Math.max(0, Math.round(+el.value || 0));
      else s.events[i][k] = el.value;
    }).then(redraw);
  }));
  main.querySelectorAll("[data-day]").forEach((el) => (el.onclick = () => {
    const i = +el.dataset.i, d = +el.dataset.day;
    save((s) => { const days = s.events[i].days; const at = days.indexOf(d); if (at >= 0) days.splice(at, 1); else days.push(d); days.sort(); }).then(redraw);
  }));
  main.querySelectorAll("[data-restore]").forEach((el) => (el.onclick = async () => {
    const i = +el.dataset.restore;
    const orig = await invoke("builtin_event", { id: settings.events[i].id });
    if (orig) save((s) => { s.events[i] = { ...orig, enabled: s.events[i].enabled }; }).then(redraw);
  }));
  main.querySelectorAll("[data-delev]").forEach((el) => (el.onclick = () => save((s) => s.events.splice(+el.dataset.delev, 1)).then(redraw)));
}

function pageSettings() {
  const num = (id, v, min, max) => `<input type="number" id="${id}" min="${min}" max="${max}" value="${v}">`;
  const row = (label, hint, ctl) => `<div class="field"><label>${label}${hint ? `<span class="hint">${hint}</span>` : ""}</label>${ctl}</div>`;
  main.innerHTML = `<h1>${L("sTitle")}</h1><p class="lead"></p><div class="cards">
    <div class="card">
      ${row(L("language"), L("restart"), `<select id="lang"><option value="es" ${settings.language === "es" ? "selected" : ""}>Español</option><option value="en" ${settings.language === "en" ? "selected" : ""}>English</option></select>`)}
      ${row(L("idle"), L("idleHint"), num("idle", settings.idleResetS, 2, 120) + " " + L("seconds"))}
      ${row(L("keep"), L("keepHint"), num("keep", settings.keepResultS, 0, 600) + " " + L("seconds"))}
      ${row(L("onlyMine"), L("onlyMineHint"), sw("onlyMine", settings.onlyMyTargets))}
    </div>
    <div class="card">
      ${row(L("withGame"), L("withGameHint"), sw("withGame", settings.onlyWithGame))}
      ${row(L("login"), L("loginHint"), sw("login", settings.launchAtLogin))}
      ${row(L("hide"), "", sw("hide", settings.hideUnfocused))}
      ${row(L("alerts"), "", sw("alerts", settings.alerts))}
      ${row(L("sound"), "", sw("sound", settings.alertSound))}
      ${row(L("limit"), "", num("limit", settings.historyLimit, 10, 5000))}
    </div></div>`;
  const on = (id, f) => (main.querySelector("#" + id).onchange = (e) => save((s) => f(s, e.target)));
  on("lang", (s, el) => { s.language = el.value; });
  on("idle", (s, el) => { s.idleResetS = Math.max(2, +el.value || 8); });
  on("keep", (s, el) => { s.keepResultS = Math.max(0, +el.value || 0); });
  on("onlyMine", (s, el) => { s.onlyMyTargets = el.checked; });
  on("hide", (s, el) => { s.hideUnfocused = el.checked; });
  on("withGame", (s, el) => { s.onlyWithGame = el.checked; });
  on("login", (s, el) => { s.launchAtLogin = el.checked; });
  on("alerts", (s, el) => { s.alerts = el.checked; });
  on("sound", (s, el) => { s.alertSound = el.checked; });
  on("limit", (s, el) => { s.historyLimit = Math.max(10, +el.value || 300); });
}

async function pageAbout() {
  const st = await invoke("status");
  main.innerHTML = `<h1>${L("aTitle")}</h1><p class="lead">v${st.version} · ${st.platform}</p>
    <div class="cards">
      <div class="card"><h3>DaevaLens</h3>
        <p>${getLang() === "es"
          ? "Overlays, combates y análisis para AION 2. Solo lee el tráfico de red que el juego recibe: no toca el cliente, no lee su memoria y no automatiza nada."
          : "Overlays, fights and analysis for AION 2. It only reads the network traffic the game receives: it never touches the client, reads its memory or automates anything."}</p>
        <p>${getLang() === "es" ? "El analizador del protocolo, la captura y el almacén de combate son de" : "The protocol parser, capture and combat storage are from"}
          <b>A2Tools DPS Meter</b> de <b>taengu</b> y colaboradores (GPL-3.0), usados sin modificar. DaevaLens también es GPL-3.0.</p>
      </div>
      <div class="card"><h3>${getLang() === "es" ? "Atajos" : "Shortcuts"}</h3>
        <p>${getLang() === "es" ? "Asígnalos a teclas en tu escritorio (Hyprland, KDE, GNOME…):" : "Bind them to keys in your desktop (Hyprland, KDE, GNOME…):"}</p>
        <p><code>daevalens --edit</code><br><code>daevalens --toggle dps</code><br><code>daevalens --reset</code><br><code>daevalens --panel</code></p>
      </div>
    </div>`;
}

// ---------- shell ----------
let lastStatus = null;
function banner() {
  const st = lastStatus;
  if (!st || st.captureOk) return "";
  if (st.platform === "windows")
    return `<div class="banner">${L("noCapture")} ${L("npcap")} <button class="btn small" id="npcap">${L("getNpcap")}</button></div>`;
  return `<div class="banner">${L("noCapture")} ${L("linuxCap")}<br><code>sudo setcap cap_net_raw=ep "$(command -v daevalens)"</code></div>`;
}
function render() {
  document.querySelectorAll("[data-i]").forEach((el) => (el.textContent = L(el.dataset.i)));
  document.querySelectorAll("nav.side button[data-page]").forEach((b) => b.classList.toggle("active", b.dataset.page === page));
  Promise.resolve(({ overlays: pageOverlays, history: pageHistory, tracker: pageTracker, events: pageEvents, settings: pageSettings, about: pageAbout })[page]())
    .then(() => {
      main.insertAdjacentHTML("afterbegin", banner());
      main.querySelector("#npcap")?.addEventListener("click", () => invoke("open_url", { url: "https://npcap.com/#download" }));
    });
}
document.querySelectorAll("nav.side button[data-page]").forEach((b) => (b.onclick = () => { page = b.dataset.page; render(); }));
// Two clicks: the first one asks.
let quitArmed = null;
document.getElementById("quit").onclick = (e) => {
  if (quitArmed) return invoke("quit");
  e.currentTarget.textContent = L("quitAsk");
  quitArmed = setTimeout(() => { quitArmed = null; e.currentTarget.textContent = L("quit"); }, 3000);
};

async function refreshStatus() {
  const st = await invoke("status");
  const changed = lastStatus && lastStatus.captureOk !== st.captureOk;
  lastStatus = st;
  if (changed) render();
  const line = (ok, k, v) => `<div><span class="dot ${ok ? "ok" : ""}"></span>${L(k)}: ${v}</div>`;
  document.getElementById("status").innerHTML =
    line(st.captureOk, "capture", L(st.captureOk ? "ok" : "missing")) +
    line(st.gameSeen, "game", L(st.gameSeen ? "seenGame" : "notSeen")) +
    line(st.dataFound, "dataFiles", L(st.dataFound ? "ok" : "missing"));
}
await refreshStatus();
setInterval(refreshStatus, 3000);

listen("settings-changed", (s) => {
  const relocale = s.language !== settings.language;
  const relock = s.locked !== settings.locked;
  settings = s;
  if (relocale) setLang(s.language);
  // Not on every change: re-rendering under a dragged slider would drop it.
  if (relocale || (relock && page === "overlays")) render();
});
listen("fight-ended", () => { if (page === "history" && detailId == null) pageHistory(); });
render();
