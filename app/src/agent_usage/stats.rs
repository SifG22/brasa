//! Aggregates usage records into what the dashboard shows for a date range.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Timelike, Utc};
use enum_iterator::{Sequence, all};
use thousands::Separable;

use super::collect::{UsageAgent, UsageData, UsagePlace};

/// Activity further apart than this is idle time, not agent time.
const IDLE_GAP: Duration = Duration::minutes(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Sequence)]
pub enum UsageRange {
    Week,
    Month,
    Quarter,
    All,
}

impl UsageRange {
    pub fn label(self) -> &'static str {
        match self {
            UsageRange::Week => "7 days",
            UsageRange::Month => "30 days",
            UsageRange::Quarter => "90 days",
            UsageRange::All => "All",
        }
    }

    fn days(self) -> Option<i64> {
        match self {
            UsageRange::Week => Some(7),
            UsageRange::Month => Some(30),
            UsageRange::Quarter => Some(90),
            UsageRange::All => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Sequence)]
pub enum Provider {
    Anthropic,
    OpenAi,
    Google,
    Xai,
    Other,
}

impl Provider {
    pub fn display_name(self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic",
            Provider::OpenAi => "OpenAI",
            Provider::Google => "Google",
            Provider::Xai => "xAI",
            Provider::Other => "Other",
        }
    }

    pub fn for_model(model: &str) -> Self {
        let model = model.to_lowercase();
        let has = |needles: &[&str]| needles.iter().any(|needle| model.contains(needle));
        if has(&["claude", "opus", "sonnet", "haiku", "fable"]) {
            Provider::Anthropic
        } else if has(&["gpt", "codex", "o1", "o3", "o4"]) {
            Provider::OpenAi
        } else if has(&["gemini", "gemma"]) {
            Provider::Google
        } else if has(&["grok"]) {
            Provider::Xai
        } else {
            Provider::Other
        }
    }
}

