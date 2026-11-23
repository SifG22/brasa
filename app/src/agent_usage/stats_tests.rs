use chrono::{Local, Utc};

use super::*;
use crate::agent_usage::collect::{Launcher, UsageRecord, UsageSession};

fn session(agent: UsageAgent, turns: Vec<DateTime<Utc>>) -> UsageSession {
    UsageSession {
        agent,
        is_subagent: false,
        location_key: None,
        launcher: Launcher::Terminal,
        place: UsagePlace::OtherTerminal,
        turns,
        known_span: None,
    }
}

fn record(
    session: usize,
    time: DateTime<Utc>,
    model: &str,
    input: u64,
    output: u64,
) -> UsageRecord {
    UsageRecord {
        session,
        timestamp: time,
        model: Some(model.to_string()),
        input_tokens: input,
        cached_tokens: input / 2,
        output_tokens: output,
        place: UsagePlace::OtherTerminal,
    }
}

#[test]
fn formats_numbers_like_the_dashboard() {
    assert_eq!(format_tokens(950), "950");
    assert_eq!(format_tokens(22_900), "22.9K");
    assert_eq!(format_tokens(963_000), "963K");
    assert_eq!(format_tokens(61_300_000), "61.3M");
    assert_eq!(format_tokens(10_000_000_000), "10B");
    assert_eq!(format_tokens(11_600_000_000), "11.6B");
    assert_eq!(format_count(1110), "1,110");
    assert_eq!(format_duration(0), "—");
    assert_eq!(format_duration(23 * 60), "23 min");
    assert_eq!(format_duration(9 * 3600 + 720), "9.2 h");
    assert_eq!(format_duration(347 * 3600), "347 h");
    assert_eq!(percent_change(200, 100), Some(100));
    assert_eq!(percent_change(10_000, 1), Some(999));
    assert_eq!(percent_change(5, 0), None);
}

#[test]
fn names_models_readably() {
    assert_eq!(model_display_name("claude-opus-5-5"), "Opus 5.5");
    assert_eq!(
        model_display_name("claude-sonnet-4-5-20250929"),
        "Sonnet 4.5"
    );
    assert_eq!(model_display_name("claude-3-5-haiku-20241022"), "Haiku 3.5");
    assert_eq!(model_display_name("gpt-6.2-astra"), "gpt-6.2-astra");
    assert_eq!(Provider::for_model("gemini-4-argon-high"), Provider::Google);
    assert_eq!(Provider::for_model("gpt-6.2-astra"), Provider::OpenAi);
}

#[test]
fn summarizes_tokens_tasks_and_time() {
    let now = Local::now();
    let start = now.with_timezone(&Utc) - Duration::hours(2);
    let minute = |n: i64| start + Duration::minutes(n);
    let data = UsageData {
        sessions: vec![
            session(UsageAgent::ClaudeCode, vec![minute(0)]),
            session(UsageAgent::Codex, vec![minute(1)]),
        ],
        records: vec![
            record(0, minute(1), "claude-opus-5-5", 1000, 10),
            record(0, minute(4), "claude-opus-5-5", 3000, 30),
            record(1, minute(2), "gpt-6.2-astra", 500, 5),
            // Long after the previous activity: idle time in between is not agent time.
            record(1, minute(60), "gpt-6.2-astra", 500, 5),
        ],
        ..Default::default()
    };

    let summary = summarize(&data, UsageRange::Week, None, now);
    assert_eq!(summary.totals.input_tokens, 5000);
    assert_eq!(summary.totals.output_tokens, 50);
    assert_eq!(summary.totals.tasks, 2);
    assert_eq!(summary.totals.turns, 2);
    assert_eq!(summary.totals.agent_seconds, 4 * 60 + 60);
    assert_eq!(summary.max_concurrent, 2);
    assert_eq!(summary.days.len(), 7);
    assert_eq!(summary.models[0].name, "Opus 5.5");
    assert_eq!(summary.models[0].totals.tasks, 1);
    assert_eq!(summary.providers[0].0, Provider::Anthropic);
    assert_eq!(summary.agents.len(), 2);

    let only_codex = summarize(&data, UsageRange::Week, Some(UsageAgent::Codex), now);
    assert_eq!(only_codex.totals.input_tokens, 1000);
    assert_eq!(only_codex.totals.tasks, 1);
    assert_eq!(only_codex.agents.len(), 2);
}

#[test]
fn local_ai_runs_count_their_whole_span() {
    let now = Local::now();
    let start = now.with_timezone(&Utc) - Duration::hours(3);
    let mut local = session(UsageAgent::LocalAi, vec![start]);
    local.place = UsagePlace::Brasa;
    local.known_span = Some((start, start + Duration::minutes(30)));
    let data = UsageData {
        sessions: vec![local],
        records: Vec::new(),
        ..Default::default()
    };

    let summary = summarize(&data, UsageRange::Month, None, now);
    assert_eq!(summary.totals.agent_seconds, 30 * 60);
    assert_eq!(summary.totals.tasks, 1);
    assert_eq!(summary.places[0].place, UsagePlace::Brasa);
}
