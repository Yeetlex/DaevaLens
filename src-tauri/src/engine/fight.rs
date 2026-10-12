//! Fights on top of upstream's combat storage.
//!
//! Upstream keeps running totals per (target, actor, skill). A fight here is a
//! stretch of activity by you or your group: it starts with the first new hit,
//! ends after `idle_reset` without one, and its numbers are the totals now minus
//! the totals when it started. Upstream resetting its own storage (zone change,
//! long idle) shows up as a total going down, and that key starts over from 0.

use super::data::GameData;
use crate::settings::{Settings, Watch};
use a2t::combat::data_storage::{DataStorage, TargetCombatData};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};

type Key = (i32, i32, i32, bool); // target, raw actor, skill code, is dot

#[derive(Clone, Copy, Default, Debug)]
struct Counter {
    damage: i64,
    hits: i64,
    crit: i64,
    back: i64,
    perfect: i64,
    double: i64,
    parry: i64,
    max_hit: i64,
}

impl Counter {
    fn minus(self, base: Counter) -> Counter {
        Counter {
            damage: self.damage - base.damage,
            hits: self.hits - base.hits,
            crit: self.crit - base.crit,
            back: self.back - base.back,
            perfect: self.perfect - base.perfect,
            double: self.double - base.double,
            parry: self.parry - base.parry,
            // A maximum cannot be subtracted: keep it only if it moved.
            max_hit: if self.max_hit > base.max_hit { self.max_hit } else { 0 },
        }
    }
    fn add(&mut self, o: Counter) {
        self.damage += o.damage;
        self.hits += o.hits;
        self.crit += o.crit;
        self.back += o.back;
        self.perfect += o.perfect;
        self.double += o.double;
        self.parry += o.parry;
        self.max_hit = self.max_hit.max(o.max_hit);
    }
}

#[derive(Clone, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRow {
    pub id: i32,
    pub name: String,
    pub job: Option<String>,
    pub damage: i64,
    pub dps: f64,
    pub share: f64,
    pub is_me: bool,
    pub crit_rate: f64,
    /// Healing done in the fight (by them and their summons) and per second.
    pub heal: i64,
    pub hps: f64,
}

#[derive(Clone, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SkillRow {
    pub code: i32,
    pub name: String,
    pub icon: Option<String>,
    pub is_dot: bool,
    pub damage: i64,
    pub share: f64,
    pub dps: f64,
    pub hits: i64,
    pub crit_rate: f64,
    pub back_rate: f64,
    pub max_hit: i64,
    /// Damage done by your summons with this skill.
    pub summon: bool,
}

#[derive(Clone, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MeStats {
    pub damage: i64,
    pub dps: f64,
    pub hits: i64,
    pub crit_rate: f64,
    pub back_rate: f64,
    pub perfect_rate: f64,
    pub double_rate: f64,
    pub best_hit: i64,
    pub best_hit_skill: Option<String>,
    /// Share of your damage done by your summons.
    pub summon_share: f64,
    pub heal: i64,
    pub hps: f64,
}

#[derive(Clone, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TargetView {
    pub id: i32,
    pub name: String,
    pub is_boss: bool,
    pub hp: Option<i64>,
    pub max_hp: Option<i64>,
    pub pct: Option<f64>,
    /// Seconds left at the group's recent damage rate on it.
    pub ttk_s: Option<f64>,
    pub dead: bool,
}

#[derive(Clone, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WatchView {
    pub name: String,
    pub on: String,
    pub icon: Option<String>,
    pub active: bool,
    pub stacks: u32,
    pub max_stacks: u32,
    pub show_stacks: bool,
    /// Fraction of the fight (or the last minute, out of combat) it was up.
    pub uptime: f64,
    pub alert: bool,
}

#[derive(Clone, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FightView {
    pub id: i64,
    pub start_ms: i64,
    pub end_ms: Option<i64>,
    pub duration_s: f64,
    pub total_damage: i64,
    pub total_dps: f64,
    pub players: Vec<PlayerRow>,
    pub me: Option<MeStats>,
    pub skills: Vec<SkillRow>,
    pub target: Option<TargetView>,
    pub targets_hit: usize,
    pub kills: usize,
    pub deaths: Vec<String>,
    pub zone: Option<String>,
}

/// One second of a fight: your damage and everyone's.
#[derive(Clone, Serialize, serde::Deserialize, Debug, Default)]
pub struct Second(pub u32, pub i64, pub i64);

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FightRecord {
    #[serde(flatten)]
    pub view: FightView,
    pub timeline: Vec<Second>,
    pub watches: Vec<WatchView>,
    /// The enemies you hit, for learning a tracker list (not saved).
    #[serde(skip)]
    pub target_ids: Vec<i32>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub kind: String,
    pub text: String,
    pub icon: Option<String>,
}

