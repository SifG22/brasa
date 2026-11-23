//! Plan limits: how much of each agent's subscription window is used, and where it is heading.

#[cfg(not(target_family = "wasm"))]
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
#[cfg(not(target_family = "wasm"))]
use serde::{Deserialize, Serialize};
#[cfg(not(target_family = "wasm"))]
use serde_json::Value;

use super::collect::{LimitReading, LimitSample, UsageAgent};

/// Limits up to this long are per-session windows (such as 5 hours); longer ones are weekly.
const SESSION_WINDOW_MAX: Duration = Duration::hours(24);
/// Readings of the same window can report reset times a few seconds apart.
const RESET_TOLERANCE: Duration = Duration::minutes(2);
/// How long saved Claude readings are kept.
#[cfg(not(target_family = "wasm"))]
const SNAPSHOT_RETENTION: Duration = Duration::days(8);
/// Claude readings this recent are reused, since Anthropic limits how often it can be asked.
#[cfg(not(target_family = "wasm"))]
const CLAUDE_MIN_REFRESH: Duration = Duration::minutes(3);
#[cfg(not(target_family = "wasm"))]
const CLAUDE_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

#[derive(Clone, Debug, PartialEq)]
pub struct LimitWindow {
    pub used_percent: f32,
    pub resets_at: DateTime<Utc>,
    pub length: Duration,
    /// Usage seen during this window, oldest first.
    pub history: Vec<(DateTime<Utc>, f32)>,
    /// The last reading's window already ended and the agent has not reported since, so the limit
    /// is full again and the next reset is not known yet.
    pub expired: bool,
}

/// One day of a limit window, for the chart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowDay {
    pub date: NaiveDate,
    /// Usage at the end of the day, or now for today; `None` for days still to come.
    pub used_percent: Option<f32>,
    /// Usage at the end of the day if the current pace holds.
    pub projected_percent: f32,
}

impl LimitWindow {
    pub fn start(&self) -> DateTime<Utc> {
        self.resets_at - self.length
    }

    /// Usage per hour so far in this window.
    fn pace(&self, now: DateTime<Utc>) -> f32 {
        let elapsed_hours = (now - self.start()).num_minutes().max(1) as f32 / 60.;
        self.used_percent / elapsed_hours
    }

    /// Usage at `time` if the pace so far holds.
    fn projected_at(&self, time: DateTime<Utc>, now: DateTime<Utc>) -> f32 {
        let remaining_hours = (time - now).num_minutes().max(0) as f32 / 60.;
        self.used_percent + self.pace(now) * remaining_hours
    }

    /// Usage at the reset if the pace so far holds.
    pub fn projected_percent(&self, now: DateTime<Utc>) -> f32 {
        self.projected_at(self.resets_at, now)
    }

    /// When the limit runs out at the current pace, if that happens before the reset.
    pub fn runs_out_at(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let pace = self.pace(now);
        if pace <= 0. || self.projected_percent(now) < 100. {
            return None;
        }
        let hours = (100. - self.used_percent).max(0.) / pace;
        Some(now + Duration::minutes((hours * 60.) as i64))
    }

    /// Each local day of the window, with the usage reached by its end.
    pub fn days(&self, now: DateTime<Utc>) -> Vec<WindowDay> {
        let first = self.start().with_timezone(&Local).date_naive();
        let last = (self.resets_at - Duration::seconds(1))
            .with_timezone(&Local)
            .date_naive();
        let today = now.with_timezone(&Local).date_naive();
        let mut days = Vec::new();
        let mut date = first;
        while date <= last {
            let end = local_day_end(date).min(self.resets_at);
            let used_percent = match date.cmp(&today) {
                std::cmp::Ordering::Less => Some(
                    self.history
                        .iter()
                        .rev()
                        .find(|(time, _)| *time <= end)
                        .map_or(0., |(_, used)| *used),
                ),
                std::cmp::Ordering::Equal => Some(self.used_percent),
                std::cmp::Ordering::Greater => None,
            };
            days.push(WindowDay {
                date,
                used_percent,
                projected_percent: self.projected_at(end, now),
            });
            date += Duration::days(1);
        }
        // A window that starts partway through a day spans one calendar day more than its
        // length; the first day's usage is already part of the next one.
        let whole_days = (self.length.num_hours() as f64 / 24.).ceil() as usize;
        if whole_days > 0 && days.len() > whole_days {
            days.drain(..days.len() - whole_days);
        }
        days
    }
}

