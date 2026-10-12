//! DaevaLens: overlays and fight analysis on top of the A2Tools DPS Meter
//! parser (https://github.com/taengu/A2Tools-DPS-Meter, GPL-3.0).

mod engine;
mod events;
mod history;
mod hyprland;
mod overlays;
mod settings;

use engine::Engine;
use history::History;
use serde::Serialize;
use settings::{Settings, SettingsStore};
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};

/// What `daevalens --<command>` can ask a running instance to do. Bind these
/// to keys in your desktop (Hyprland, KDE, GNOME…) or use the tray menu.
const USAGE: &str = "\
Usage: daevalens [command]
  --panel            open the control panel
  --edit             toggle edit mode (move/resize overlays)
  --toggle <widget>  show/hide an overlay: dps target tracker skills summary alerts info events
  --reset            clear the current fight
  --background       start without opening the panel (used at login)
  --quit             close DaevaLens";

fn handle_args(app: &AppHandle, args: &[String]) {
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--panel" => overlays::open_panel(app),
            "--edit" => overlays::toggle_edit(app),
            "--toggle" => {
                if let Some(w) = it.next() {
                    overlays::toggle_widget(app, w);
                }
            }
            "--reset" => {
                app.state::<Arc<Engine>>().reset();
            }
            "--quit" => app.exit(0),
            _ => {}
        }
    }
}

#[tauri::command]
fn get_settings(store: State<SettingsStore>) -> Settings {
    store.get()
}

#[tauri::command]
fn save_settings(app: AppHandle, store: State<SettingsStore>, settings: Settings) -> Settings {
    let language_changed = store.get().language != settings.language;
    let autostart_changed = store.get().launch_at_login != settings.launch_at_login;
    let saved = store.update(|s| *s = settings);
    if autostart_changed {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        let result = if saved.launch_at_login { manager.enable() } else { manager.disable() };
        if let Err(e) = result {
            tracing::warn!("autostart: {e}");
        }
    }
    overlays::sync(&app);
    let _ = app.emit("settings-changed", &saved);
    if language_changed {
        // Names are loaded once, in the language at start.
        let _ = app.emit("restart-needed", ());
    }
    saved
}

#[tauri::command]
fn get_live(engine: State<Arc<Engine>>) -> engine::fight::LiveState {
    engine.last.lock().clone()
}

#[tauri::command]
fn list_history(history: State<History>) -> Vec<history::HistoryEntry> {
    history.list()
}

#[tauri::command]
fn get_fight(history: State<History>, id: i64) -> Option<serde_json::Value> {
    history.get(id)
}

#[tauri::command]
fn delete_fight(history: State<History>, id: i64) {
    history.delete(id);
}

#[tauri::command]
fn open_url(app: AppHandle, url: String) {
    use tauri_plugin_opener::OpenerExt;
    if url.starts_with("https://") {
        let _ = app.opener().open_url(url, None::<&str>);
    }
}

/// Replace the tracker list with the preset of the class you are playing.
#[tauri::command]
fn load_class_preset(app: AppHandle, store: State<SettingsStore>, engine: State<Arc<Engine>>) -> Settings {
    let job = engine.last.lock().me_job.clone();
    let saved = store.update(|s| {
        if let Some(job) = job {
            s.watches = settings::class_preset(&job, &s.language);
        }
    });
    let _ = app.emit("settings-changed", &saved);
    saved
}

#[tauri::command]
fn get_upcoming(store: State<SettingsStore>) -> Vec<events::Upcoming> {
    let s = store.get();
    events::upcoming(&s.events, &s.region, a2t::clock::now_ms())
}

/// The built-in definition of an event, to undo edits.
#[tauri::command]
fn builtin_event(store: State<SettingsStore>, id: String) -> Option<events::EventSetting> {
    events::builtin_events(&store.get().language).into_iter().find(|e| e.id == id)
}

#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn reset_fight(engine: State<Arc<Engine>>) {
    engine.reset();
}

#[tauri::command]
fn toggle_edit(app: AppHandle) {
    overlays::toggle_edit(&app);
}

#[tauri::command]
fn toggle_widget(app: AppHandle, widget: String) {
    overlays::toggle_widget(&app, &widget);
}

