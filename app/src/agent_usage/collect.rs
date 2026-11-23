//! Reads the history that AI coding agents keep on this computer and turns it into usage
//! records. Nothing leaves the machine: every source is a local file.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, TimeZone, Utc};
use enum_iterator::Sequence;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Sequence)]
pub enum UsageAgent {
    ClaudeCode,
    Codex,
    GeminiCli,
    OpenCode,
    /// The local model that Brasa runs with Ollama.
    LocalAi,
}

impl UsageAgent {
    pub fn display_name(self) -> &'static str {
        match self {
            UsageAgent::ClaudeCode => "Claude Code",
            UsageAgent::Codex => "Codex",
            UsageAgent::GeminiCli => "Gemini CLI",
            UsageAgent::OpenCode => "opencode",
            UsageAgent::LocalAi => "Local AI (Ollama)",
        }
    }

    /// The agent started by a command typed in a terminal, if any.
    pub fn from_command(command: &str) -> Option<Self> {
        if command.contains("ollama run") {
            return Some(UsageAgent::LocalAi);
        }
        let program = command.split_whitespace().next()?;
        let program = program.trim_matches(|c| c == '\'' || c == '"' || c == '&');
        let name = Path::new(program)
            .file_stem()?
            .to_str()?
            .to_ascii_lowercase();
        match name.as_str() {
            "claude" => Some(UsageAgent::ClaudeCode),
            "codex" => Some(UsageAgent::Codex),
            "gemini" => Some(UsageAgent::GeminiCli),
            "opencode" => Some(UsageAgent::OpenCode),
            _ => None,
        }
    }
}

/// How a session was started, as recorded by the agent itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Launcher {
    Terminal,
    Editor,
    DesktopApp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Sequence)]
pub enum UsagePlace {
    Brasa,
    BrasaCode,
    OtherTerminal,
    OtherEditor,
    DesktopApp,
}

impl UsagePlace {
    pub fn display_name(self) -> &'static str {
        match self {
            UsagePlace::Brasa => "Brasa",
            UsagePlace::BrasaCode => "Brasa Code",
            UsagePlace::OtherTerminal => "Other terminals",
            UsagePlace::OtherEditor => "Other editors",
            UsagePlace::DesktopApp => "Desktop apps",
        }
    }
}

#[derive(Clone, Debug)]
pub struct UsageSession {
    pub agent: UsageAgent,
    /// Sessions an agent opens on its own (such as Codex reviewers) add tokens and time, but they
    /// are not tasks the user asked for.
    pub is_subagent: bool,
    /// Normalized working directory, or for Gemini CLI the hash it stores instead.
    pub location_key: Option<String>,
    pub launcher: Launcher,
    pub place: UsagePlace,
    /// When the user sent each prompt.
    pub turns: Vec<DateTime<Utc>>,
    /// Exact start and end for sessions without token records, such as local AI runs.
    pub known_span: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

#[derive(Clone, Debug)]
pub struct UsageRecord {
    pub session: usize,
    pub timestamp: DateTime<Utc>,
    pub model: Option<String>,
    /// Every prompt token, cached or not.
    pub input_tokens: u64,
    pub cached_tokens: u64,
    pub output_tokens: u64,
    pub place: UsagePlace,
}

#[derive(Clone, Debug, Default)]
pub struct UsageData {
    pub sessions: Vec<UsageSession>,
    pub records: Vec<UsageRecord>,
    /// Codex's plan limits each time it reported them, oldest first.
    pub codex_limits: Vec<LimitSample>,
}

/// An agent's plan limits at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct LimitSample {
    pub time: DateTime<Utc>,
    pub plan: Option<String>,
    pub windows: Vec<LimitReading>,
}

/// How much of one limit window was used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LimitReading {
    pub length: Duration,
    pub used_percent: f32,
    pub resets_at: DateTime<Utc>,
}

