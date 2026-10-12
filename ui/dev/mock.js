// Stand-in for the Tauri bridge when the UI is opened in a plain browser
// (ui/dev/preview.html): fake but demanding data, to check the layout.
(() => {
  const ICON = "https://assets.playnccdn.com/static-aion2-gamedata/resources";
  const names = ["N0BARA", "Xx_DarkLordOfTheAbyss_xX", "Zoey", "LisanaeITA", "GrofSprit", "Calligraphy", "Sigun", "Neustrashimy", "Silvera", "AVeryLongNameIndeed", "Bob", "Ana"];
  const jobs = ["Elementalist", "Gladiator", "Cleric", "Sorcerer", "Ranger", "Templar", "Assassin", "Chanter", "Fighter", "Sorcerer", "Cleric", "Ranger"];
  const skills = [
    ["Incineración de llama: Explosión ardiente de fuego eterno", "ICON_EL_SKILL_001"], ["Impacto helado", "ICON_EL_SKILL_002"],
    ["Fusión elemental", "ICON_EL_SKILL_010"], ["Maldición", "ICON_EL_SKILL_014"], ["Corrosión", "ICON_EL_SKILL_015"],
    ["Espíritu de agua: Ataque", null], ["Ráfaga continua", "ICON_EL_SKILL_005"], ["Espíritu ancestral", "ICON_EL_SKILL_020"],
    ["Ataque básico", null], ["Dominio espacial", "ICON_EL_SKILL_011"], ["Nube de maldición", "ICON_EL_SKILL_022"],
  ];
  const settings = {
    language: new URLSearchParams(location.search).get("lang") || "es", idleResetS: 8, keepResultS: 15, locked: !location.search.includes("edit"),
    hideUnfocused: false, onlyWithGame: true, launchAtLogin: false, alerts: true, alertSound: false, onlyMyTargets: true, historyLimit: 300, watchPresetFor: "Elementalist",
    overlays: Object.fromEntries(["dps", "target", "tracker", "skills", "summary", "alerts", "info"].map((w) => [w, { enabled: true, x: null, y: null, width: 300, height: 200, opacity: 70, scale: 100 }])),
    watches: [
      { name: "Unificación elemental", on: "self", alert: false, stacks: true },
      { name: "Corrosión", on: "target", alert: true, stacks: false },
      { name: "Maldición", on: "target", alert: true, stacks: false },
      { name: "Favor de Espíritu del Gran Espíritu Ancestral", on: "self", alert: false, stacks: false },
      { name: "Bendición ardiente", on: "self", alert: false, stacks: false },
    ],
  };
  let t = 0;
  function fight(final = false) {
    const dur = 47 + t * 0.1;
    const players = names.map((n, i) => {
      const dps = Math.round(98765432 / (i + 1.3) + Math.sin(t / 7 + i) * 400000);
      return { id: i + 1, name: n, job: jobs[i], damage: dps * dur, dps, share: 1 / (i + 1.3) / 3.2, isMe: i === 0, critRate: 0.42 };
    });
    const total = players.reduce((a, p) => a + p.damage, 0);
    return {
      id: 1760000000000, startMs: Date.now() - dur * 1000, endMs: final ? Date.now() : null, durationS: dur,
      totalDamage: total, totalDps: total / dur, players,
      me: { damage: players[0].damage, dps: players[0].dps, hits: 1234, critRate: 0.417, backRate: 0.12, perfectRate: 0.05, doubleRate: 0.08, bestHit: 987654321, bestHitSkill: skills[2][0], summonShare: 0.31 },
      skills: skills.map(([name, ic], i) => ({ code: 16000000 + i, name, icon: ic && `${ICON}/${ic}.png`, isDot: i === 4, summon: i === 5, damage: 9e8 / (i + 1), share: 0.3 / (i + 1), dps: 3e7 / (i + 1), hits: 120 - i * 7, critRate: 0.4, backRate: 0.1, maxHit: 2e7 / (i + 1) })),
      target: { id: 99, name: "Guardián Ancestral del Abismo Profundo de Krao (Pesadilla)", isBoss: true, hp: 812345678, maxHp: 1999999999, pct: 0.41 - t * 0.001, ttkS: 1234.5, dead: false },
      targetsHit: 3, kills: 12, deaths: ["Zoey"], zone: null,
    };
  }
  const live = () => ({
    nowMs: Date.now(), phase: t % 60 < 45 ? "fight" : "result", fight: fight(), meName: "N0BARA", meJob: "Elementalist", pingMs: 37, gameSeen: true,
    tracker: settings.watches.map((w, i) => ({ name: w.name, on: w.on, icon: `${ICON}/ICON_EL_SKILL_0${15 + i}.png`, active: (t + i * 3) % 10 > 2, stacks: (t >> 1) % 6, maxStacks: w.stacks ? 5 : 1, showStacks: w.stacks, uptime: 0.5 + i * 0.1, alert: w.alert })),
  });
  const timeline = Array.from({ length: 48 }, (_, s) => [s, 1e8 + Math.sin(s / 4) * 4e7, 6e8 + Math.cos(s / 6) * 2e8]);
  const handlers = {};
  const emit = (ev, payload) => (handlers[ev] || []).forEach((cb) => cb({ payload }));
  const responses = {
    get_settings: () => settings, save_settings: (a) => Object.assign(settings, a.settings), get_live: live,
    list_history: () => Array.from({ length: 14 }, (_, i) => ({ id: 1760000000000 - i * 99999, startMs: Date.now() - i * 3.6e6, durationS: 30 + i * 17, target: i % 3 ? "Araña de cueva" : "Guardián Ancestral del Abismo Profundo de Krao (Pesadilla)", isBoss: !(i % 3), myDps: 3.2e7 / (i + 1), totalDps: 9e8, players: 6, kills: 3 })),
    get_fight: () => ({ ...fight(true), timeline, watches: live().tracker }),
    seen_abnormals: () => [{ name: "Escudo de tierra", on: "self", icon: null }, { name: "Ralentización", on: "target", icon: null }],
    status: () => ({ version: "0.1.0", captureOk: !location.search.includes("nocap"), dataFound: true, gameSeen: true, device: "enp16s0", platform: location.search.includes("win") ? "windows" : "linux" }),
  };
  const win = { startDragging() {}, startResizeDragging() {} };
  window.__TAURI__ = {
    core: { invoke: async (cmd, args) => (responses[cmd] ? responses[cmd](args || {}) : null) },
    event: { listen: async (ev, cb) => { (handlers[ev] ||= []).push(cb); return () => {}; } },
    window: { getCurrentWindow: () => win },
  };
  setInterval(() => { t++; emit("live", live()); }, 250);
  setInterval(() => {
    emit("fight-ended", fight(true));
    ["Corrosión", "Maldición", "Favor de Espíritu del Gran Espíritu Ancestral"].forEach((n, i) => setTimeout(() => emit("alert", { kind: i === 2 ? "death" : "dropped", text: n, icon: `${ICON}/ICON_EL_SKILL_015.png` }), i * 300));
  }, 4000);
})();