/// A short, readable model name: `claude-opus-5-5` becomes `Opus 5.5`. Other models keep their
/// own names, which are already readable.
pub fn model_display_name(model: &str) -> String {
    let Some(rest) = model.strip_prefix("claude-") else {
        return model.to_string();
    };
    let mut family = None;
    let mut version = Vec::new();
    for part in rest.split('-') {
        let is_number = part.chars().all(|c| c.is_ascii_digit());
        if is_number && part.len() < 8 {
            version.push(part);
        } else if !is_number && family.is_none() {
            let mut chars = part.chars();
            family = chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect::<String>());
        }
    }
    match family {
        Some(family) if version.is_empty() => family,
        Some(family) => format!("{family} {}", version.join(".")),
        None => model.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub input_tokens: u64,
    pub cached_tokens: u64,
    pub output_tokens: u64,
    pub tasks: u64,
    pub turns: u64,
    pub agent_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DayStats {
    pub date: NaiveDate,
    pub totals: Totals,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelStats {
    pub name: String,
    pub provider: Provider,
    pub totals: Totals,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentStats {
    pub agent: UsageAgent,
    pub totals: Totals,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceStats {
    pub place: UsagePlace,
    pub totals: Totals,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsageSummary {
    pub first_day: NaiveDate,
    pub today: NaiveDate,
    pub totals: Totals,
    /// The same length of time just before this range; `None` for all time.
    pub previous: Option<Totals>,
    pub max_concurrent: usize,
    pub days: Vec<DayStats>,
    /// Activity by weekday (Monday first) and hour.
    pub heatmap: [[u64; 24]; 7],
    /// Sorted by tokens, largest first.
    pub models: Vec<ModelStats>,
    /// Sorted by tokens, largest first.
    pub providers: Vec<(Provider, u64)>,
    /// Every agent with activity in the range, ignoring the agent filter.
    pub agents: Vec<AgentStats>,
    pub places: Vec<PlaceStats>,
}

impl UsageSummary {
    pub fn days_with_activity(&self) -> usize {
        self.days
            .iter()
            .filter(|day| day.totals.turns > 0 || day.totals.input_tokens > 0)
            .count()
    }

    /// The busiest weekday (Monday is 0) and hour, if there was any activity.
    pub fn busiest_slot(&self) -> Option<(usize, usize)> {
        let mut best = None;
        let mut best_count = 0;
        for (weekday, hours) in self.heatmap.iter().enumerate() {
            for (hour, count) in hours.iter().enumerate() {
                if *count > best_count {
                    best_count = *count;
                    best = Some((weekday, hour));
                }
            }
        }
        best
    }
}

/// Summarizes `data` for `range`, counting only `agent` when one is given.
pub fn summarize(
    data: &UsageData,
    range: UsageRange,
    agent: Option<UsageAgent>,
    now: DateTime<Local>,
) -> UsageSummary {
    let today = now.date_naive();
    let first_day = match range.days() {
        Some(days) => today - Duration::days(days - 1),
        None => earliest_day(data).unwrap_or(today).min(today),
    };
    let start = local_midnight(first_day);
    let end = now.with_timezone(&Utc);
    let wanted = |session_agent: UsageAgent| agent.is_none_or(|agent| agent == session_agent);

    let window = Window::compute(data, start, end, &wanted);
    let previous = range.days().map(|days| {
        let previous_start = start - Duration::days(days);
        Window::compute(data, previous_start, start, &wanted).totals_where(data, |_| true)
    });
    let all_agents = Window::compute(data, start, end, &|_| true);

    let day_count = (today - first_day).num_days() + 1;
    let mut days: Vec<DayStats> = (0..day_count)
        .map(|offset| DayStats {
            date: first_day + Duration::days(offset),
            totals: Totals::default(),
        })
        .collect();
    let day_index = |time: DateTime<Utc>| {
        usize::try_from((time.with_timezone(&Local).date_naive() - first_day).num_days()).ok()
    };
    let mut heatmap = [[0u64; 24]; 7];

    for record in &window.records {
        let record = &data.records[*record];
        if let Some(day) = day_index(record.timestamp).and_then(|index| days.get_mut(index)) {
            day.totals.input_tokens += record.input_tokens;
            day.totals.cached_tokens += record.cached_tokens;
            day.totals.output_tokens += record.output_tokens;
        }
        bump_heatmap(&mut heatmap, record.timestamp);
    }
    for (session, turns) in &window.turns {
        for turn in turns {
            if let Some(day) = day_index(*turn).and_then(|index| days.get_mut(index)) {
                day.totals.turns += 1;
            }
            bump_heatmap(&mut heatmap, *turn);
        }
        if !data.sessions[*session].is_subagent
            && let Some(first) = window.first_activity.get(session)
            && let Some(day) = day_index(*first).and_then(|index| days.get_mut(index))
        {
            day.totals.tasks += 1;
        }
    }
    for (from, to) in window.intervals.values().flatten() {
        if let Some(day) = day_index(*from).and_then(|index| days.get_mut(index)) {
            day.totals.agent_seconds += (*to - *from).num_seconds();
        }
    }

    let mut models: HashMap<String, ModelStats> = HashMap::new();
    let mut model_sessions: HashMap<String, HashSet<usize>> = HashMap::new();
    for record in &window.records {
        let record = &data.records[*record];
        let Some(model) = &record.model else {
            continue;
        };
        let name = model_display_name(model);
        let stats = models.entry(name.clone()).or_insert_with(|| ModelStats {
            name: name.clone(),
            provider: Provider::for_model(model),
            totals: Totals::default(),
        });
        stats.totals.input_tokens += record.input_tokens;
        stats.totals.cached_tokens += record.cached_tokens;
        stats.totals.output_tokens += record.output_tokens;
        if !data.sessions[record.session].is_subagent {
            model_sessions
                .entry(name)
                .or_default()
                .insert(record.session);
        }
    }
    let mut models: Vec<ModelStats> = models
        .into_values()
        .map(|mut stats| {
            stats.totals.tasks = model_sessions
                .get(&stats.name)
                .map_or(0, |s| s.len() as u64);
            stats
        })
        .collect();
    models.sort_by_key(|stats| std::cmp::Reverse(token_total(&stats.totals)));

    let mut providers: HashMap<Provider, u64> = HashMap::new();
    for stats in &models {
        *providers.entry(stats.provider).or_default() += token_total(&stats.totals);
    }
    let mut providers: Vec<(Provider, u64)> = providers.into_iter().collect();
    providers.sort_by_key(|(_, tokens)| std::cmp::Reverse(*tokens));

    let agents = all::<UsageAgent>()
        .map(|agent| AgentStats {
            agent,
            totals: all_agents.totals_where(data, |session| data.sessions[session].agent == agent),
        })
        .filter(|stats| stats.totals != Totals::default())
        .collect();
    let places = all::<UsagePlace>()
        .map(|place| PlaceStats {
            place,
            totals: window.totals_for_place(data, place),
        })
        .filter(|stats| stats.totals != Totals::default())
        .collect();

    UsageSummary {
        first_day,
        today,
        totals: window.totals_where(data, |_| true),
        previous,
        max_concurrent: window.max_concurrent(),
        days,
        heatmap,
        models,
        providers,
        agents,
        places,
    }
}

pub fn token_total(totals: &Totals) -> u64 {
    totals.input_tokens + totals.output_tokens
}

/// `11.6B`, `402M`, `22.9K` or `950`.
pub fn format_tokens(tokens: u64) -> String {
    let (value, suffix) = match tokens {
        0..1_000 => return tokens.to_string(),
        1_000..1_000_000 => (tokens as f64 / 1e3, "K"),
        1_000_000..1_000_000_000 => (tokens as f64 / 1e6, "M"),
        _ => (tokens as f64 / 1e9, "B"),
    };
    let rounded = if value >= 99.95 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}").trim_end_matches(".0").to_string()
    };
    format!("{rounded}{suffix}")
}

/// `1,110`.
pub fn format_count(count: u64) -> String {
    count.separate_with_commas()
}

/// `347 h`, `9.2 h`, `23 min` or `—` when there is no time.
pub fn format_duration(seconds: i64) -> String {
    let hours = seconds as f64 / 3600.;
    if seconds <= 0 {
        "—".to_string()
    } else if seconds < 60 {
        "<1 min".to_string()
    } else if hours < 1. {
        format!("{} min", seconds / 60)
    } else if hours < 10. {
        format!("{} h", format!("{hours:.1}").trim_end_matches(".0"))
    } else {
        format!("{hours:.0} h")
    }
}

/// Percent change from `previous` to `current`, capped at ±999; `None` without a previous value.
pub fn percent_change(current: u64, previous: u64) -> Option<i64> {
    if previous == 0 {
        return None;
    }
    let change = (current as f64 - previous as f64) / previous as f64 * 100.;
    Some((change.round() as i64).clamp(-999, 999))
}

fn bump_heatmap(heatmap: &mut [[u64; 24]; 7], time: DateTime<Utc>) {
    let local = time.with_timezone(&Local);
    let weekday = local.weekday().num_days_from_monday() as usize;
    heatmap[weekday][local.hour() as usize] += 1;
}

fn local_midnight(day: NaiveDate) -> DateTime<Utc> {
    let midnight = day.and_hms_opt(0, 0, 0).unwrap_or_default();
    Local
        .from_local_datetime(&midnight)
        .earliest()
        .map(|time| time.with_timezone(&Utc))
        .unwrap_or_else(|| Utc.from_utc_datetime(&midnight))
}

fn earliest_day(data: &UsageData) -> Option<NaiveDate> {
    let first_record = data.records.first().map(|record| record.timestamp);
    let first_turn = data
        .sessions
        .iter()
        .filter_map(|session| session.turns.first().copied())
        .min();
    [first_record, first_turn]
        .into_iter()
        .flatten()
        .min()
        .map(|time| time.with_timezone(&Local).date_naive())
}

/// The records, prompts and active time of the sessions that fall within a time window.
struct Window {
    records: Vec<usize>,
    turns: HashMap<usize, Vec<DateTime<Utc>>>,
    first_activity: HashMap<usize, DateTime<Utc>>,
    intervals: HashMap<usize, Vec<(DateTime<Utc>, DateTime<Utc>)>>,
}

impl Window {
    fn compute(
        data: &UsageData,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        wanted: &dyn Fn(UsageAgent) -> bool,
    ) -> Self {
        let in_window = |time: &DateTime<Utc>| *time >= start && *time < end;
        let mut records = Vec::new();
        let mut activity: HashMap<usize, Vec<DateTime<Utc>>> = HashMap::new();
        for (index, record) in data.records.iter().enumerate() {
            if in_window(&record.timestamp) && wanted(data.sessions[record.session].agent) {
                records.push(index);
                activity
                    .entry(record.session)
                    .or_default()
                    .push(record.timestamp);
            }
        }

        let mut turns = HashMap::new();
        let mut intervals = HashMap::new();
        for (index, session) in data.sessions.iter().enumerate() {
            if !wanted(session.agent) {
                continue;
            }
            let session_turns: Vec<DateTime<Utc>> =
                session.turns.iter().copied().filter(in_window).collect();
            if let Some((from, to)) = session.known_span {
                let (from, to) = (from.max(start), to.min(end));
                if from < to {
                    intervals.insert(index, vec![(from, to)]);
                }
            }
            if !session_turns.is_empty() {
                activity
                    .entry(index)
                    .or_default()
                    .extend(session_turns.iter().copied());
                turns.insert(index, session_turns);
            }
        }

        let mut first_activity = HashMap::new();
        for (session, times) in &mut activity {
            times.sort();
            if let Some(first) = times.first() {
                first_activity.insert(*session, *first);
            }
            if !intervals.contains_key(session) {
                intervals.insert(*session, active_intervals(times));
            }
        }
        for session in intervals.keys() {
            if let Some((from, _)) = intervals[session].first() {
                first_activity.entry(*session).or_insert(*from);
            }
        }
        for session in first_activity.keys() {
            turns.entry(*session).or_default();
        }

        Self {
            records,
            turns,
            first_activity,
            intervals,
        }
    }

    fn totals_where(&self, data: &UsageData, include: impl Fn(usize) -> bool) -> Totals {
        let mut totals = Totals::default();
        for record in &self.records {
            let record = &data.records[*record];
            if include(record.session) {
                totals.input_tokens += record.input_tokens;
                totals.cached_tokens += record.cached_tokens;
                totals.output_tokens += record.output_tokens;
            }
        }
        for (session, turns) in &self.turns {
            if !include(*session) {
                continue;
            }
            totals.turns += turns.len() as u64;
            if !data.sessions[*session].is_subagent {
                totals.tasks += 1;
            }
        }
        for (session, intervals) in &self.intervals {
            if include(*session) {
                totals.agent_seconds += intervals
                    .iter()
                    .map(|(from, to)| (*to - *from).num_seconds())
                    .sum::<i64>();
            }
        }
        totals
    }

    fn totals_for_place(&self, data: &UsageData, place: UsagePlace) -> Totals {
        let mut totals = Totals::default();
        for record in &self.records {
            let record = &data.records[*record];
            if record.place == place {
                totals.input_tokens += record.input_tokens;
                totals.cached_tokens += record.cached_tokens;
                totals.output_tokens += record.output_tokens;
            }
        }
        let session_totals =
            self.totals_where(data, |session| data.sessions[session].place == place);
        totals.tasks = session_totals.tasks;
        totals.turns = session_totals.turns;
        totals.agent_seconds = session_totals.agent_seconds;
        totals
    }

    fn max_concurrent(&self) -> usize {
        let mut edges: Vec<(DateTime<Utc>, i32)> = self
            .intervals
            .values()
            .flatten()
            .filter(|(from, to)| from < to)
            .flat_map(|(from, to)| [(*from, 1), (*to, -1)])
            .collect();
        // Ends sort before starts at the same instant, so back-to-back work is not concurrent.
        edges.sort();
        let mut current = 0i32;
        let mut max = 0i32;
        for (_, change) in edges {
            current += change;
            max = max.max(current);
        }
        max as usize
    }
}

/// Joins activity closer together than [`IDLE_GAP`] into active stretches.
fn active_intervals(times: &[DateTime<Utc>]) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut intervals: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
    for time in times {
        match intervals.last_mut() {
            Some((_, to)) if *time - *to <= IDLE_GAP => *to = (*to).max(*time),
            _ => intervals.push((*time, *time)),
        }
    }
    intervals.retain(|(from, to)| from < to);
    intervals
}

#[cfg(test)]
#[path = "stats_tests.rs"]
mod tests;