/// One row of Brasa's command history.
#[derive(Clone, Debug)]
pub struct BrasaCommand {
    pub command: String,
    pub pwd: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub brasa_session: Option<i64>,
}

/// An agent that ran in a Brasa terminal.
#[derive(Clone, Debug, PartialEq)]
pub struct BrasaAgentRun {
    pub agent: UsageAgent,
    pub pwd: Option<String>,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

/// Where to look for history.
#[derive(Clone, Debug, Default)]
pub struct UsageSources {
    pub claude_dirs: Vec<PathBuf>,
    pub codex_dirs: Vec<PathBuf>,
    pub gemini_dirs: Vec<PathBuf>,
    pub opencode_dirs: Vec<PathBuf>,
    pub brasa_databases: Vec<PathBuf>,
    pub brasa_code_workspaces: Option<PathBuf>,
}

impl UsageSources {
    pub fn detect(brasa_database: Option<PathBuf>) -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let env_dirs = |var: &str| -> Vec<PathBuf> {
            std::env::var(var)
                .map(|value| value.split(',').map(PathBuf::from).collect())
                .unwrap_or_default()
        };

        let mut claude_dirs: Vec<PathBuf> = env_dirs("CLAUDE_CONFIG_DIR")
            .into_iter()
            .map(|dir| dir.join("projects"))
            .collect();
        if claude_dirs.is_empty() {
            claude_dirs = vec![
                home.join(".claude").join("projects"),
                home.join(".config").join("claude").join("projects"),
            ];
        }

        let codex_home = std::env::var("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".codex"));
        let codex_dirs = vec![
            codex_home.join("sessions"),
            codex_home.join("archived_sessions"),
        ];

        let gemini_dirs = vec![home.join(".gemini").join("tmp")];

        let data_home = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".local").join("share"));
        let opencode_dirs = vec![data_home.join("opencode").join("storage")];

        // Brasa kept its history in warp.sqlite before it was renamed, so older runs live there.
        let brasa_databases = brasa_database
            .map(|database| {
                let legacy = database.with_file_name("warp.sqlite");
                vec![database, legacy]
            })
            .unwrap_or_default();

        let brasa_code_workspaces = dirs::config_dir()
            .map(|dir| dir.join("Brasa Code").join("User").join("workspaceStorage"));

        Self {
            claude_dirs,
            codex_dirs,
            gemini_dirs,
            opencode_dirs,
            brasa_databases,
            brasa_code_workspaces,
        }
    }
}

/// Reads every source and works out where each session ran.
pub fn collect(sources: &UsageSources, brasa_commands: Vec<BrasaCommand>) -> UsageData {
    let mut builder = UsageBuilder::default();
    for dir in &sources.claude_dirs {
        for file in files_with_extension(dir, "jsonl") {
            builder.read_claude_file(&file);
        }
    }
    for dir in &sources.codex_dirs {
        for file in files_with_extension(dir, "jsonl") {
            builder.read_codex_file(&file);
        }
    }
    for dir in &sources.gemini_dirs {
        for file in files_with_extension(dir, "json") {
            let in_chats_dir = file
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == "chats");
            if in_chats_dir {
                builder.read_gemini_file(&file);
            }
        }
    }
    for dir in &sources.opencode_dirs {
        for file in files_with_extension(&dir.join("message"), "json") {
            builder.read_opencode_file(&file);
        }
    }

    let runs = agent_runs_from_commands(brasa_commands);
    for run in runs.iter().filter(|run| run.agent == UsageAgent::LocalAi) {
        builder.add_local_ai_run(run);
    }

    let brasa_code_folders = sources
        .brasa_code_workspaces
        .as_deref()
        .map(read_brasa_code_folders)
        .unwrap_or_default();

    let mut data = builder.finish();
    attribute_places(&mut data, &runs, &brasa_code_folders);
    data
}

