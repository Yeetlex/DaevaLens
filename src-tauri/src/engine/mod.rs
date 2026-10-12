//! Live capture and the tick loop that turns upstream's storage into fights.

pub mod data;
pub mod fight;

use crate::history::History;
use crate::settings::{class_preset, SettingsStore};
use a2t::capture::captured_payload::CapturedPayload;
use a2t::capture::combat_port_detector::CombatPortDetector;
use a2t::capture::pcap_capturer::PcapCapturer;
use a2t::combat::capture_dispatcher::CaptureDispatcher;
use a2t::combat::data_storage::DataStorage;
use a2t::combat::ping_tracker::PingTracker;
use data::GameData;
use fight::{LiveState, Tracker};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// How often the overlays get a new state.
pub const TICK_MS: u64 = 100;

pub struct Engine {
    pub storage: Arc<DataStorage>,
    pub data: Arc<GameData>,
    pub ping: Arc<PingTracker>,
    pub port: Arc<CombatPortDetector>,
    pub tracker: Mutex<Tracker>,
    pub last: Mutex<LiveState>,
    pub capture_ok: bool,
}

impl Engine {
    pub fn start(app: &AppHandle, settings: &SettingsStore) -> Arc<Engine> {
        let language = settings.get().language;
        let resource_dir = app.path().resource_dir().ok();
        let data = Arc::new(GameData::load(GameData::find_dir(resource_dir.as_deref()), &language));

        let storage = Arc::new(DataStorage::new());
        storage.set_abnormal_stack_limits(data.stack_limits.clone());
        let ping = Arc::new(PingTracker::with_perf_clock(a2t::platform::clock::perf_clock()));
        let port = Arc::new(CombatPortDetector::new());

        let capture_ok = a2t::platform::pcap::library_available();
        let (tx, rx) = tokio::sync::mpsc::channel::<CapturedPayload>(4096);
        let capturer = PcapCapturer::new(tx);
        if capture_ok {
            capturer.start();
        } else {
            tracing::error!("packet capture library missing (Npcap on Windows, libpcap on Linux)");
        }
        let mut dispatcher = CaptureDispatcher::new(
            storage.clone(),
            data.skills.clone(),
            data.npcs.clone(),
            port.clone(),
            ping.clone(),
        );
        dispatcher.set_dot_skill_ids(data.dot_skill_ids.clone());
        tauri::async_runtime::spawn(async move {
            let _keep = capturer;
            dispatcher.run(rx).await;
        });

        let engine = Arc::new(Engine {
            storage,
            data,
            ping,
            port,
            tracker: Mutex::new(Tracker::new()),
            last: Mutex::new(LiveState::default()),
            capture_ok,
        });
        engine.clone().spawn_loop(app.clone());
        engine
    }

    fn spawn_loop(self: Arc<Self>, app: AppHandle) {
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(TICK_MS));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut ticks: u64 = 0;
            let mut game_running = false;
            let mut shown = true;
            let mut hypr_tries = 0u32;
            loop {
                interval.tick().await;
                ticks += 1;
                let store = app.state::<SettingsStore>();
                let settings = store.get();
                let now = a2t::clock::now_ms();
                let (mut live, ended, alerts) = {
                    let mut tracker = self.tracker.lock();
                    let (live, ended) = tracker.tick(&self.storage, &self.data, &settings, now);
                    (live, ended, tracker.last_alerts.clone())
                };
                live.ping_ms = self.ping.current_ping_ms().map(|p| p as u32);
                live.game_seen = self.port.current_port().is_some();

                // First time we learn the class: a starter watch list.
                if let Some(job) = live.me_job.clone()
                    && settings.watch_preset_for.is_none()
                {
                    let preset = class_preset(&job, &settings.language);
                    store.update(|s| {
                        if s.watches.is_empty() {
                            s.watches = preset;
                        }
                        s.watch_preset_for = Some(job);
                    });
                    let _ = app.emit("settings-changed", store.get());
                }

                if let Some(record) = ended {
                    let _ = app.emit("fight-ended", &record.view);
                    let history = app.state::<History>();
                    history.save(&record, settings.history_limit);
                }
                for alert in alerts {
                    let _ = app.emit("alert", &alert);
                }
                let _ = app.emit("live", &live);
                *self.last.lock() = live;

                // Overlays only with the game (and in front of it, if asked).
                if ticks % 20 == 1 {
                    game_running = a2t::platform::window_detector::find_aion2_window();
                }
                if ticks % 5 == 0 {
                    let focused = !settings.hide_unfocused
                        || a2t::platform::window_detector::is_aion2_foreground()
                        || app.webview_windows().values().any(|w| w.is_focused().unwrap_or(false));
                    let mut show = (!settings.only_with_game || game_running) && focused;
                    if show && crate::hyprland::running() && let Some(here) = crate::hyprland::on_game_workspace() {
                        show = here;
                    }
                    if show != shown {
                        shown = show;
                        crate::overlays::set_all_visible(&app, show);
                        // Hyprland: follow the game to its workspace. Its window
                        // can appear a while after the process: keep trying.
                        hypr_tries = if show && crate::hyprland::running() { 30 } else { 0 };
                    }
                }
                if hypr_tries > 0 && ticks % 10 == 0 {
                    hypr_tries = if crate::hyprland::move_overlays_to_game() { 0 } else { hypr_tries - 1 };
                }
            }
        });
    }

    pub fn reset(&self) {
        self.tracker.lock().reset();
    }
}