#[derive(Clone, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LiveState {
    pub now_ms: i64,
    /// "idle", "fight" or "result" (the last fight, kept on screen a while).
    pub phase: &'static str,
    pub fight: Option<FightView>,
    pub tracker: Vec<WatchView>,
    pub me_name: Option<String>,
    pub me_job: Option<String>,
    pub ping_ms: Option<u32>,
    pub game_seen: bool,
}

struct Fight {
    start_ms: i64,
    last_activity_ms: i64,
    baseline: HashMap<Key, Counter>,
    my_targets: HashSet<i32>,
    focus: Option<i32>,
    focus_hit_ms: i64,
    seconds: Vec<Second>,
    known_dead: HashSet<i32>,
    deaths: Vec<String>,
    kills: HashSet<i32>,
}

pub struct Tracker {
    prev: HashMap<Key, Counter>,
    fight: Option<Fight>,
    result: Option<(FightView, i64)>,
    /// (time, damage on focus target) for the time-to-kill rate.
    focus_samples: VecDeque<(i64, i64)>,
    /// Per watch: up, down since, alerted, and the target it was up on.
    watch_state: HashMap<String, (bool, Option<i64>, bool, i32)>,
    pub last_alerts: Vec<Alert>,
    initialized: bool,
    /// Healing per player in the current fight, refreshed about once a second:
    /// upstream keeps every heal tick of the session, too much to copy each tick.
    heal_cache: HashMap<i32, i64>,
    heal_calls: u32,
}

fn job_of(t: &TargetCombatData, actor: i32) -> Option<String> {
    t.actors.get(&actor).and_then(|a| a.job).map(|j| format!("{j:?}"))
}

