// Shared helpers for the overlays and the panel.

const T = window.__TAURI__;
export const invoke = (cmd, args) => T.core.invoke(cmd, args);
export const listen = (event, cb) => T.event.listen(event, (e) => cb(e.payload));
export const currentWindow = () => T.window.getCurrentWindow();

const STRINGS = {
  es: {
    idle: "Fuera de combate", fight: "En combate", result: "Último combate",
    waiting: "Esperando al juego…", noData: "Sin datos todavía",
    total: "Total", dps: "DPS", damage: "Daño", share: "Parte", crit: "Crít.", back: "Espalda",
    hits: "Golpes", maxHit: "Mejor golpe", duration: "Duración", players: "Jugadores",
    kills: "Bajas", deaths: "Caídos", ttk: "Muere en", boss: "JEFE",
    uptime: "Activo", stacks: "Cargas", summon: "invocación", dot: "DoT",
    myDps: "Tu DPS", groupDps: "DPS grupo", bestHit: "Mejor golpe", critRate: "Críticos",
    dropped: "¡Se ha caído {name}!", died: "{name} ha caído", ping: "Ping",
    editHint: "Arrastra para mover · esquina para redimensionar",
    trackerEmpty: "Añade buffs o debuffs desde el panel",
    onSelf: "en ti", onTarget: "en el objetivo",
    heal: "Curación", hps: "CPS", myHps: "Tu curación/s",
    upcoming: "Próximos", openNow: "abierto", eventSoon: "{name} en {n} min", eventNow: "¡{name}, ya!", noEvents: "Elige eventos en el panel",
  },
  en: {
    idle: "Out of combat", fight: "In combat", result: "Last fight",
    waiting: "Waiting for the game…", noData: "No data yet",
    total: "Total", dps: "DPS", damage: "Damage", share: "Share", crit: "Crit", back: "Back",
    hits: "Hits", maxHit: "Best hit", duration: "Duration", players: "Players",
    kills: "Kills", deaths: "Deaths", ttk: "Dies in", boss: "BOSS",
    uptime: "Uptime", stacks: "Stacks", summon: "summon", dot: "DoT",
    myDps: "Your DPS", groupDps: "Group DPS", bestHit: "Best hit", critRate: "Crit rate",
    dropped: "{name} dropped!", died: "{name} died", ping: "Ping",
    editHint: "Drag to move · corner to resize",
    trackerEmpty: "Add buffs or debuffs from the panel",
    onSelf: "on you", onTarget: "on target",
    heal: "Healing", hps: "HPS", myHps: "Your HPS",
    upcoming: "Upcoming", openNow: "open", eventSoon: "{name} in {n} min", eventNow: "{name}, now!", noEvents: "Pick events in the panel",
  },
};

let lang = "es";
export const setLang = (l) => { lang = STRINGS[l] ? l : "en"; };
export const getLang = () => lang;
export const t = (key, vars = {}) =>
  (STRINGS[lang][key] ?? STRINGS.en[key] ?? key).replace(/\{(\w+)\}/g, (_, k) => vars[k] ?? "");

export const JOBS = {
  Gladiator: { es: "Gladiador", en: "Gladiator", color: "#e0705a" },
  Templar: { es: "Templario", en: "Templar", color: "#d9b24c" },
  Ranger: { es: "Arquero", en: "Ranger", color: "#7fc96b" },
  Assassin: { es: "Asesino", en: "Assassin", color: "#b07be0" },
  Sorcerer: { es: "Hechicero", en: "Sorcerer", color: "#5aa8f0" },
  Cleric: { es: "Clérigo", en: "Cleric", color: "#f0e6a8" },
  Elementalist: { es: "Maestro espiritual", en: "Spiritmaster", color: "#6fe0d0" },
  Chanter: { es: "Cantor", en: "Chanter", color: "#f0a050" },
  Fighter: { es: "Luchador", en: "Fighter", color: "#ef6f9a" },
};
export const jobColor = (job) => JOBS[job]?.color ?? "#8a93b8";
export const jobName = (job) => JOBS[job]?.[lang] ?? job ?? "";

export function fmt(n) {
  if (n == null || !isFinite(n)) return "–";
  const a = Math.abs(n);
  if (a >= 1e9) return (n / 1e9).toFixed(2) + "B";
  if (a >= 1e6) return (n / 1e6).toFixed(a >= 1e8 ? 0 : 2) + "M";
  if (a >= 1e4) return (n / 1e3).toFixed(a >= 1e5 ? 0 : 1) + "k";
  return Math.round(n).toLocaleString(lang);
}
export const pct = (x, digits = 0) => (x == null || !isFinite(x) ? "–" : (x * 100).toFixed(digits) + "%");
export function clock(seconds) {
  if (seconds == null || !isFinite(seconds)) return "–";
  const s = Math.max(0, Math.floor(seconds));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, "0")}`;
}
export const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

// Icons from the game CDN; a neutral glyph when there is none or it fails.
const FALLBACK = "data:image/svg+xml," + encodeURIComponent(
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="5" fill="#2a2f4a"/><path d="M12 5l1.6 5.4L19 12l-5.4 1.6L12 19l-1.6-5.4L5 12l5.4-1.6z" fill="#8a93b8"/></svg>');
// How many fixed-height items fit whole in a list element.
export const capacity = (el, item, gap) => Math.max(1, Math.floor((el.clientHeight + gap) / (item + gap)));

export const icon = (url, cls = "ic") =>
  `<img class="${cls}" src="${esc(url || FALLBACK)}" onerror="this.onerror=null;this.src='${FALLBACK}'" alt="">`;
