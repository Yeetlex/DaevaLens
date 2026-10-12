//! User settings, kept as one JSON file in the app config directory.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Every overlay widget the app can show.
pub const WIDGETS: &[&str] = &["dps", "target", "tracker", "skills", "summary", "alerts", "info"];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OverlayConfig {
    pub enabled: bool,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: u32,
    pub height: u32,
    /// Background opacity, 0-100.
    pub opacity: u8,
    /// UI scale in percent.
    pub scale: u16,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self { enabled: false, x: None, y: None, width: 320, height: 200, opacity: 70, scale: 100 }
    }
}

/// A buff or debuff to follow in the tracker, matched by its name.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Watch {
    pub name: String,
    /// "self" (on you) or "target" (on your target, applied by you).
    pub on: String,
    /// Raise an alert when it drops during a fight.
    #[serde(default)]
    pub alert: bool,
    /// Show stacks as pips (e.g. Element Unification).
    #[serde(default)]
    pub stacks: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub language: String,
    /// Seconds without damage from you or your group that end a fight.
    pub idle_reset_s: u32,
    /// Seconds the last fight stays on the live overlays before they clear.
    pub keep_result_s: u32,
    /// Overlays let clicks through to the game.
    pub locked: bool,
    /// Hide overlays while the game is not the focused window.
    pub hide_unfocused: bool,
    /// Show overlays only while the game is running.
    pub only_with_game: bool,
    /// Start DaevaLens (in the background) when you log in.
    pub launch_at_login: bool,
    pub alerts: bool,
    pub alert_sound: bool,
    /// Only count damage dealt to the targets you hit yourself.
    pub only_my_targets: bool,
    pub overlays: BTreeMap<String, OverlayConfig>,
    pub watches: Vec<Watch>,
    /// The class whose list `watches` is (the last class you played).
    #[serde(alias = "watchPresetFor")]
    pub active_class: Option<String>,
    /// Every other class's list, kept while you play another.
    pub watches_by_class: BTreeMap<String, Vec<Watch>>,
    pub history_limit: usize,
}

impl Default for Settings {
    fn default() -> Self {
        let mut overlays = BTreeMap::new();
        for (name, enabled, w, h) in [
            ("dps", true, 320, 210),
            ("target", true, 320, 74),
            ("tracker", true, 260, 170),
            ("skills", false, 340, 260),
            ("summary", true, 340, 160),
            ("alerts", true, 440, 230),
            ("info", false, 240, 34),
        ] {
            overlays.insert(name.to_string(), OverlayConfig { enabled, width: w, height: h, ..Default::default() });
        }
        Self {
            language: "es".into(),
            idle_reset_s: 8,
            keep_result_s: 15,
            locked: true,
            hide_unfocused: false,
            only_with_game: true,
            launch_at_login: false,
            alerts: true,
            alert_sound: true,
            only_my_targets: true,
            overlays,
            watches: Vec::new(),
            active_class: None,
            watches_by_class: BTreeMap::new(),
            history_limit: 300,
        }
    }
}

/// Starter watch list per class, by abnormal name in the given language.
pub fn class_preset(job: &str, language: &str) -> Vec<Watch> {
    let es = language == "es";
    let w = |name_es: &str, name_en: &str, on: &str, alert: bool, stacks: bool| Watch {
        name: if es { name_es } else { name_en }.to_string(),
        on: on.to_string(),
        alert,
        stacks,
    };
    match job {
        "Elementalist" => vec![
            w("Unificación elemental", "Element Unification", "self", false, true),
            w("Corrosión", "Corrode", "target", true, false),
            w("Maldición", "Curse", "target", true, false),
            w("Favor de Espíritu", "Spirit's Benediction", "self", false, false),
            w("Bendición ardiente", "Flame Blessing", "self", false, false),
        ],
        _ => Vec::new(),
    }
}

pub struct SettingsStore {
    path: PathBuf,
    inner: RwLock<Settings>,
}

impl SettingsStore {
    pub fn load(dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.json");
        let mut settings: Settings = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        // New widgets added in later versions get their defaults.
        let defaults = Settings::default();
        for name in WIDGETS {
            if !settings.overlays.contains_key(*name) {
                settings.overlays.insert(name.to_string(), defaults.overlays[*name].clone());
            }
        }
        // The first build sized the tracker for four entries, not five.
        if let Some(t) = settings.overlays.get_mut("tracker")
            && (t.width, t.height) == (260, 150)
        {
            t.height = 170;
        }
        // 0.1.0 made the alerts too short for several at once.
        if let Some(a) = settings.overlays.get_mut("alerts")
            && a.height < 200
        {
            a.height = 230;
            a.width = a.width.max(440);
        }
        Self { path, inner: RwLock::new(settings) }
    }

    pub fn get(&self) -> Settings {
        self.inner.read().clone()
    }

    pub fn update(&self, f: impl FnOnce(&mut Settings)) -> Settings {
        let mut guard = self.inner.write();
        f(&mut guard);
        let snapshot = guard.clone();
        drop(guard);
        self.save(&snapshot);
        snapshot
    }

    fn save(&self, settings: &Settings) {
        let Ok(text) = serde_json::to_string_pretty(settings) else { return };
        let tmp = self.path.with_extension("json.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, &self.path);
        }
    }
}