fn files_with_extension(dir: &Path, extension: &str) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().is_some_and(|ext| ext == extension))
        .collect()
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|time| time.with_timezone(&Utc))
}

fn timestamp_from_millis(value: &Value) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(value.as_i64()?).single()
}

fn u64_at(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or_default()
}

/// Lowercased on Windows, where paths are case-insensitive, and without trailing separators.
pub fn normalize_dir(path: &str) -> String {
    let trimmed = path.trim_end_matches(['/', '\\']);
    if cfg!(windows) {
        trimmed.replace('/', "\\").to_lowercase()
    } else {
        trimmed.to_string()
    }
}

fn gemini_project_hash(path: &str) -> String {
    format!("{:x}", Sha256::digest(path.as_bytes()))
}

#[derive(Default)]
struct UsageBuilder {
    sessions: Vec<UsageSession>,
    session_ids: HashMap<(UsageAgent, String), usize>,
    records: Vec<UsageRecord>,
    seen_messages: HashSet<String>,
    codex_limits: Vec<LimitSample>,
}

impl UsageBuilder {
    fn session(&mut self, agent: UsageAgent, id: &str, launcher: Launcher) -> usize {
        if let Some(index) = self.session_ids.get(&(agent, id.to_string())) {
            return *index;
        }
        let index = self.sessions.len();
        self.sessions.push(UsageSession {
            agent,
            is_subagent: false,
            location_key: None,
            launcher,
            place: UsagePlace::OtherTerminal,
            turns: Vec::new(),
            known_span: None,
        });
        self.session_ids.insert((agent, id.to_string()), index);
        index
    }

    fn push_record(
        &mut self,
        session: usize,
        timestamp: DateTime<Utc>,
        model: Option<String>,
        input_tokens: u64,
        cached_tokens: u64,
        output_tokens: u64,
    ) {
        if input_tokens == 0 && output_tokens == 0 {
            return;
        }
        self.records.push(UsageRecord {
            session,
            timestamp,
            model,
            input_tokens,
            cached_tokens,
            output_tokens,
            place: UsagePlace::OtherTerminal,
        });
    }

