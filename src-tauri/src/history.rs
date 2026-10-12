//! Every finished fight, one JSON file each, newest kept up to a limit.

use crate::engine::fight::FightRecord;
use serde::Serialize;
use std::path::PathBuf;

pub struct History {
    dir: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub start_ms: i64,
    pub duration_s: f64,
    pub target: Option<String>,
    pub is_boss: bool,
    pub my_dps: f64,
    pub total_dps: f64,
    pub players: usize,
    pub kills: usize,
}

impl History {
    pub fn new(dir: PathBuf) -> Self {
        let dir = dir.join("fights");
        let _ = std::fs::create_dir_all(&dir);
        Self { dir }
    }

    pub fn save(&self, record: &FightRecord, limit: usize) {
        // Fights under a few seconds or with no damage of yours are noise.
        if record.view.duration_s < 3.0 || record.view.me.as_ref().is_none_or(|m| m.damage <= 0) {
            return;
        }
        if let Ok(text) = serde_json::to_string(record) {
            let _ = std::fs::write(self.dir.join(format!("{}.json", record.view.id)), text);
        }
        let mut files = self.files();
        while files.len() > limit.max(10) {
            if let Some((_, path)) = files.first() {
                let _ = std::fs::remove_file(path);
            }
            files.remove(0);
        }
    }

    fn files(&self) -> Vec<(i64, PathBuf)> {
        let mut files: Vec<(i64, PathBuf)> = std::fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let id = path.file_stem()?.to_str()?.parse::<i64>().ok()?;
                Some((id, path))
            })
            .collect();
        files.sort_by_key(|(id, _)| *id);
        files
    }

    pub fn list(&self) -> Vec<HistoryEntry> {
        self.files()
            .into_iter()
            .rev()
            .filter_map(|(_, path)| {
                let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
                let f = |k: &str| v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0);
                Some(HistoryEntry {
                    id: v.get("id")?.as_i64()?,
                    start_ms: v.get("startMs")?.as_i64()?,
                    duration_s: f("durationS"),
                    target: v.pointer("/target/name").and_then(|x| x.as_str()).map(String::from),
                    is_boss: v.pointer("/target/isBoss").and_then(|x| x.as_bool()).unwrap_or(false),
                    my_dps: v.pointer("/me/dps").and_then(|x| x.as_f64()).unwrap_or(0.0),
                    total_dps: f("totalDps"),
                    players: v.get("players").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0),
                    kills: v.get("kills").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
                })
            })
            .collect()
    }

    pub fn get(&self, id: i64) -> Option<serde_json::Value> {
        let text = std::fs::read_to_string(self.dir.join(format!("{id}.json"))).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn delete(&self, id: i64) {
        let _ = std::fs::remove_file(self.dir.join(format!("{id}.json")));
    }
}