fn local_day_end(date: NaiveDate) -> DateTime<Utc> {
    let next = (date + Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap_or_default();
    Local
        .from_local_datetime(&next)
        .earliest()
        .map(|time| time.with_timezone(&Utc))
        .unwrap_or_else(|| Utc.from_utc_datetime(&next))
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanLimits {
    pub agent: UsageAgent,
    pub plan: Option<String>,
    pub session: Option<LimitWindow>,
    pub weekly: Option<LimitWindow>,
    /// Why these limits may be out of date, such as Anthropic asking Brasa to wait.
    pub notice: Option<String>,
}

/// Codex's current limits, from the readings it saved in its session files.
pub fn codex_limits(samples: &[LimitSample], now: DateTime<Utc>) -> Option<PlanLimits> {
    let latest = samples.last()?;
    let window = |is_session: bool| {
        let current = latest
            .windows
            .iter()
            .find(|reading| (reading.length <= SESSION_WINDOW_MAX) == is_session)?;
        Some(window_from_readings(current, samples, now))
    };
    Some(PlanLimits {
        agent: UsageAgent::Codex,
        plan: latest.plan.as_deref().map(capitalize),
        session: window(true),
        weekly: window(false),
        notice: None,
    })
}

/// The window `current` belongs to, with every reading of it. A window that has already reset
/// starts again from zero.
fn window_from_readings(
    current: &LimitReading,
    samples: &[LimitSample],
    now: DateTime<Utc>,
) -> LimitWindow {
    if current.resets_at <= now {
        return LimitWindow {
            used_percent: 0.,
            resets_at: current.resets_at,
            length: current.length,
            history: Vec::new(),
            expired: true,
        };
    }
    let same_window = |reading: &LimitReading| {
        reading.length == current.length
            && (reading.resets_at - current.resets_at).abs() <= RESET_TOLERANCE
    };
    let history = samples
        .iter()
        .filter_map(|sample| {
            let reading = sample.windows.iter().find(|reading| same_window(reading))?;
            Some((sample.time, reading.used_percent))
        })
        .collect();
    LimitWindow {
        used_percent: current.used_percent,
        resets_at: current.resets_at,
        length: current.length,
        history,
        expired: false,
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// A saved Claude reading, so the weekly chart has a history.
#[cfg(not(target_family = "wasm"))]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClaudeSnapshot {
    time: DateTime<Utc>,
    session: Option<(f32, DateTime<Utc>)>,
    weekly: Option<(f32, DateTime<Utc>)>,
}

/// Asks Anthropic for the Claude plan limits of the Claude Code login on this computer, the same
/// way Claude Code's `/usage` does, and saves the reading to `history_file`. A recent reading is
/// reused instead of asking again, and the last one is shown when Anthropic cannot answer.
#[cfg(not(target_family = "wasm"))]
pub fn fetch_claude_limits(history_file: &Path, now: DateTime<Utc>) -> Result<PlanLimits, String> {
    let credentials = claude_config_dirs()
        .into_iter()
        .find_map(|dir| std::fs::read_to_string(dir.join(".credentials.json")).ok())
        .ok_or("No Claude Code login found on this computer.")?;
    let credentials: Value = serde_json::from_str(&credentials)
        .map_err(|_| "Could not read the Claude Code login.".to_string())?;
    let oauth = credentials
        .get("claudeAiOauth")
        .ok_or("Claude Code is not signed in with a Claude plan.")?;
    let token = oauth
        .get("accessToken")
        .and_then(Value::as_str)
        .ok_or("Claude Code is not signed in with a Claude plan.")?;
    let plan = oauth
        .get("subscriptionType")
        .and_then(Value::as_str)
        .map(capitalize);

    let mut history = read_snapshots(history_file);
    history.retain(|saved| now - saved.time <= SNAPSHOT_RETENTION);
    if let Some(latest) = history.last()
        && now - latest.time < CLAUDE_MIN_REFRESH
    {
        return Ok(claude_limits_from(&history, plan, now, None));
    }

    let expired = oauth
        .get("expiresAt")
        .and_then(Value::as_i64)
        .is_some_and(|expires| expires < now.timestamp_millis());
    let reading = if expired {
        Err("Your Claude Code login has expired. Open Claude Code once to renew it.".to_string())
    } else {
        ask_anthropic(token, now)
    };
    match reading {
        Ok(snapshot) => {
            history.push(snapshot);
            if let Ok(text) = serde_json::to_string(&history)
                && let Err(err) = std::fs::write(history_file, text)
            {
                log::warn!("Could not save the Claude limits history: {err}");
            }
            Ok(claude_limits_from(&history, plan, now, None))
        }
        Err(message) => match history.last() {
            Some(latest) => {
                let checked = latest.time.with_timezone(&Local).format("%H:%M");
                let notice = format!("{message} Showing the reading from {checked}.");
                Ok(claude_limits_from(&history, plan, now, Some(notice)))
            }
            None => Err(message),
        },
    }
}

/// One reading of the Claude plan limits from Anthropic.
#[cfg(not(target_family = "wasm"))]
fn ask_anthropic(token: &str, now: DateTime<Utc>) -> Result<ClaudeSnapshot, String> {
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .and_then(|client| {
            client
                .get(CLAUDE_USAGE_URL)
                .bearer_auth(token)
                .header("anthropic-beta", "oauth-2025-04-20")
                .header("User-Agent", "brasa")
                .send()
        })
        .map_err(|_| "Could not reach Anthropic. Check your connection.".to_string())?;
    match response.status().as_u16() {
        200..=299 => {}
        401 => {
            return Err(
                "Anthropic did not accept your Claude Code login. Open Claude Code once to renew it."
                    .into(),
            );
        }
        429 => return Err("Anthropic asked to wait a few minutes before asking again.".into()),
        status => return Err(format!("Anthropic answered with error {status}.")),
    }
    let body: Value = response
        .json()
        .map_err(|_| "Anthropic sent an answer Brasa does not understand.".to_string())?;
    let reading = |key: &str| -> Option<(f32, DateTime<Utc>)> {
        let window = body.get(key)?;
        let used = window.get("utilization")?.as_f64()? as f32;
        let resets_at = DateTime::parse_from_rfc3339(window.get("resets_at")?.as_str()?)
            .ok()?
            .with_timezone(&Utc);
        Some((used, resets_at))
    };
    let snapshot = ClaudeSnapshot {
        time: now,
        session: reading("five_hour"),
        weekly: reading("seven_day"),
    };
    if snapshot.session.is_none() && snapshot.weekly.is_none() {
        return Err("Anthropic did not report any limits for this plan.".into());
    }
    Ok(snapshot)
}

/// The Claude limits as of the latest saved reading, with the history of each window.
#[cfg(not(target_family = "wasm"))]
fn claude_limits_from(
    history: &[ClaudeSnapshot],
    plan: Option<String>,
    now: DateTime<Utc>,
    notice: Option<String>,
) -> PlanLimits {
    let latest = history.last();
    let window = |pick: fn(&ClaudeSnapshot) -> Option<(f32, DateTime<Utc>)>, length: Duration| {
        let (used_percent, resets_at) = pick(latest?)?;
        let readings = history
            .iter()
            .filter_map(|saved| {
                let (used, saved_reset) = pick(saved)?;
                ((saved_reset - resets_at).abs() <= RESET_TOLERANCE).then_some((saved.time, used))
            })
            .collect();
        Some(LimitWindow {
            used_percent: if resets_at <= now { 0. } else { used_percent },
            resets_at,
            length,
            history: readings,
            expired: resets_at <= now,
        })
    };
    PlanLimits {
        agent: UsageAgent::ClaudeCode,
        plan,
        session: window(|saved| saved.session, Duration::hours(5)),
        weekly: window(|saved| saved.weekly, Duration::days(7)),
        notice,
    }
}

#[cfg(not(target_family = "wasm"))]
fn read_snapshots(path: &Path) -> Vec<ClaudeSnapshot> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

#[cfg(not(target_family = "wasm"))]
fn claude_config_dirs() -> Vec<PathBuf> {
    let from_env: Vec<PathBuf> = std::env::var("CLAUDE_CONFIG_DIR")
        .map(|value| value.split(',').map(PathBuf::from).collect())
        .unwrap_or_default();
    if !from_env.is_empty() {
        return from_env;
    }
    let home = dirs::home_dir().unwrap_or_default();
    vec![home.join(".claude"), home.join(".config").join("claude")]
}

#[cfg(test)]
#[path = "limits_tests.rs"]
mod tests;