fn rate(part: i64, whole: i64) -> f64 {
    if whole > 0 { part as f64 / whole as f64 } else { 0.0 }
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            prev: HashMap::new(),
            fight: None,
            result: None,
            focus_samples: VecDeque::new(),
            watch_state: HashMap::new(),
            last_alerts: Vec::new(),
            initialized: false,
            heal_cache: HashMap::new(),
            heal_calls: 0,
        }
    }

    /// Forget the current fight and what is on screen.
    pub fn reset(&mut self) {
        self.fight = None;
        self.result = None;
        self.focus_samples.clear();
    }

    /// Advance one tick. Returns the live state and, when a fight just ended,
    /// its record for the history.
    pub fn tick(
        &mut self,
        storage: &DataStorage,
        data: &GameData,
        settings: &Settings,
        now: i64,
    ) -> (LiveState, Option<FightRecord>) {
        self.last_alerts.clear();
        let snapshot = storage.get_combat_snapshot_light();
        let summons = storage.get_summon_data();
        let me = storage.local_player_id().map(|id| id as i32);
        let owner_of = |actor: i32| -> i32 { summons.get(&actor).copied().unwrap_or(actor) };
        let party: HashSet<i32> = storage
            .get_party_members()
            .keys()
            .filter_map(|name| storage.find_id_by_nickname(name))
            .collect();

        // Current counters, and what moved since the last tick.
        let mut current: HashMap<Key, Counter> = HashMap::with_capacity(self.prev.len() + 16);
        for (&target, t) in &snapshot {
            for (&actor, a) in &t.actors {
                for (&(skill, dot), s) in &a.skills {
                    current.insert(
                        (target, actor, skill, dot),
                        Counter {
                            damage: s.total_damage as i64,
                            hits: s.hit_count as i64,
                            crit: s.crit_count as i64,
                            back: s.back_count as i64,
                            perfect: s.perfect_count as i64,
                            double: s.double_count as i64,
                            parry: s.parry_count as i64,
                            max_hit: s.max_damage as i64,
                        },
                    );
                }
            }
        }
        let mut moved_mine = false;
        let mut moved_group = false;
        let mut my_hit_targets: Vec<i32> = Vec::new();
        let mut first_hit_ms = now;
        for (key, c) in &current {
            let before = self.prev.get(key).map(|p| p.damage).unwrap_or(0);
            // Unchanged, or the very first look at storage (old damage).
            if c.damage == before || c.damage <= 0 || !self.initialized {
                continue;
            }
            let owner = owner_of(key.1);
            let is_me = Some(owner) == me;
            if is_me {
                moved_mine = true;
                my_hit_targets.push(key.0);
            } else if party.contains(&owner)
                || self.fight.as_ref().is_some_and(|f| f.my_targets.contains(&key.0))
            {
                moved_group = true;
            }
            if let Some(t) = snapshot.get(&key.0)
                && let Some(a) = t.actors.get(&key.1)
                && a.last_damage_time > 0
            {
                first_hit_ms = first_hit_ms.min(a.last_damage_time);
            }
        }
        // Before the local player is known, any damage starts a fight.
        let active = moved_mine || moved_group || (me.is_none() && !my_hit_targets.is_empty());

        // Start / end.
        let mut ended: Option<FightRecord> = None;
        if self.fight.is_none() && (moved_mine || (me.is_none() && active)) {
            self.fight = Some(Fight {
                start_ms: first_hit_ms.min(now),
                last_activity_ms: now,
                baseline: self.prev.clone(),
                my_targets: HashSet::new(),
                focus: None,
                focus_hit_ms: 0,
                seconds: Vec::new(),
                known_dead: storage.get_dead_entities(),
                deaths: Vec::new(),
                kills: HashSet::new(),
            });
            self.result = None;
            self.focus_samples.clear();
        }
        if let Some(f) = self.fight.as_mut() {
            if active {
                f.last_activity_ms = now;
            }
            for t in &my_hit_targets {
                f.my_targets.insert(*t);
                f.focus = Some(*t);
                f.focus_hit_ms = now;
            }
            // Upstream reset a key: start that key over.
            for (key, c) in &current {
                if let Some(b) = f.baseline.get(key)
                    && (c.damage < b.damage || c.hits < b.hits)
                {
                    f.baseline.insert(*key, Counter::default());
                }
            }
        }

        let fight_over = self
            .fight
            .as_ref()
            .is_some_and(|f| now - f.last_activity_ms > settings.idle_reset_s as i64 * 1000);

        let mut view = None;
        if self.fight.is_some() {
            let v = self.build_view(storage, data, settings, &snapshot, &current, &owner_of, me, &party, now, fight_over);
            if fight_over {
                let f = self.fight.take().expect("fight");
                let watches = self.watch_views(storage, data, settings, me, f.focus, f.start_ms, f.last_activity_ms);
                let target_ids = f.my_targets.iter().copied().collect();
                ended = Some(FightRecord { view: v.clone(), timeline: f.seconds, watches, target_ids });
                self.heal_cache.clear();
                self.result = Some((v.clone(), now));
            }
            view = Some(v);
        }

        let mut phase = if self.fight.is_some() { "fight" } else { "idle" };
        if self.fight.is_none() {
            view = None;
            if let Some((v, at)) = &self.result {
                if now - at < settings.keep_result_s as i64 * 1000 {
                    phase = "result";
                    view = Some(v.clone());
                } else {
                    self.result = None;
                }
            }
        }

        let (start, focus) = match &self.fight {
            Some(f) => (f.start_ms, f.focus),
            None => (now - 60_000, None),
        };
        let tracker = self.watch_views(storage, data, settings, me, focus, start, now);
        if self.fight.is_some() && settings.alerts {
            let focus_dead = focus.is_some_and(|id| storage.is_entity_dead(id));
            self.check_alerts(&tracker, focus.unwrap_or(0), focus_dead, now);
        } else {
            // A new fight starts from scratch: nothing it has not put up yet "dropped".
            self.watch_state.clear();
        }

        self.prev = current;
        self.initialized = true;
        let profile = storage.local_profile();
        let me_job = profile
            .class
            .map(|c| format!("{c:?}"))
            .or_else(|| me.and_then(|id| snapshot.values().find_map(|t| job_of(t, id))));
        (
            LiveState {
                now_ms: now,
                phase,
                fight: view,
                tracker,
                me_name: storage.local_character_name().or(profile.name.clone()),
                me_job,
                ping_ms: None,
                game_seen: false,
            },
            ended,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build_view(
        &mut self,
        storage: &DataStorage,
        data: &GameData,
        settings: &Settings,
        snapshot: &HashMap<i32, TargetCombatData>,
        current: &HashMap<Key, Counter>,
        owner_of: &dyn Fn(i32) -> i32,
        me: Option<i32>,
        party: &HashSet<i32>,
        now: i64,
        over: bool,
    ) -> FightView {
        let f = self.fight.as_mut().expect("fight");
        let end = if over { f.last_activity_ms } else { now };
        let duration_s = ((end - f.start_ms) as f64 / 1000.0).max(1.0);

        // Fight-only counters, per player and per (my) skill.
        let mut per_player: HashMap<i32, Counter> = HashMap::new();
        let mut per_player_job: HashMap<i32, String> = HashMap::new();
        let mut my_skills: HashMap<(i32, bool, bool), Counter> = HashMap::new(); // skill, dot, by summon
        let mut damage_on: HashMap<i32, i64> = HashMap::new();
        for (key, c) in current {
            let base = f.baseline.get(key).copied().unwrap_or_default();
            let d = c.minus(base);
            if d.damage <= 0 && d.hits <= 0 {
                continue;
            }
            let (target, actor, skill, dot) = *key;
            let owner = owner_of(actor);
            let mine = Some(owner) == me;
            let in_scope = !settings.only_my_targets
                || me.is_none()
                || f.my_targets.contains(&target)
                || mine
                || party.contains(&owner);
            if !in_scope || storage.is_summon(owner) {
                continue;
            }
            per_player.entry(owner).or_default().add(d);
            if let Some(job) = snapshot.get(&target).and_then(|t| job_of(t, owner)) {
                per_player_job.insert(owner, job);
            }
            *damage_on.entry(target).or_default() += d.damage;
            if mine {
                my_skills.entry((skill, dot, actor != owner)).or_default().add(d);
            }
        }

        let total_damage: i64 = per_player.values().map(|c| c.damage).sum();
        let my_total = me.and_then(|id| per_player.get(&id)).map(|c| c.damage).unwrap_or(0);

        // Timeline: add this tick's growth to the current second.
        let sec = ((now - f.start_ms).max(0) / 1000) as u32;
        let (prev_my, prev_all) = f
            .seconds
            .iter()
            .fold((0i64, 0i64), |acc, s| (acc.0 + s.1, acc.1 + s.2));
        if !over && (my_total > prev_my || total_damage > prev_all) {
            let (dm, da) = ((my_total - prev_my).max(0), (total_damage - prev_all).max(0));
            match f.seconds.last_mut() {
                Some(last) if last.0 == sec => {
                    last.1 += dm;
                    last.2 += da;
                }
                _ => f.seconds.push(Second(sec, dm, da)),
            }
        }

        // Healing inside the fight window, per healer (summons count for their owner).
        if self.heal_calls % 10 == 0 || over {
            self.heal_cache.clear();
            for (actor, skills) in storage.get_heal_snapshot() {
                let owner = owner_of(actor);
                if !(Some(owner) == me || party.contains(&owner) || per_player.contains_key(&owner)) {
                    continue;
                }
                let healed: i64 = skills
                    .values()
                    .flat_map(|s| s.ticks.iter())
                    .filter(|(t, _)| *t >= f.start_ms && *t <= end)
                    .map(|(_, a)| *a)
                    .sum();
                if healed > 0 {
                    *self.heal_cache.entry(owner).or_default() += healed;
                }
            }
        }
        self.heal_calls = self.heal_calls.wrapping_add(1);
        for id in self.heal_cache.keys() {
            per_player.entry(*id).or_default();
        }
        let heals = &self.heal_cache;

        let mut players: Vec<PlayerRow> = per_player
            .iter()
            .filter(|(id, c)| c.damage > 0 || heals.get(id).is_some_and(|h| *h > 0))
            .map(|(&id, c)| {
                let heal = heals.get(&id).copied().unwrap_or(0);
                PlayerRow {
                    id,
                    name: storage.get_nickname(id).unwrap_or_else(|| format!("#{id}")),
                    job: per_player_job.get(&id).cloned(),
                    damage: c.damage,
                    dps: c.damage as f64 / duration_s,
                    share: rate(c.damage, total_damage),
                    is_me: Some(id) == me,
                    crit_rate: rate(c.crit, c.hits),
                    heal,
                    hps: heal as f64 / duration_s,
                }
            })
            .collect();
        players.sort_by(|a, b| b.damage.cmp(&a.damage));

        let mut skills: Vec<SkillRow> = my_skills
            .iter()
            .filter(|(_, c)| c.damage > 0)
            .map(|(&(code, dot, summon), c)| SkillRow {
                code,
                name: data.skill_name(code),
                icon: data.skill_icon(code),
                is_dot: dot,
                damage: c.damage,
                share: rate(c.damage, my_total),
                dps: c.damage as f64 / duration_s,
                hits: c.hits,
                crit_rate: rate(c.crit, c.hits),
                back_rate: rate(c.back, c.hits),
                max_hit: c.max_hit,
                summon,
            })
            .collect();
        // One row per name: skill levels and variants share a name.
        let mut merged: Vec<SkillRow> = Vec::new();
        skills.sort_by(|a, b| b.damage.cmp(&a.damage));
        for s in skills {
            if let Some(m) = merged.iter_mut().find(|m| m.name == s.name && m.is_dot == s.is_dot && m.summon == s.summon) {
                let hits = m.hits + s.hits;
                m.crit_rate = (m.crit_rate * m.hits as f64 + s.crit_rate * s.hits as f64) / hits.max(1) as f64;
                m.back_rate = (m.back_rate * m.hits as f64 + s.back_rate * s.hits as f64) / hits.max(1) as f64;
                m.damage += s.damage;
                m.share += s.share;
                m.dps += s.dps;
                m.hits = hits;
                m.max_hit = m.max_hit.max(s.max_hit);
            } else {
                merged.push(s);
            }
        }

        let me_stats = me.and_then(|id| per_player.get(&id)).map(|c| {
            let best = merged.iter().max_by_key(|s| s.max_hit);
            let summon_damage: i64 = merged.iter().filter(|s| s.summon).map(|s| s.damage).sum();
            MeStats {
                damage: c.damage,
                dps: c.damage as f64 / duration_s,
                hits: c.hits,
                crit_rate: rate(c.crit, c.hits),
                back_rate: rate(c.back, c.hits),
                perfect_rate: rate(c.perfect, c.hits),
                double_rate: rate(c.double, c.hits),
                best_hit: best.map(|s| s.max_hit).unwrap_or(0),
                best_hit_skill: best.map(|s| s.name.clone()),
                summon_share: rate(summon_damage, c.damage),
                heal: heals.get(&me.unwrap_or(0)).copied().unwrap_or(0),
                hps: heals.get(&me.unwrap_or(0)).copied().unwrap_or(0) as f64 / duration_s,
            }
        });

        // Deaths of players in the fight, kills of your targets.
        let dead = storage.get_dead_entities();
        for id in dead.difference(&f.known_dead.clone()) {
            f.known_dead.insert(*id);
            if f.my_targets.contains(id) {
                f.kills.insert(*id);
            } else if per_player.contains_key(id) || Some(*id) == me {
                let name = storage.get_nickname(*id).unwrap_or_else(|| format!("#{id}"));
                f.deaths.push(name.clone());
                self.last_alerts.push(Alert { kind: "death".into(), text: name, icon: None });
            }
        }

        // Focus target.
        let target = f.focus.map(|id| {
            let code = storage.mob_code(id);
            let name = code.and_then(|c| data.npc_name(c)).unwrap_or_else(|| format!("#{id}"));
            let hp = storage.get_mob_current_hp(id).map(|v| v as i64);
            let max_hp = storage.get_mob_hp(id).map(|v| v as i64);
            let on_it = damage_on.get(&id).copied().unwrap_or(0);
            if self.focus_samples.back().is_some_and(|s| s.1 > on_it) {
                self.focus_samples.clear();
            }
            self.focus_samples.push_back((now, on_it));
            while self.focus_samples.front().is_some_and(|s| now - s.0 > 6000) {
                self.focus_samples.pop_front();
            }
            let ttk_s = match (hp, self.focus_samples.front()) {
                (Some(hp), Some(&(t0, d0))) if now > t0 + 500 && on_it > d0 => {
                    Some(hp as f64 / ((on_it - d0) as f64 / ((now - t0) as f64 / 1000.0)))
                }
                _ => None,
            };
            TargetView {
                id,
                name,
                is_boss: storage.is_boss(id),
                pct: match (hp, max_hp) {
                    (Some(h), Some(m)) if m > 0 => Some((h as f64 / m as f64).clamp(0.0, 1.0)),
                    _ => None,
                },
                hp,
                max_hp,
                ttk_s,
                dead: storage.is_entity_dead(id),
            }
        });

        FightView {
            id: f.start_ms,
            start_ms: f.start_ms,
            end_ms: over.then_some(end),
            duration_s,
            total_damage,
            total_dps: total_damage as f64 / duration_s,
            players,
            me: me_stats,
            skills: merged,
            target,
            targets_hit: f.my_targets.len(),
            kills: f.kills.len(),
            deaths: f.deaths.clone(),
            zone: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn watch_views(
        &self,
        storage: &DataStorage,
        data: &GameData,
        settings: &Settings,
        me: Option<i32>,
        focus: Option<i32>,
        start: i64,
        now: i64,
    ) -> Vec<WatchView> {
        let Some(me) = me else { return Vec::new() };
        if settings.watches.is_empty() {
            return Vec::new();
        }
        let mut entities: HashSet<i32> = HashSet::from([me]);
        if let Some(t) = focus {
            entities.insert(t);
        }
        let tracks = storage.fight_abnormal_tracks(start, now, &entities);
        let span = (now - start).max(1) as f64;
        settings
            .watches
            .iter()
            .map(|w: &Watch| {
                let wanted = w.name.to_lowercase();
                let matching: Vec<_> = tracks
                    .iter()
                    .filter(|t| {
                        let on_ok = match w.on.as_str() {
                            "target" => Some(t.entity) == focus && t.owner == me,
                            _ => t.entity == me,
                        };
                        on_ok && data.abnormal_name(t.abnormal).to_lowercase() == wanted
                    })
                    .collect();
                let is_open = |t: &a2t::capture::abnormal::Track| t.end_ms.is_none_or(|e| e >= now - 250);
                let active = matching.iter().any(|t| is_open(t));
                let stacks = matching
                    .iter()
                    .filter(|t| is_open(t))
                    .filter_map(|t| t.stacks.last().map(|s| s.1))
                    .max()
                    .unwrap_or(if active { 1 } else { 0 });
                // Union of the intervals it was up, inside the window.
                let mut spans: Vec<(i64, i64)> = matching
                    .iter()
                    .map(|t| (t.start_ms.max(start), t.end_ms.unwrap_or(now).min(now)))
                    .filter(|(a, b)| b > a)
                    .collect();
                spans.sort();
                let mut up = 0i64;
                let mut cur: Option<(i64, i64)> = None;
                for (a, b) in spans {
                    match cur {
                        Some((ca, cb)) if a <= cb => cur = Some((ca, cb.max(b))),
                        Some((ca, cb)) => {
                            up += cb - ca;
                            cur = Some((a, b));
                        }
                        None => cur = Some((a, b)),
                    }
                }
                if let Some((a, b)) = cur {
                    up += b - a;
                }
                let first = matching.first();
                WatchView {
                    name: w.name.clone(),
                    on: w.on.clone(),
                    icon: first.and_then(|t| data.abnormal_icon(t.abnormal)),
                    active,
                    stacks,
                    max_stacks: first.and_then(|t| data.stack_limits.get(&t.abnormal).copied()).unwrap_or(1),
                    show_stacks: w.stacks,
                    uptime: (up as f64 / span).clamp(0.0, 1.0),
                    alert: w.alert,
                }
            })
            .collect()
    }

    fn check_alerts(&mut self, tracker: &[WatchView], focus: i32, focus_dead: bool, now: i64) {
        for w in tracker.iter().filter(|w| w.alert) {
            let key = format!("{}|{}", w.on, w.name);
            let state = self.watch_state.entry(key).or_insert((false, None, false, 0));
            let on_target = w.on == "target";
            // A debuff leaving with its target (killed, or you switched) did not drop.
            if on_target && (focus != state.3 || focus_dead) {
                *state = (w.active, None, false, focus);
                continue;
            }
            if w.active {
                *state = (true, None, false, focus);
            } else if state.0 {
                let since = *state.1.get_or_insert(now);
                // A refresh can drop it for a moment: wait before alerting.
                if now - since >= 1200 && !state.2 {
                    state.2 = true;
                    self.last_alerts.push(Alert { kind: "dropped".into(), text: w.name.clone(), icon: w.icon.clone() });
                }
            }
        }
    }
}

/// A starter tracker list for a class with no preset: what you kept up in
/// your first fight. Your own timed buffs on you, and the debuffs you put on
/// your targets, by time up; passives and auras without a timer are left out.
pub fn learn_watches(
    storage: &DataStorage,
    data: &GameData,
    me: i32,
    start: i64,
    end: i64,
    targets: &[i32],
) -> Vec<Watch> {
    let mut entities: HashSet<i32> = targets.iter().copied().collect();
    entities.insert(me);
    let mut up: HashMap<(String, &'static str), i64> = HashMap::new();
    for t in storage.fight_abnormal_tracks(start, end, &entities) {
        if t.endless || t.owner != me {
            continue;
        }
        let on = if t.entity == me { "self" } else { "target" };
        let name = data.abnormal_name(t.abnormal);
        if name.starts_with('#') {
            continue;
        }
        let span = t.end_ms.unwrap_or(end).min(end) - t.start_ms.max(start);
        *up.entry((name, on)).or_default() += span.max(0);
    }
    let mut ranked: Vec<_> = up.into_iter().filter(|(_, ms)| *ms >= 3000).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    let pick = |on: &str, n: usize| {
        ranked
            .iter()
            .filter(|((_, o), _)| *o == on)
            .take(n)
            .map(|((name, o), _)| Watch { name: name.clone(), on: o.to_string(), alert: false, stacks: false })
            .collect::<Vec<_>>()
    };
    let mut out = pick("self", 4);
    out.extend(pick("target", 3));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2t::entity::damage_packet::ParsedDamagePacket;

    const ME: i32 = 5001; // Spiritmaster
    const SPIRIT: i32 = 5002; // my summon
    const GLADIATOR: i32 = 6001;
    const CLERIC: i32 = 7001;
    const STRANGER: i32 = 8001;
    const BOSS: i32 = 90001;
    const OTHER_MOB: i32 = 90002;

    fn hit(storage: &DataStorage, actor: i32, target: i32, skill: i32, dmg: i32, ts: i64) {
        let mut p = ParsedDamagePacket::new();
        p.set_actor_id(actor);
        p.set_target_id(target);
        p.set_skill_code(skill);
        p.set_damage(dmg);
        p.set_timestamp(ts);
        storage.append_damage(p);
    }

    fn setup() -> (DataStorage, GameData, Settings) {
        let storage = DataStorage::new();
        storage.set_local_player_id(Some(ME as i64));
        for (id, name) in [(ME, "Yo"), (GLADIATOR, "Glad"), (CLERIC, "Cleri"), (STRANGER, "Otro")] {
            storage.append_nickname(id, name);
        }
        storage.append_mob(BOSS, 2_000_001);
        storage.append_mob(OTHER_MOB, 2_000_002);
        storage.register_confirmed_summon_by_id(SPIRIT, ME);
        let data = GameData::load(GameData::find_dir(None), "es");
        (storage, data, Settings::default())
    }

    #[test]
    fn group_fight_with_all_roles() {
        let (storage, data, settings) = setup();
        let mut tr = Tracker::new();
        let t0 = 1_800_000_000_000i64;
        // Damage from before DaevaLens looked does not count.
        hit(&storage, ME, OTHER_MOB, 16_010_000, 999_999, t0 - 60_000);
        tr.tick(&storage, &data, &settings, t0);

        let mut ended = None;
        for i in 1..=60 {
            let now = t0 + i * 100;
            if i <= 30 {
                hit(&storage, ME, BOSS, 16_010_000, 1_000, now); // Spiritmaster skill
                hit(&storage, SPIRIT, BOSS, 16_050_000, 500, now); // spirit, counts for me
                hit(&storage, GLADIATOR, BOSS, 11_010_000, 2_000, now); // Gladiator on my target
                hit(&storage, CLERIC, BOSS, 17_010_000, 100, now); // Cleric chips in
                storage.append_heal(CLERIC, 17_020_000, 3_000, false, now);
                hit(&storage, STRANGER, OTHER_MOB, 15_010_000, 50_000, now); // not my target
            }
            let (live, rec) = tr.tick(&storage, &data, &Settings { idle_reset_s: 2, ..settings.clone() }, now + 1);
            if i == 25 {
                let f = live.fight.as_ref().expect("fight running");
                let me = f.players.iter().find(|p| p.is_me).expect("me");
                assert_eq!(me.damage, 25 * 1_500, "my hits + my spirit's since the first one, nothing from before");
                assert!(f.players.iter().all(|p| p.id != STRANGER), "stranger on another mob is out");
                assert!(f.players.iter().any(|p| p.id == GLADIATOR), "gladiator on my target is in");
            }
            if rec.is_some() {
                ended = rec;
            }
        }
        let rec = ended.expect("fight ended after the idle reset");
        let f = &rec.view;
        let me = f.players.iter().find(|p| p.is_me).unwrap();
        assert_eq!(me.damage, 30 * 1_500);
        let me_stats = f.me.as_ref().unwrap();
        assert!(me_stats.summon_share > 0.3 && me_stats.summon_share < 0.35, "{}", me_stats.summon_share);
        let cleric = f.players.iter().find(|p| p.id == CLERIC).expect("cleric listed");
        assert_eq!(cleric.heal, 30 * 3_000, "cleric healing in the window");
        assert!(cleric.hps > 0.0);
        let glad = f.players.iter().find(|p| p.id == GLADIATOR).unwrap();
        assert_eq!(glad.damage, 30 * 2_000);
        assert_eq!(f.players[0].id, GLADIATOR, "sorted by damage");
        assert!(rec.target_ids.contains(&BOSS));
    }

    #[test]
    fn every_class_is_named() {
        let (storage, data, settings) = setup();
        let mut tr = Tracker::new();
        let t0 = 1_800_000_000_000i64;
        tr.tick(&storage, &data, &settings, t0);
        // One player per class band 11..19, all on my target.
        for (k, band) in (11..=19).enumerate() {
            let id = 20_000 + k as i32;
            storage.append_nickname(id, &format!("P{band}"));
            hit(&storage, id, BOSS, band * 1_000_000 + 10_000, 100, t0 + 50);
        }
        hit(&storage, ME, BOSS, 16_010_000, 100, t0 + 50);
        let (live, _) = tr.tick(&storage, &data, &settings, t0 + 100);
        let f = live.fight.expect("fight");
        assert_eq!(f.players.len(), 10, "{:?}", f.players.iter().map(|p| (&p.name, &p.job)).collect::<Vec<_>>());
    }

    // Real records from upstream's tests (a Spiritmaster, entity 6759), and a
    // Corrode on enemy 7000 built in the same layout.
    const SELF: i32 = 6759;
    const ENEMY: i32 = 7000;
    const BENEDICTION_ON: &str = "342a38e73401138202e165a6091027000000000000bc5cba14a1010000e73403300af7000047e44c47b46a22c700e85a46";
    const BENEDICTION_OFF: &str = "0d2c38e7340100820201";
    const CORRODE_ON: &str = "342a38d8360113ac02614ba0091027000000000000bc5cba14a1010000e73403f06df6000047e44c47b46a22c700e85a46";
    const CORRODE_OFF: &str = "0d2c38d8360100ac0201";

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    fn abnormal(storage: &DataStorage, record: &str, at: i64) {
        let mut bytes = hex(record);
        // An "added" record carries the server's end time (bytes 21..29): the
        // captured one is from 2026-10-06, so move it to 10 s after `at`.
        if bytes.len() > 29 && bytes[1..3] == [0x2a, 0x38] {
            bytes[21..29].copy_from_slice(&(at + 10_000).to_le_bytes());
        }
        a2t::clock::set_override(Some(at));
        storage.note_abnormal_packet(&bytes);
        a2t::clock::set_override(None);
    }

    fn solo() -> (DataStorage, GameData) {
        let storage = DataStorage::new();
        storage.set_local_player_id(Some(SELF as i64));
        storage.append_nickname(SELF, "Yo");
        storage.append_mob(ENEMY, 2_000_001);
        storage.append_mob(ENEMY + 1, 2_000_001);
        (storage, GameData::load(GameData::find_dir(None), "es"))
    }

    fn watch(name: &str, on: &str) -> Watch {
        Watch { name: name.into(), on: on.into(), alert: true, stacks: false }
    }

    #[test]
    fn self_buff_is_tracked_and_learned() {
        let (storage, data) = solo();
        let settings = Settings { watches: vec![watch("Favor de Espíritu", "self")], idle_reset_s: 30, ..Settings::default() };
        let mut tr = Tracker::new();
        let t0 = a2t::clock::now_ms();
        tr.tick(&storage, &data, &settings, t0);
        hit(&storage, SELF, ENEMY, 16_010_000, 100, t0 + 50);
        abnormal(&storage, BENEDICTION_ON, t0 + 100);
        let (live, _) = tr.tick(&storage, &data, &settings, t0 + 1_000);
        assert!(live.tracker[0].active, "buff up: {:?}", live.tracker);
        abnormal(&storage, BENEDICTION_OFF, t0 + 6_000);
        hit(&storage, SELF, ENEMY, 16_010_000, 100, t0 + 6_500);
        let (live, _) = tr.tick(&storage, &data, &settings, t0 + 8_000);
        let w = &live.tracker[0];
        assert!(!w.active);
        assert!(w.uptime > 0.6 && w.uptime < 0.8, "5.9 s of 8 s: {}", w.uptime);
        let learned = learn_watches(&storage, &data, SELF, t0, t0 + 8_000, &[ENEMY]);
        assert!(learned.iter().any(|w| w.name == "Favor de Espíritu" && w.on == "self"), "{learned:?}");
    }

    #[test]
    fn target_debuff_alerts_only_on_a_real_drop() {
        let (storage, data) = solo();
        let settings = Settings { watches: vec![watch("Corrosión", "target")], idle_reset_s: 30, ..Settings::default() };
        let mut tr = Tracker::new();
        let t0 = a2t::clock::now_ms();
        let mut tick = |tr: &mut Tracker, at: i64| {
            hit(&storage, SELF, ENEMY, 16_010_000, 100, at - 10);
            let (live, _) = tr.tick(&storage, &data, &settings, at);
            (live.tracker[0].active, tr.last_alerts.iter().filter(|a| a.kind == "dropped").count())
        };
        tick(&mut tr, t0);
        abnormal(&storage, CORRODE_ON, t0 + 100);
        assert_eq!(tick(&mut tr, t0 + 500), (true, 0), "up on my target");
        abnormal(&storage, CORRODE_OFF, t0 + 2_000);
        assert_eq!(tick(&mut tr, t0 + 2_500), (false, 0), "not yet: could be a refresh");
        assert_eq!(tick(&mut tr, t0 + 3_400), (false, 0), "1.2 s after it was seen down, not before");
        assert_eq!(tick(&mut tr, t0 + 3_800), (false, 1), "dropped from a live target");
        assert_eq!(tick(&mut tr, t0 + 4_200).1, 0, "once");

        // Up again, then the target dies: that is not a drop.
        abnormal(&storage, CORRODE_ON, t0 + 5_000);
        assert_eq!(tick(&mut tr, t0 + 5_500), (true, 0));
        storage.mark_entity_dead(ENEMY);
        abnormal(&storage, CORRODE_OFF, t0 + 6_000);
        for at in [t0 + 6_500, t0 + 7_500, t0 + 9_000] {
            assert_eq!(tick(&mut tr, at).1, 0, "no alert for a dead target");
        }
    }
}