    fn read_claude_file(&mut self, path: &Path) {
        let Ok(file) = File::open(path) else {
            return;
        };
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            self.read_claude_line(&line);
        }
    }

    fn read_claude_line(&mut self, line: &str) {
        // Most lines are tool results; skipping them before parsing keeps big histories fast.
        if line.contains("\"usage\"") && line.contains("\"type\":\"assistant\"") {
            if let Ok(entry) = serde_json::from_str::<ClaudeLine<ClaudeResponse>>(line) {
                self.add_claude_response(entry);
            }
        } else if line.contains("\"type\":\"user\"") && !line.contains("\"tool_use_id\"") {
            if let Ok(entry) = serde_json::from_str::<ClaudeLine<ClaudePrompt>>(line) {
                self.add_claude_prompt(entry);
            }
        }
    }

    /// The session a Claude Code line belongs to, and when it was written.
    fn claude_session<M>(&mut self, entry: &ClaudeLine<M>) -> Option<(usize, DateTime<Utc>)> {
        let session_id = entry.session_id.as_deref()?;
        let timestamp = entry.timestamp.as_deref().and_then(parse_timestamp)?;
        let launcher = match entry.entrypoint.as_deref() {
            Some(entrypoint) if entrypoint.contains("desktop") => Launcher::DesktopApp,
            Some(entrypoint)
                if ["vscode", "ide", "jetbrains"]
                    .iter()
                    .any(|editor| entrypoint.contains(editor)) =>
            {
                Launcher::Editor
            }
            _ => Launcher::Terminal,
        };
        let session = self.session(UsageAgent::ClaudeCode, session_id, launcher);
        if let Some(cwd) = entry.cwd.as_deref() {
            self.sessions[session]
                .location_key
                .get_or_insert_with(|| normalize_dir(cwd));
        }
        Some((session, timestamp))
    }

    fn add_claude_response(&mut self, entry: ClaudeLine<ClaudeResponse>) {
        let Some((session, timestamp)) = self.claude_session(&entry) else {
            return;
        };
        let Some(ClaudeResponse {
            id,
            model,
            usage: Some(usage),
        }) = entry.message
        else {
            return;
        };
        // Claude Code writes one line per content block, all with the same usage.
        if let Some(id) = id {
            let key = format!("{id}:{}", entry.request_id.unwrap_or_default());
            if !self.seen_messages.insert(key) {
                return;
            }
        }
        let model = model.filter(|model| !model.starts_with('<'));
        let cached = usage.cache_read_input_tokens.unwrap_or_default();
        let input = usage.input_tokens.unwrap_or_default()
            + usage.cache_creation_input_tokens.unwrap_or_default()
            + cached;
        let output = usage.output_tokens.unwrap_or_default();
        self.push_record(session, timestamp, model, input, cached, output);
    }

    fn add_claude_prompt(&mut self, entry: ClaudeLine<ClaudePrompt>) {
        if entry.kind.as_deref() != Some("user") {
            return;
        }
        let Some((session, timestamp)) = self.claude_session(&entry) else {
            return;
        };
        let skip = entry.is_meta.unwrap_or_default()
            || entry.is_sidechain.unwrap_or_default()
            || entry.is_compact_summary.unwrap_or_default();
        let typed = entry
            .message
            .and_then(|message| message.content)
            .is_some_and(|content| is_typed_prompt(&content));
        if !skip && typed {
            self.sessions[session].turns.push(timestamp);
        }
    }

    fn read_codex_file(&mut self, path: &Path) {
        let Ok(file) = File::open(path) else {
            return;
        };
        let mut session = None;
        let mut model = None;
        let mut previous_total = None;
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            let relevant = [
                "session_meta",
                "turn_context",
                "token_count",
                "user_message",
            ]
            .iter()
            .any(|kind| line.contains(kind));
            if !relevant {
                continue;
            }
            let Ok(entry) = serde_json::from_str::<CodexLine>(&line) else {
                continue;
            };
            let timestamp = entry.timestamp.as_deref().and_then(parse_timestamp);
            match entry.kind.as_str() {
                "session_meta" => {
                    let payload = &entry.payload;
                    let Some(id) = payload.get("id").and_then(Value::as_str) else {
                        continue;
                    };
                    let originator = payload
                        .get("originator")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_lowercase();
                    let launcher = if originator.contains("desktop") {
                        Launcher::DesktopApp
                    } else if originator.contains("vscode") {
                        Launcher::Editor
                    } else {
                        Launcher::Terminal
                    };
                    let index = self.session(UsageAgent::Codex, id, launcher);
                    let state = &mut self.sessions[index];
                    state.is_subagent = payload
                        .get("source")
                        .is_some_and(|source| source.get("subagent").is_some());
                    if let Some(cwd) = payload.get("cwd").and_then(Value::as_str) {
                        state.location_key = Some(normalize_dir(cwd));
                    }
                    session = Some(index);
                }
                "turn_context" => {
                    if let Some(name) = entry.payload.get("model").and_then(Value::as_str) {
                        model = Some(name.to_string());
                    }
                }
                "event_msg" => {
                    let (Some(session), Some(timestamp)) = (session, timestamp) else {
                        continue;
                    };
                    match entry.payload.get("type").and_then(Value::as_str) {
                        Some("user_message") => self.sessions[session].turns.push(timestamp),
                        Some("token_count") => {
                            if let Some(sample) = entry
                                .payload
                                .get("rate_limits")
                                .and_then(|limits| codex_limit_sample(limits, timestamp))
                            {
                                self.codex_limits.push(sample);
                            }
                            let Some(info) = entry.payload.get("info") else {
                                continue;
                            };
                            let total = info.get("total_token_usage").cloned();
                            // Codex repeats the last count when only the rate limits change.
                            if total.is_some() && total == previous_total {
                                continue;
                            }
                            let Some(usage) = info.get("last_token_usage").cloned() else {
                                continue;
                            };
                            previous_total = total;
                            self.push_record(
                                session,
                                timestamp,
                                model.clone(),
                                u64_at(&usage, "input_tokens"),
                                u64_at(&usage, "cached_input_tokens"),
                                u64_at(&usage, "output_tokens"),
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn read_gemini_file(&mut self, path: &Path) {
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(chat) = serde_json::from_str::<Value>(&text) else {
            return;
        };
        let Some(id) = chat.get("sessionId").and_then(Value::as_str) else {
            return;
        };
        let session = self.session(UsageAgent::GeminiCli, id, Launcher::Terminal);
        if let Some(hash) = chat.get("projectHash").and_then(Value::as_str) {
            self.sessions[session].location_key = Some(hash.to_string());
        }
        let messages = chat.get("messages").and_then(Value::as_array);
        for message in messages.into_iter().flatten() {
            let Some(timestamp) = message
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(parse_timestamp)
            else {
                continue;
            };
            match message.get("type").and_then(Value::as_str) {
                Some("user") => self.sessions[session].turns.push(timestamp),
                Some("gemini") => {
                    let Some(tokens) = message.get("tokens") else {
                        continue;
                    };
                    let model = message
                        .get("model")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    self.push_record(
                        session,
                        timestamp,
                        model,
                        u64_at(tokens, "input") + u64_at(tokens, "tool"),
                        u64_at(tokens, "cached"),
                        u64_at(tokens, "output") + u64_at(tokens, "thoughts"),
                    );
                }
                _ => {}
            }
        }
    }

    fn read_opencode_file(&mut self, path: &Path) {
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(message) = serde_json::from_str::<Value>(&text) else {
            return;
        };
        let (Some(id), Some(timestamp)) = (
            message.get("sessionID").and_then(Value::as_str),
            message
                .get("time")
                .and_then(|time| time.get("created"))
                .and_then(timestamp_from_millis),
        ) else {
            return;
        };
        let session = self.session(UsageAgent::OpenCode, id, Launcher::Terminal);
        if let Some(cwd) = message
            .get("path")
            .and_then(|path| path.get("cwd"))
            .and_then(Value::as_str)
        {
            self.sessions[session]
                .location_key
                .get_or_insert_with(|| normalize_dir(cwd));
        }
        match message.get("role").and_then(Value::as_str) {
            Some("user") => self.sessions[session].turns.push(timestamp),
            Some("assistant") => {
                let Some(tokens) = message.get("tokens") else {
                    return;
                };
                let cache = tokens.get("cache").cloned().unwrap_or_default();
                let cached = u64_at(&cache, "read");
                let model = message
                    .get("modelID")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                self.push_record(
                    session,
                    timestamp,
                    model,
                    u64_at(tokens, "input") + cached + u64_at(&cache, "write"),
                    cached,
                    u64_at(tokens, "output") + u64_at(tokens, "reasoning"),
                );
            }
            _ => {}
        }
    }

    fn add_local_ai_run(&mut self, run: &BrasaAgentRun) {
        let id = format!("{}", run.start.timestamp_millis());
        let session = self.session(UsageAgent::LocalAi, &id, Launcher::Terminal);
        let state = &mut self.sessions[session];
        state.place = UsagePlace::Brasa;
        state.turns.push(run.start);
        state.known_span = run.end.map(|end| (run.start, end));
    }

    fn finish(mut self) -> UsageData {
        for session in &mut self.sessions {
            session.turns.sort();
        }
        self.records.sort_by_key(|record| record.timestamp);
        self.codex_limits.sort_by_key(|sample| sample.time);
        UsageData {
            sessions: self.sessions,
            records: self.records,
            codex_limits: self.codex_limits,
        }
    }
}

fn codex_limit_sample(limits: &Value, time: DateTime<Utc>) -> Option<LimitSample> {
    let windows: Vec<LimitReading> = ["primary", "secondary"]
        .iter()
        .filter_map(|name| {
            let window = limits.get(name)?;
            Some(LimitReading {
                length: Duration::minutes(window.get("window_minutes")?.as_i64()?),
                used_percent: window.get("used_percent")?.as_f64()? as f32,
                resets_at: Utc
                    .timestamp_opt(window.get("resets_at")?.as_i64()?, 0)
                    .single()?,
            })
        })
        .collect();
    (!windows.is_empty()).then(|| LimitSample {
        time,
        plan: limits
            .get("plan_type")
            .and_then(Value::as_str)
            .map(str::to_string),
        windows,
    })
}

/// Whether a user message is a prompt the person typed rather than tool output or a notice.
fn is_typed_prompt(content: &Value) -> bool {
    let text = match content {
        Value::String(text) => Some(text.as_str()),
        Value::Array(blocks) => blocks
            .iter()
            .find(|block| block.get("type").and_then(Value::as_str) == Some("text"))
            .and_then(|block| block.get("text"))
            .and_then(Value::as_str),
        _ => None,
    };
    text.is_some_and(|text| {
        !text.starts_with("<local-command-stdout>") && !text.starts_with("[Request interrupted")
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeLine<M> {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<String>,
    session_id: Option<String>,
    cwd: Option<String>,
    entrypoint: Option<String>,
    request_id: Option<String>,
    is_meta: Option<bool>,
    is_sidechain: Option<bool>,
    is_compact_summary: Option<bool>,
    message: Option<M>,
}

/// An API response. Its content is left unparsed, since only the usage matters.
#[derive(Deserialize)]
struct ClaudeResponse {
    id: Option<String>,
    model: Option<String>,
    usage: Option<ClaudeUsage>,
}

#[derive(Deserialize)]
struct ClaudePrompt {
    content: Option<Value>,
}

#[derive(Deserialize)]
struct ClaudeUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct CodexLine {
    timestamp: Option<String>,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    payload: Value,
}

/// Turns Brasa's command history into agent runs. A run without an end lasts until the next
/// command in the same Brasa session, since nothing else runs there while the agent is open.
pub fn agent_runs_from_commands(mut commands: Vec<BrasaCommand>) -> Vec<BrasaAgentRun> {
    commands.sort_by_key(|command| command.start);
    let mut next_start: HashMap<i64, DateTime<Utc>> = HashMap::new();
    let mut runs = Vec::new();
    for command in commands.iter().rev() {
        let Some(start) = command.start else {
            continue;
        };
        if let Some(agent) = UsageAgent::from_command(&command.command) {
            let next = command
                .brasa_session
                .and_then(|session| next_start.get(&session).copied());
            let end = command.end.or(next).unwrap_or(start + Duration::hours(12));
            runs.push(BrasaAgentRun {
                agent,
                pwd: command.pwd.clone(),
                start,
                end: (command.end.is_some() || next.is_some()).then_some(end),
            });
        }
        if let Some(session) = command.brasa_session {
            next_start.insert(session, start);
        }
    }
    runs.reverse();
    runs
}

/// Grace before a Brasa command's start, since its timestamp and the agent's clock can differ.
const RUN_START_GRACE: Duration = Duration::minutes(2);
/// How long a run is assumed to last when Brasa never saw it end.
const OPEN_RUN_LENGTH: Duration = Duration::hours(12);

fn attribute_places(
    data: &mut UsageData,
    runs: &[BrasaAgentRun],
    brasa_code_folders: &HashSet<String>,
) {
    let mut windows: HashMap<(UsageAgent, String), Vec<(DateTime<Utc>, DateTime<Utc>)>> =
        HashMap::new();
    for run in runs {
        let Some(pwd) = &run.pwd else {
            continue;
        };
        let key = if run.agent == UsageAgent::GeminiCli {
            gemini_project_hash(pwd)
        } else {
            normalize_dir(pwd)
        };
        let end = run.end.unwrap_or(run.start + OPEN_RUN_LENGTH);
        windows
            .entry((run.agent, key))
            .or_default()
            .push((run.start - RUN_START_GRACE, end));
    }

    let place_at = |session: &UsageSession, time: DateTime<Utc>| match session.launcher {
        Launcher::DesktopApp => UsagePlace::DesktopApp,
        Launcher::Editor => {
            let in_brasa_code = session
                .location_key
                .as_ref()
                .is_some_and(|key| brasa_code_folders.contains(key));
            if in_brasa_code {
                UsagePlace::BrasaCode
            } else {
                UsagePlace::OtherEditor
            }
        }
        Launcher::Terminal => {
            let in_brasa = session.location_key.as_ref().is_some_and(|key| {
                windows
                    .get(&(session.agent, key.clone()))
                    .is_some_and(|spans| {
                        spans
                            .iter()
                            .any(|(start, end)| (*start..=*end).contains(&time))
                    })
            });
            if in_brasa {
                UsagePlace::Brasa
            } else {
                UsagePlace::OtherTerminal
            }
        }
    };

    for record in &mut data.records {
        record.place = place_at(&data.sessions[record.session], record.timestamp);
    }
    let mut first_record_time: HashMap<usize, DateTime<Utc>> = HashMap::new();
    for record in &data.records {
        first_record_time
            .entry(record.session)
            .or_insert(record.timestamp);
    }
    for (index, session) in data.sessions.iter_mut().enumerate() {
        if session.agent == UsageAgent::LocalAi {
            continue;
        }
        let first = session
            .turns
            .first()
            .copied()
            .or_else(|| first_record_time.get(&index).copied());
        if let Some(first) = first {
            session.place = place_at(session, first);
        }
    }
}

/// The folders Brasa Code has opened, normalized like session directories.
fn read_brasa_code_folders(workspace_storage: &Path) -> HashSet<String> {
    let Ok(entries) = std::fs::read_dir(workspace_storage) else {
        return HashSet::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| std::fs::read_to_string(entry.path().join("workspace.json")).ok())
        .filter_map(|text| serde_json::from_str::<Value>(&text).ok())
        .filter_map(|workspace| {
            let folder = workspace.get("folder")?.as_str()?;
            let path = url::Url::parse(folder).ok()?.to_file_path().ok()?;
            Some(normalize_dir(&path.to_string_lossy()))
        })
        .collect()
}

/// Reads Brasa's command history from a database file. Missing files give no commands.
#[cfg(feature = "local_fs")]
pub fn read_brasa_commands(database: &Path) -> Vec<BrasaCommand> {
    use chrono::NaiveDateTime;
    use diesel::{QueryDsl, RunQueryDsl};

    use crate::persistence::{establish_ro_connection, schema::commands::dsl};

    let Some(url) = database.is_file().then(|| database.to_str()).flatten() else {
        return Vec::new();
    };
    let Ok(mut connection) = establish_ro_connection(url) else {
        return Vec::new();
    };
    let rows = dsl::commands
        .select((
            dsl::command,
            dsl::pwd,
            dsl::start_ts,
            dsl::completed_ts,
            dsl::session_id,
        ))
        .load::<(
            String,
            Option<String>,
            Option<NaiveDateTime>,
            Option<NaiveDateTime>,
            Option<i64>,
        )>(&mut connection)
        .unwrap_or_default();
    rows.into_iter()
        .map(|(command, pwd, start, end, brasa_session)| BrasaCommand {
            command,
            pwd,
            start: start.map(|time| Utc.from_utc_datetime(&time)),
            end: end.map(|time| Utc.from_utc_datetime(&time)),
            brasa_session,
        })
        .collect()
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
