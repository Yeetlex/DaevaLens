//! Overlay windows: one borderless, transparent, always-on-top window per
//! widget, labelled `ov-<widget>`. Locked, they let clicks through to the
//! game; in edit mode they can be dragged and resized, and remember where.

use crate::settings::{SettingsStore, WIDGETS};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder, WindowEvent};

/// Whether overlays should be on screen now (game running, in front…).
static SHOWN: AtomicBool = AtomicBool::new(true);

/// Before the overlays exist: whether they start on screen.
pub fn set_initially_shown(shown: bool) {
    SHOWN.store(shown, Ordering::Relaxed);
}

pub fn label(widget: &str) -> String {
    format!("ov-{widget}")
}

/// Open the enabled overlays and close the others.
pub fn sync(app: &AppHandle) {
    let settings = app.state::<SettingsStore>().get();
    for widget in WIDGETS {
        let cfg = &settings.overlays[*widget];
        let open = app.get_webview_window(&label(widget));
        match (cfg.enabled, open) {
            (true, None) => open_widget(app, widget),
            (false, Some(w)) => {
                let _ = w.close();
            }
            _ => {}
        }
    }
    apply_lock(app);
}

fn open_widget(app: &AppHandle, widget: &str) {
    let settings = app.state::<SettingsStore>().get();
    let cfg = settings.overlays[widget].clone();
    let url = WebviewUrl::App(format!("overlay.html?w={widget}").into());
    let builder = WebviewWindowBuilder::new(app, label(widget), url)
        .title(format!("DaevaLens {widget}"))
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .focused(false)
        .visible(SHOWN.load(Ordering::Relaxed))
        .inner_size(cfg.width as f64, cfg.height as f64);
    let window = match builder.build() {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("overlay {widget} failed: {e}");
            return;
        }
    };
    if let (Some(x), Some(y)) = (cfg.x, cfg.y) {
        let _ = window.set_position(PhysicalPosition::new(x, y));
    } else {
        let _ = window.set_position(default_position(&window, &settings, widget));
    }
    // tao aborts on click-through for a window that was never shown (GTK has
    // no GdkWindow for it yet): only for visible ones, the rest when shown.
    if SHOWN.load(Ordering::Relaxed) {
        let _ = window.set_ignore_cursor_events(settings.locked);
    }

    let app2 = app.clone();
    let name = widget.to_string();
    window.on_window_event(move |event| match event {
        WindowEvent::Moved(pos) => remember(&app2, &name, Some(*pos), None),
        WindowEvent::Resized(size) => remember(&app2, &name, None, Some(*size)),
        _ => {}
    });
}

/// First-run layout: a column on the left, alerts and status centred on top.
fn default_position(window: &tauri::WebviewWindow, settings: &crate::settings::Settings, widget: &str) -> PhysicalPosition<i32> {
    let (mw, scale) = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| (m.size().width as i32, m.scale_factor()))
        .unwrap_or((1920, 1.0));
    let px = |v: u32| (v as f64 * scale).round() as i32;
    let center = |w: &str| (mw - px(settings.overlays[w].width)) / 2;
    match widget {
        "info" => PhysicalPosition::new(center("info"), px(60)),
        "alerts" => PhysicalPosition::new(center("alerts"), px(150)),
        _ => {
            let column = ["dps", "target", "tracker", "summary", "skills"];
            let mut y = px(120);
            for w in column {
                if w == widget {
                    break;
                }
                if settings.overlays[w].enabled {
                    y += px(settings.overlays[w].height + 10);
                }
            }
            PhysicalPosition::new(px(30), y)
        }
    }
}

fn remember(app: &AppHandle, widget: &str, pos: Option<PhysicalPosition<i32>>, size: Option<PhysicalSize<u32>>) {
    // Only what the player did in edit mode is kept: compositors also move
    // windows around on their own (e.g. when a game goes fullscreen).
    let store = app.state::<SettingsStore>();
    if store.get().locked {
        return;
    }
    let scale = app
        .get_webview_window(&label(widget))
        .and_then(|w| w.scale_factor().ok())
        .unwrap_or(1.0);
    store.update(|s| {
        if let Some(cfg) = s.overlays.get_mut(widget) {
            if let Some(p) = pos {
                cfg.x = Some(p.x);
                cfg.y = Some(p.y);
            }
            if let Some(sz) = size {
                cfg.width = (sz.width as f64 / scale).round() as u32;
                cfg.height = (sz.height as f64 / scale).round() as u32;
            }
        }
    });
}

/// Locked: clicks go through. Unlocked (edit mode): drag and resize.
pub fn apply_lock(app: &AppHandle) {
    let settings = app.state::<SettingsStore>().get();
    for widget in WIDGETS {
        if let Some(w) = app.get_webview_window(&label(widget))
            && w.is_visible().unwrap_or(false)
        {
            let _ = w.set_ignore_cursor_events(settings.locked);
        }
    }
    let _ = app.emit("settings-changed", &settings);
}

pub fn set_all_visible(app: &AppHandle, visible: bool) {
    SHOWN.store(visible, Ordering::Relaxed);
    let locked = app.state::<SettingsStore>().get().locked;
    for widget in WIDGETS {
        if let Some(w) = app.get_webview_window(&label(widget)) {
            if visible {
                let _ = w.show();
                let _ = w.set_ignore_cursor_events(locked);
            } else {
                let _ = w.hide();
            }
        }
    }
}

pub fn toggle_widget(app: &AppHandle, widget: &str) {
    if !WIDGETS.contains(&widget) {
        return;
    }
    app.state::<SettingsStore>().update(|s| {
        if let Some(cfg) = s.overlays.get_mut(widget) {
            cfg.enabled = !cfg.enabled;
        }
    });
    sync(app);
}

pub fn toggle_edit(app: &AppHandle) {
    app.state::<SettingsStore>().update(|s| s.locked = !s.locked);
    apply_lock(app);
}

pub fn open_panel(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("panel") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let Ok(panel) = WebviewWindowBuilder::new(app, "panel", WebviewUrl::App("panel.html".into()))
        .title("DaevaLens")
        .inner_size(1150.0, 800.0)
        .min_inner_size(900.0, 620.0)
        .center()
        .build()
    else {
        return;
    };
    // Closing the panel in edit mode would leave the overlays catching clicks
    // over the game: lock them again.
    let app2 = app.clone();
    panel.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) && !app2.state::<SettingsStore>().get().locked {
            app2.state::<SettingsStore>().update(|s| s.locked = true);
            apply_lock(&app2);
        }
    });
}