#[tauri::command]
fn open_panel(app: AppHandle) {
    overlays::open_panel(&app);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeenAbnormal {
    name: String,
    on: String,
    icon: Option<String>,
}

/// Buffs on you and debuffs you put on your targets in the last 10 minutes,
/// to pick from when adding to the tracker.
#[tauri::command]
fn seen_abnormals(engine: State<Arc<Engine>>) -> Vec<SeenAbnormal> {
    let Some(me) = engine.storage.local_player_id().map(|id| id as i32) else { return Vec::new() };
    let now = a2t::clock::now_ms();
    let mut entities: HashSet<i32> = engine
        .storage
        .get_combat_snapshot_light()
        .into_iter()
        .filter(|(_, t)| now - t.last_damage_time < 600_000)
        .map(|(id, _)| id)
        .collect();
    entities.insert(me);
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for t in engine.storage.fight_abnormal_tracks(now - 600_000, now, &entities) {
        let on = if t.entity == me {
            "self"
        } else if t.owner == me {
            "target"
        } else {
            continue;
        };
        let name = engine.data.abnormal_name(t.abnormal);
        if name.starts_with('#') || !seen.insert((on, name.clone())) {
            continue;
        }
        out.push(SeenAbnormal { name, on: on.into(), icon: engine.data.abnormal_icon(t.abnormal) });
    }
    out
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    version: &'static str,
    capture_ok: bool,
    data_found: bool,
    game_seen: bool,
    device: Option<String>,
    platform: &'static str,
}

#[tauri::command]
fn status(engine: State<Arc<Engine>>) -> Status {
    Status {
        version: env!("CARGO_PKG_VERSION"),
        capture_ok: engine.capture_ok,
        data_found: engine.data.dir.is_some(),
        game_seen: engine.port.current_port().is_some(),
        device: engine.port.current_device(),
        platform: std::env::consts::OS,
    }
}

/// Linux trays need libayatana-appindicator (or libappindicator), and Tauri
/// aborts without it. Many desktops do not ship it: run without a tray then.
fn tray_supported() -> bool {
    #[cfg(target_os = "linux")]
    {
        [
            "libayatana-appindicator3.so.1",
            "libappindicator3.so.1",
            "libayatana-appindicator3.so",
            "libappindicator3.so",
        ]
        .iter()
        // SAFETY: only checks that the library loads; nothing is called.
        .any(|name| unsafe { libloading::Library::new(name) }.is_ok())
    }
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    if !tray_supported() {
        tracing::warn!("no tray: libayatana-appindicator is not installed (use the panel or `daevalens --panel`)");
        return Ok(());
    }
    let panel = MenuItem::with_id(app, "panel", "Panel", true, None::<&str>)?;
    let edit = MenuItem::with_id(app, "edit", "Edit overlays / Editar overlays", true, None::<&str>)?;
    let reset = MenuItem::with_id(app, "reset", "Reset fight / Reiniciar combate", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit / Salir", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&panel, &edit, &reset, &sep, &quit])?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("DaevaLens")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "panel" => overlays::open_panel(app),
            "edit" => overlays::toggle_edit(app),
            "reset" => app.state::<Arc<Engine>>().reset(),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// On Wayland, GTK windows cannot keep themselves above others or let clicks
/// through; through XWayland they can. Players can override with GDK_BACKEND.
#[cfg(target_os = "linux")]
fn prefer_xwayland() {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let x11 = std::env::var_os("DISPLAY").is_some();
    if wayland && x11 && std::env::var_os("GDK_BACKEND").is_none() {
        // SAFETY: called first thing in main, before any other thread exists.
        unsafe { std::env::set_var("GDK_BACKEND", "x11") };
    }
    // WebKitGTK's GPU buffers fail on some XWayland setups (see upstream
    // platform/linux/process.rs); shared memory works everywhere.
    if std::env::var_os("WEBKIT_DMABUF_RENDERER_FORCE_SHM").is_none()
        && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
    {
        unsafe { std::env::set_var("WEBKIT_DMABUF_RENDERER_FORCE_SHM", "1") };
    }
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return;
    }
    #[cfg(target_os = "linux")]
    prefer_xwayland();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,a2tools_dps_meter_lib=warn".into()),
        )
        .init();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.len() <= 1 {
                overlays::open_panel(app);
            } else {
                handle_args(app, &argv);
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ));
    #[cfg(any(windows, target_os = "macos"))]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_live,
            list_history,
            get_fight,
            delete_fight,
            reset_fight,
            toggle_edit,
            toggle_widget,
            open_panel,
            seen_abnormals,
            status,
            open_url,
            quit,
            load_class_preset,
            get_upcoming,
            builtin_event
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let config_dir = app.path().app_config_dir().unwrap_or_else(|_| ".".into());
            let data_dir = app.path().app_data_dir().unwrap_or_else(|_| ".".into());
            let store = SettingsStore::load(config_dir);
            let first_run = !store.get().overlays.values().any(|o| o.x.is_some());
            app.manage(History::new(data_dir));
            let engine = Engine::start(&handle, &store);
            app.manage(store);
            app.manage(engine);
            if let Err(e) = build_tray(&handle) {
                tracing::warn!("no tray icon: {e}");
            }
            // Do not flash the overlays at login only to hide them half a second later.
            let s = app.state::<SettingsStore>().get();
            let game = a2t::platform::window_detector::find_aion2_window();
            let here = !hyprland::running() || hyprland::on_game_workspace().unwrap_or(true);
            overlays::set_initially_shown((!s.only_with_game || game) && here);
            overlays::sync(&handle);
            let background = args.iter().any(|a| a == "--background");
            if (first_run && !background) || args.iter().any(|a| a == "--panel") {
                overlays::open_panel(&handle);
            }
            handle_args(&handle, &args.iter().filter(|a| *a != "--panel" && *a != "--background").cloned().collect::<Vec<_>>());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building DaevaLens")
        .run(|_app, event| {
            // Closing the panel keeps the overlays running.
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event
                && code.is_none()
            {
                api.prevent_exit();
            }
        });
}
