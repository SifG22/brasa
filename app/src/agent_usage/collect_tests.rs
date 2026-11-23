use std::fs;

use chrono::{TimeZone, Utc};

use super::*;

fn utc(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 22, hour, minute, 0).unwrap()
}

fn claude_assistant(session: &str, id: &str, time: &str, entrypoint: &str) -> String {
    format!(
        r#"{{"type":"assistant","sessionId":"{session}","timestamp":"{time}","cwd":"C:\\Users\\me\\app","entrypoint":"{entrypoint}","requestId":"req_{id}","message":{{"id":"{id}","model":"claude-opus-5-5","content":[{{"type":"text","text":"hi"}}],"usage":{{"input_tokens":2,"cache_creation_input_tokens":100,"cache_read_input_tokens":900,"output_tokens":50}}}}}}"#
    )
}

fn claude_prompt(session: &str, time: &str) -> String {
    format!(
        r#"{{"type":"user","sessionId":"{session}","timestamp":"{time}","cwd":"C:\\Users\\me\\app","entrypoint":"cli","message":{{"role":"user","content":"add a button"}}}}"#
    )
}

#[test]
fn reads_claude_usage_once_per_message() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("C--Users-me-app");
    fs::create_dir_all(&project).unwrap();
    let tool_result = r#"{"type":"user","sessionId":"s1","timestamp":"2026-11-22T10:00:30Z","message":{"role":"user","content":[{"tool_use_id":"t1","type":"tool_result","content":"ok"}]}}"#;
    let lines = [
        claude_prompt("s1", "2026-11-22T10:00:00Z"),
        claude_assistant("s1", "msg_1", "2026-11-22T10:00:10Z", "cli"),
        // Claude Code writes one line per content block with the same usage.
        claude_assistant("s1", "msg_1", "2026-11-22T10:00:11Z", "cli"),
        tool_result.to_string(),
        claude_assistant("s1", "msg_2", "2026-11-22T10:01:00Z", "cli"),
    ];
    fs::write(project.join("s1.jsonl"), lines.join("\n")).unwrap();

    let sources = UsageSources {
        claude_dirs: vec![dir.path().to_path_buf()],
        ..Default::default()
    };
    let data = collect(&sources, Vec::new());

    assert_eq!(data.sessions.len(), 1);
    assert_eq!(data.sessions[0].turns.len(), 1);
    assert_eq!(data.records.len(), 2);
    let record = &data.records[0];
    assert_eq!(record.input_tokens, 1002);
    assert_eq!(record.cached_tokens, 900);
    assert_eq!(record.output_tokens, 50);
    assert_eq!(record.model.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(record.place, UsagePlace::OtherTerminal);
}

#[test]
fn reads_codex_token_counts_without_repeats() {
    let dir = tempfile::tempdir().unwrap();
    let day = dir.path().join("2026").join("11").join("22");
    fs::create_dir_all(&day).unwrap();
    let count = |time: &str, total: u64| {
        format!(
            r#"{{"timestamp":"{time}","type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"input_tokens":{total},"cached_input_tokens":0,"output_tokens":10}},"last_token_usage":{{"input_tokens":100,"cached_input_tokens":40,"output_tokens":10}}}}}}}}"#
        )
    };
    let lines = [
        r#"{"timestamp":"2026-11-22T09:00:00Z","type":"session_meta","payload":{"id":"c1","cwd":"C:\\Users\\me\\app","originator":"codex-tui"}}"#.to_string(),
        r#"{"timestamp":"2026-11-22T09:00:01Z","type":"turn_context","payload":{"model":"gpt-6.2-astra"}}"#.to_string(),
        r#"{"timestamp":"2026-11-22T09:00:02Z","type":"event_msg","payload":{"type":"user_message","message":"fix it"}}"#.to_string(),
        count("2026-11-22T09:00:10Z", 100),
        // Same total again: only the rate limits changed.
        count("2026-11-22T09:00:11Z", 100),
        count("2026-11-22T09:00:20Z", 200),
    ];
    fs::write(day.join("rollout.jsonl"), lines.join("\n")).unwrap();

    let sources = UsageSources {
        codex_dirs: vec![dir.path().to_path_buf()],
        ..Default::default()
    };
    let data = collect(&sources, Vec::new());

    assert_eq!(data.sessions.len(), 1);
    assert_eq!(data.sessions[0].agent, UsageAgent::Codex);
    assert_eq!(data.sessions[0].turns.len(), 1);
    assert_eq!(data.records.len(), 2);
    assert_eq!(data.records[0].cached_tokens, 40);
    assert_eq!(data.records[0].model.as_deref(), Some("gpt-6.2-astra"));
}

#[test]
fn recognizes_agent_commands() {
    assert_eq!(
        UsageAgent::from_command("claude"),
        Some(UsageAgent::ClaudeCode)
    );
    assert_eq!(
        UsageAgent::from_command("C:\\tools\\codex.exe --full-auto"),
        Some(UsageAgent::Codex)
    );
    assert_eq!(
        UsageAgent::from_command("cd 'C:\\x'; ollama run 'qwen2.5-coder:3b'"),
        Some(UsageAgent::LocalAi)
    );
    assert_eq!(UsageAgent::from_command("git status"), None);
}

#[test]
fn open_agent_runs_end_at_the_next_command_in_their_session() {
    let commands = vec![
        BrasaCommand {
            command: "claude".to_string(),
            pwd: Some("C:\\Users\\me\\app".to_string()),
            start: Some(utc(10, 0)),
            end: None,
            brasa_session: Some(1),
        },
        BrasaCommand {
            command: "git status".to_string(),
            pwd: Some("C:\\Users\\me\\app".to_string()),
            start: Some(utc(11, 0)),
            end: Some(utc(11, 1)),
            brasa_session: Some(1),
        },
    ];
    let runs = agent_runs_from_commands(commands);
    assert_eq!(
        runs,
        vec![BrasaAgentRun {
            agent: UsageAgent::ClaudeCode,
            pwd: Some("C:\\Users\\me\\app".to_string()),
            start: utc(10, 0),
            end: Some(utc(11, 0)),
        }]
    );
}

#[test]
fn attributes_terminal_sessions_started_in_brasa() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    fs::create_dir_all(&project).unwrap();
    let lines = [
        claude_prompt("in-brasa", "2026-11-22T10:00:30Z"),
        claude_assistant("in-brasa", "msg_a", "2026-11-22T10:01:00Z", "cli"),
        claude_prompt("elsewhere", "2026-11-22T15:00:00Z"),
        claude_assistant("elsewhere", "msg_b", "2026-11-22T15:01:00Z", "cli"),
        claude_assistant("editor", "msg_c", "2026-11-22T16:00:00Z", "claude-vscode"),
    ];
    fs::write(project.join("s.jsonl"), lines.join("\n")).unwrap();
    let sources = UsageSources {
        claude_dirs: vec![dir.path().to_path_buf()],
        ..Default::default()
    };
    let commands = vec![BrasaCommand {
        command: "claude".to_string(),
        pwd: Some("C:\\Users\\me\\app\\".to_string()),
        start: Some(utc(10, 0)),
        end: Some(utc(12, 0)),
        brasa_session: Some(7),
    }];

    let data = collect(&sources, commands);

    let place_of = |session: usize| data.sessions[session].place;
    assert_eq!(place_of(0), UsagePlace::Brasa);
    assert_eq!(place_of(1), UsagePlace::OtherTerminal);
    assert_eq!(place_of(2), UsagePlace::OtherEditor);
    assert_eq!(data.records[0].place, UsagePlace::Brasa);
}
