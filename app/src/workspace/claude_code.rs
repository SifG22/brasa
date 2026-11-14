use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The longest command output, in characters, handed to Claude Code. Errors are almost always at the
/// end, so longer output keeps its tail.
const MAX_OUTPUT_CHARS: usize = 12_000;

/// The command that starts Claude Code, first moving to `cwd` when there is one and asking it to
/// read `prompt_file` when there is one.
///
/// Arguments are single-quoted, which PowerShell, bash, zsh and fish all read literally, so a path
/// containing a single quote is left out: Claude Code then starts wherever the new tab opened, or
/// without the prompt.
pub(crate) fn claude_code_command(cwd: Option<&Path>, prompt_file: Option<&Path>) -> String {
    let mut command = String::new();
    if let Some(dir) = cwd.and_then(quotable) {
        command.push_str(&format!("cd '{dir}'; "));
    }
    command.push_str("claude");
    if let Some(file) = prompt_file.and_then(quotable) {
        command.push_str(&format!(
            " 'Lee el archivo {file}: tiene un comando de mi terminal y su salida. \
             Explícame en español qué ha pasado y, si algo ha fallado, arréglalo.'"
        ));
    }
    command
}

fn quotable(path: &Path) -> Option<&str> {
    path.to_str().filter(|path| !path.contains('\''))
}

/// The message for Claude Code about a `command` (usually one that failed) and its `output`.
pub(crate) fn fix_with_claude_prompt(command: &str, output: &str) -> String {
    let output = output.trim_end();
    let skip = output.chars().count().saturating_sub(MAX_OUTPUT_CHARS);
    let tail: String = output.chars().skip(skip).collect();
    let cut_note = if skip > 0 {
        "(Solo el final de la salida; el principio se ha recortado.)\n"
    } else {
        ""
    };
    format!(
        "Comando de mi terminal:\n\n```\n{}\n```\n\nSalida:\n{cut_note}\n```\n{tail}\n```\n",
        command.trim()
    )
}

/// The model used for local AI when Ollama has none that suits better. `ollama run` downloads it
/// the first time.
pub(crate) const DEFAULT_LOCAL_MODEL: &str = "qwen2.5-coder:3b";

/// Where to get Ollama when it is not installed.
pub(crate) const OLLAMA_DOWNLOAD_URL: &str = "https://ollama.com/download";

/// What `ollama list` says about the models on this computer.
pub(crate) enum LocalModels {
    /// Ollama is not installed.
    NotInstalled,
    /// Ollama is installed; these are its models, possibly none.
    Installed(Vec<String>),
}

/// Asks Ollama which models it has.
pub(crate) fn local_models() -> LocalModels {
    let mut command = std::process::Command::new("ollama");
    command.arg("list");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        // Without this a console window flashes while `ollama list` runs.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    match command.output() {
        Ok(output) => {
            LocalModels::Installed(parse_ollama_list(&String::from_utf8_lossy(&output.stdout)))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => LocalModels::NotInstalled,
        // Ollama could not be started for another reason; `ollama run` will show why.
        Err(_) => LocalModels::Installed(Vec::new()),
    }
}

/// The model names in the output of `ollama list`, in its order.
pub(crate) fn parse_ollama_list(list: &str) -> Vec<String> {
    list.lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// The best model for code among `models`: the first one made for code, otherwise the first one,
/// otherwise [`DEFAULT_LOCAL_MODEL`].
pub(crate) fn pick_local_model(models: &[String]) -> String {
    const CODE_MODELS: [&str; 5] = ["coder", "codellama", "codestral", "codegemma", "starcoder"];
    models
        .iter()
        .find(|model| CODE_MODELS.iter().any(|hint| model.contains(hint)))
        .or_else(|| models.first())
        .cloned()
        .unwrap_or_else(|| DEFAULT_LOCAL_MODEL.to_owned())
}

/// The command that starts the local AI (Ollama) with `model`, first moving to `cwd`, and hands it
/// the contents of `prompt_file` when there is one. Without a prompt it opens a chat.
///
/// PowerShell has no `<` redirection, so it pipes the file instead, in UTF-8 so accents survive.
pub(crate) fn local_ai_command(
    cwd: Option<&Path>,
    prompt_file: Option<&Path>,
    model: &str,
    is_powershell: bool,
) -> String {
    let mut command = String::new();
    if let Some(dir) = cwd.and_then(quotable) {
        command.push_str(&format!("cd '{dir}'; "));
    }
    let model = model.replace('\'', "");
    match prompt_file.and_then(quotable) {
        Some(file) if is_powershell => command.push_str(&format!(
            "$OutputEncoding = [Text.UTF8Encoding]::new($false); \
             Get-Content -Raw -Encoding UTF8 '{file}' | ollama run '{model}'"
        )),
        Some(file) => command.push_str(&format!("ollama run '{model}' < '{file}'")),
        None => command.push_str(&format!("ollama run '{model}'")),
    }
    command
}

/// The message for the local AI about a `command` (usually one that failed) and its `output`.
/// Small models need the task spelled out, so it goes first.
pub(crate) fn explain_with_local_ai_prompt(command: &str, output: &str) -> String {
    format!(
        "Eres un experto en terminales y programación. Explica en español, de forma breve y \
         clara, qué ha pasado con este comando. Si ha fallado, di por qué y qué hay que hacer \
         para arreglarlo.\n\n{}",
        fix_with_claude_prompt(command, output)
    )
}

/// Saves `prompt` in a new file in the temp folder and returns its path.
pub(crate) fn write_prompt_file(prompt: &str) -> io::Result<PathBuf> {
    let dir = std::env::temp_dir().join("brasa");
    std::fs::create_dir_all(&dir)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    let file = dir.join(format!("claude-{stamp}.md"));
    std::fs::write(&file, prompt)?;
    Ok(file)
}

#[cfg(test)]
#[path = "claude_code_tests.rs"]
mod tests;
