//! Scheduled game events (rifts, sieges, world bosses…): when the next one is
//! and alerts a few minutes before. Pure clock arithmetic, nothing is read
//! from the game. Every time is editable from the panel, because community
//! sources disagree on some of them.

use chrono::{Datelike, Duration, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Server clocks of the global regions.
pub fn region_tz(region: &str) -> Tz {
    match region {
        "na-east" => chrono_tz::America::New_York,
        "na-west" => chrono_tz::America::Los_Angeles,
        "sa" => chrono_tz::America::Sao_Paulo,
        "asia" => chrono_tz::Asia::Tokyo,
        _ => chrono_tz::Europe::Berlin,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct EventSetting {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    /// Minutes of warning before it starts.
    pub lead_min: u32,
    /// "HH:MM" start times.
    pub times: Vec<String>,
    /// 0 = Monday … 6 = Sunday; empty = every day.
    pub days: Vec<u8>,
    /// "utc" or "server" (the region's clock).
    pub tz: String,
    /// How long it stays open (entry window), for "open now".
    pub duration_min: u32,
    /// Times agreed on by the sources we used.
    pub confirmed: bool,
    pub builtin: bool,
}

impl Default for EventSetting {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            enabled: true,
            lead_min: 5,
            times: Vec::new(),
            days: Vec::new(),
            tz: "server".into(),
            duration_min: 10,
            confirmed: true,
            builtin: false,
        }
    }
}

fn hours(list: &[u32], minute: u32) -> Vec<String> {
    list.iter().map(|h| format!("{h:02}:{minute:02}")).collect()
}

/// The built-in calendar of the global servers (EU/NA), autumn 2026.
pub fn builtin_events(language: &str) -> Vec<EventSetting> {
    let es = language == "es";
    let n = |a: &str, b: &str| if es { a } else { b }.to_string();
    let every3 = |offset: u32| (0..8).map(|i| i * 3 + offset).collect::<Vec<_>>();
    let all_hours: Vec<u32> = (0..24).collect();
    vec![
        EventSetting {
            id: "rift".into(),
            name: n("Grieta espaciotemporal", "Spacetime Rift"),
            times: hours(&every3(0), 0),
            tz: "utc".into(),
            duration_min: 10,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "kaira".into(),
            name: n("Vigilante Kaira", "Watcher Kaira"),
            times: hours(&every3(2), 0),
            tz: "utc".into(),
            duration_min: 30,
            confirmed: false,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "siege".into(),
            name: n("Asedio de artefactos", "Artifact Siege"),
            times: vec!["21:00".into()],
            days: vec![0, 3, 5],
            duration_min: 30,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "siege_bosses".into(),
            name: n("Jefes del Abismo tras el asedio", "Post-siege Abyss bosses"),
            times: vec!["21:30".into()],
            days: vec![0, 3, 5],
            duration_min: 30,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "nahma".into(),
            name: n("Señor guardián Nahma", "Guardian Lord Nahma"),
            times: vec!["21:00".into()],
            days: vec![4, 6],
            duration_min: 30,
            confirmed: false,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "shugo".into(),
            name: n("Festival Shugo", "Shugo Festival"),
            enabled: false,
            lead_min: 2,
            times: hours(&all_hours, 0),
            duration_min: 10,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "invasion".into(),
            name: n("Invasión dimensional", "Dimensional Invasion"),
            enabled: false,
            lead_min: 2,
            times: hours(&all_hours, 30),
            duration_min: 3,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "daily_reset".into(),
            name: n("Reinicio diario", "Daily reset"),
            enabled: false,
            lead_min: 30,
            times: vec!["07:00".into()],
            tz: "utc".into(),
            duration_min: 0,
            builtin: true,
            ..Default::default()
        },
        EventSetting {
            id: "weekly_reset".into(),
            name: n("Reinicio semanal", "Weekly reset"),
            enabled: false,
            lead_min: 60,
            times: vec!["07:00".into()],
            days: vec![2],
            tz: "utc".into(),
            duration_min: 0,
            builtin: true,
            ..Default::default()
        },
    ]
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Upcoming {
    pub id: String,
    pub name: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub open: bool,
    pub confirmed: bool,
}

fn weekday_index(w: Weekday) -> u8 {
    w.num_days_from_monday() as u8
}

/// Starts of `e` from a day before `now` to a week after, in UTC ms.
fn occurrences(e: &EventSetting, server: Tz, now_ms: i64) -> Vec<i64> {
    let tz: Tz = if e.tz == "utc" { chrono_tz::UTC } else { server };
    let Some(now) = Utc.timestamp_millis_opt(now_ms).single() else { return Vec::new() };
    let today = now.with_timezone(&tz).date_naive();
    let times: Vec<NaiveTime> = e.times.iter().filter_map(|t| NaiveTime::parse_from_str(t.trim(), "%H:%M").ok()).collect();
    let mut out = Vec::new();
    for offset in -1..=8 {
        let day = today + Duration::days(offset);
        if !e.days.is_empty() && !e.days.contains(&weekday_index(day.weekday())) {
            continue;
        }
        for t in &times {
            // A time skipped by a DST jump has no instant; an ambiguous one takes the first.
            if let Some(dt) = tz.from_local_datetime(&day.and_time(*t)).earliest() {
                out.push(dt.timestamp_millis());
            }
        }
    }
    out.sort();
    out
}

/// The next (or currently open) occurrence of every enabled event, soonest first.
pub fn upcoming(events: &[EventSetting], region: &str, now_ms: i64) -> Vec<Upcoming> {
    let server = region_tz(region);
    let mut out: Vec<Upcoming> = events
        .iter()
        .filter(|e| e.enabled)
        .filter_map(|e| {
            let dur = e.duration_min as i64 * 60_000;
            let start = occurrences(e, server, now_ms).into_iter().find(|s| s + dur.max(1) > now_ms)?;
            Some(Upcoming {
                id: e.id.clone(),
                name: e.name.clone(),
                start_ms: start,
                end_ms: start + dur,
                open: start <= now_ms,
                confirmed: e.confirmed,
            })
        })
        .collect();
    out.sort_by_key(|u| u.start_ms);
    out
}

/// Alerts once per occurrence: `lead_min` before it starts, and when it starts.
#[derive(Default)]
pub struct EventAlerts {
    sent: HashSet<(String, i64, bool)>,
}

pub struct EventAlert {
    pub name: String,
    pub minutes: i64,
}

impl EventAlerts {
    pub fn check(&mut self, events: &[EventSetting], list: &[Upcoming], now_ms: i64) -> Vec<EventAlert> {
        let mut out = Vec::new();
        for u in list {
            let Some(e) = events.iter().find(|e| e.id == u.id) else { continue };
            let lead_ms = e.lead_min as i64 * 60_000;
            let to_start = u.start_ms - now_ms;
            // Warning: inside the lead window, and not when it is already about to start.
            if e.lead_min > 0 && to_start <= lead_ms && to_start > 30_000 && self.sent.insert((u.id.clone(), u.start_ms, false)) {
                out.push(EventAlert { name: u.name.clone(), minutes: (to_start + 59_999) / 60_000 });
            }
            // Start: within the first minute only (not when DaevaLens opens mid-event).
            if to_start <= 0 && to_start > -60_000 && self.sent.insert((u.id.clone(), u.start_ms, true)) {
                out.push(EventAlert { name: u.name.clone(), minutes: 0 });
            }
        }
        // Forget what is long past.
        self.sent.retain(|(_, start, _)| now_ms - start < 6 * 3_600_000);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(s: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis()
    }

    #[test]
    fn rift_every_three_hours_utc() {
        let ev = builtin_events("es");
        let list = upcoming(&ev, "eu", ms("2026-10-12T16:40:00Z"));
        let rift = list.iter().find(|u| u.id == "rift").unwrap();
        assert_eq!(rift.start_ms, ms("2026-10-12T18:00:00Z"));
        assert!(!rift.open);
        // 18:05: inside the 10-minute entry window, it is open.
        let list = upcoming(&ev, "eu", ms("2026-10-12T18:05:00Z"));
        let rift = list.iter().find(|u| u.id == "rift").unwrap();
        assert!(rift.open && rift.start_ms == ms("2026-10-12T18:00:00Z"));
    }

    #[test]
    fn siege_on_server_clock_across_dst() {
        let ev = builtin_events("es");
        // Monday 12 Oct 2026, Berlin on CEST (UTC+2): 21:00 local = 19:00 UTC.
        let s = upcoming(&ev, "eu", ms("2026-10-12T10:00:00Z")).into_iter().find(|u| u.id == "siege").unwrap();
        assert_eq!(s.start_ms, ms("2026-10-12T19:00:00Z"));
        // Saturday 31 Oct, after the switch to CET (UTC+1): 20:00 UTC.
        let s = upcoming(&ev, "eu", ms("2026-10-30T10:00:00Z")).into_iter().find(|u| u.id == "siege").unwrap();
        assert_eq!(s.start_ms, ms("2026-10-31T20:00:00Z"));
        // NA East: Monday 21:00 EDT = Tuesday 01:00 UTC.
        let s = upcoming(&ev, "na-east", ms("2026-10-12T10:00:00Z")).into_iter().find(|u| u.id == "siege").unwrap();
        assert_eq!(s.start_ms, ms("2026-10-13T01:00:00Z"));
    }

    #[test]
    fn alerts_once_before_and_at_start() {
        let ev = builtin_events("es");
        let mut alerts = EventAlerts::default();
        let rift_alerts = |alerts: &mut EventAlerts, t: &str| {
            let now = ms(t);
            alerts.check(&ev, &upcoming(&ev, "eu", now), now).into_iter().filter(|a| a.name.starts_with("Grieta")).map(|a| a.minutes).collect::<Vec<_>>()
        };
        assert!(rift_alerts(&mut alerts, "2026-10-12T17:50:00Z").is_empty());
        assert_eq!(rift_alerts(&mut alerts, "2026-10-12T17:55:00Z"), vec![5]);
        assert!(rift_alerts(&mut alerts, "2026-10-12T17:56:00Z").is_empty());
        assert_eq!(rift_alerts(&mut alerts, "2026-10-12T18:00:10Z"), vec![0]);
        assert!(rift_alerts(&mut alerts, "2026-10-12T18:00:40Z").is_empty());
    }
}
