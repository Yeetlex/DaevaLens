//! Game data shipped by upstream under `src/data`: skill, NPC and abnormal
//! names per language, abnormal stack limits and icon names.

use a2t::i18n::lookup::{NpcLookup, SkillLookup};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Base of the game's own icon CDN (the same one the game's website uses).
pub const ICON_BASE: &str = "https://assets.playnccdn.com/static-aion2-gamedata/resources";

pub struct GameData {
    pub dir: Option<PathBuf>,
    pub skills: Arc<SkillLookup>,
    pub npcs: Arc<NpcLookup>,
    pub dot_skill_ids: HashSet<i32>,
    pub stack_limits: HashMap<u32, u32>,
    abnormal_names: HashMap<u32, String>,
    abnormal_icons: HashMap<u32, String>,
    skill_icons: HashMap<String, String>,
}

impl GameData {
    /// Look for the data directory next to the binary (bundle), in the
    /// resource dir, or in the source tree (development).
    pub fn find_dir(resource_dir: Option<&Path>) -> Option<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(dir) = resource_dir {
            candidates.push(dir.join("data"));
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            candidates.push(dir.join("data"));
        }
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        candidates.push(manifest.join("../upstream/src/data"));
        candidates
            .into_iter()
            .find(|d| d.join("i18n").join("skills").exists())
    }

    pub fn load(dir: Option<PathBuf>, language: &str) -> Self {
        let skills = SkillLookup::new();
        let npcs = NpcLookup::new();
        let mut data = GameData {
            dir: dir.clone(),
            skills: Arc::new(SkillLookup::new()),
            npcs: Arc::new(NpcLookup::new()),
            dot_skill_ids: HashSet::new(),
            stack_limits: HashMap::new(),
            abnormal_names: HashMap::new(),
            abnormal_icons: HashMap::new(),
            skill_icons: HashMap::new(),
        };
        let Some(dir) = dir else {
            tracing::warn!("game data directory not found: names will be ids");
            return data;
        };
        a2t::i18n::lookup::load_language(&skills, &npcs, &dir, language);
        data.skills = Arc::new(skills);
        data.npcs = Arc::new(npcs);

        if let Ok(text) = std::fs::read_to_string(dir.join("dot_skill_ids.json"))
            && let Ok(ids) = serde_json::from_str::<Vec<i32>>(&text)
        {
            data.dot_skill_ids = ids.into_iter().collect();
        }
        if let Ok(text) = std::fs::read_to_string(dir.join("abnormals.json")) {
            data.stack_limits = a2t::capture::abnormal::stack_limits(&text);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
                && let Some(map) = json.get("abnormals").and_then(|v| v.as_object())
            {
                for (id, entry) in map {
                    if let (Ok(id), Some(icon)) = (id.parse::<u32>(), entry.get("icon").and_then(|v| v.as_str())) {
                        data.abnormal_icons.insert(id, icon.to_string());
                    }
                }
            }
        }
        let names = Self::read_names(&dir.join("i18n").join("abnormals"), language);
        data.abnormal_names = names
            .into_iter()
            .filter_map(|(k, v)| k.parse::<u32>().ok().map(|k| (k, v)))
            .collect();
        if let Ok(text) = std::fs::read_to_string(dir.join("skill_icons.json")) {
            data.skill_icons = serde_json::from_str(&text).unwrap_or_default();
        }
        data
    }

    fn read_names(dir: &Path, language: &str) -> HashMap<String, String> {
        for lang in [language, "en"] {
            if let Ok(text) = std::fs::read_to_string(dir.join(format!("{lang}.json")))
                && let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&text)
            {
                return map;
            }
        }
        HashMap::new()
    }

    pub fn skill_name(&self, code: i32) -> String {
        let name = self.skills.get_skill_name(code);
        if name.is_empty() { format!("#{code}") } else { name }
    }

    pub fn npc_name(&self, code: i32) -> Option<String> {
        let name = self.npcs.get_npc_name(code);
        (!name.is_empty()).then_some(name)
    }

    pub fn abnormal_name(&self, id: u32) -> String {
        self.abnormal_names.get(&id).cloned().unwrap_or_else(|| format!("#{id}"))
    }

    pub fn abnormal_icon(&self, id: u32) -> Option<String> {
        self.abnormal_icons.get(&id).map(|icon| format!("{ICON_BASE}/{icon}.png"))
    }

    /// Icon of a skill, as upstream resolves it: the first four digits of the
    /// eight-digit code index the game's icon table. Basic attacks have none.
    pub fn skill_icon(&self, code: i32) -> Option<String> {
        let digits = code.unsigned_abs().to_string();
        let code8 = if digits.len() >= 8 { digits[..8].to_string() } else { format!("{digits:0<8}") };
        let (prefix2, mid4) = (&code8[..2], &code8[2..6]);
        if (("11"..="18").contains(&prefix2) && mid4 == "0000" && &code8[6..] != "00")
            || code8.starts_with("1000")
            || code8.starts_with("1699")
        {
            return None;
        }
        self.skill_icons.get(&code8[..4]).map(|icon| format!("{ICON_BASE}/{icon}.png"))
    }
}
