//! Hyprland: overlays open on whatever workspace is active, which is rarely
//! the game's. When the game shows up, move them to its workspace through
//! `hyprctl`. Works with both the classic and the Lua configuration syntax.

use std::process::Command;

pub fn running() -> bool {
    cfg!(target_os = "linux") && std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

fn hyprctl(args: &[&str]) -> Option<String> {
    let out = Command::new("hyprctl").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The workspace of the game window: Steam's `steam_app_3393110`, or any
/// window titled AION2 (Lutris, Heroic…).
fn game_workspace() -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(&hyprctl(&["-j", "clients"])?).ok()?;
    json.as_array()?.iter().find_map(|c| {
        let class = c.get("class")?.as_str()?.to_lowercase();
        let title = c.get("title")?.as_str()?.trim().to_lowercase();
        (class == "steam_app_3393110" || class.contains("aion2") || title == "aion2")
            .then(|| c.pointer("/workspace/name")?.as_str().map(String::from))?
    })
}

fn our_windows() -> Vec<(String, String)> {
    let Some(text) = hyprctl(&["-j", "clients"]) else { return Vec::new() };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { return Vec::new() };
    json.as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| {
            let title = c.get("title")?.as_str()?;
            let addr = c.get("address")?.as_str()?.to_string();
            let ws = c.pointer("/workspace/name")?.as_str()?.to_string();
            title.starts_with("DaevaLens ").then_some((addr, ws))
        })
        .collect()
}

/// Move every overlay to the game's workspace. Returns false when the game
/// window is not there (yet).
pub fn move_overlays_to_game() -> bool {
    let Some(ws) = game_workspace() else { return false };
    for (addr, current) in our_windows() {
        if current == ws {
            continue;
        }
        let classic = format!("{ws},address:{addr}");
        let ok = hyprctl(&["dispatch", "movetoworkspacesilent", &classic]).is_some_and(|o| o.trim() == "ok");
        if !ok {
            let lua = format!(
                "hl.dsp.window.move({{ window = \"address:{addr}\", workspace = \"{ws}\", follow = false }})"
            );
            let _ = hyprctl(&["dispatch", &lua]);
        }
    }
    true
}

/// Whether the active workspace is the game's: `None` when the game window is
/// not found. A fullscreen window covers the floating windows that were
/// already there, so overlays are shown (mapped again, above it) only while
/// you look at the game's workspace, and hidden on the others.
pub fn on_game_workspace() -> Option<bool> {
    let game = game_workspace()?;
    let active: serde_json::Value = serde_json::from_str(&hyprctl(&["-j", "activeworkspace"])?).ok()?;
    Some(active.get("name")?.as_str()? == game)
}
