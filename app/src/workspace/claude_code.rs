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
            " 'Lee el archivo {file}: tiene un comando que ha fallado en mi terminal y su salida. \
             Explícame en español qué ha pasado y arréglalo.'"
        ));
    }
    command
}

fn quotable(path: &Path) -> Option<&str> {
    path.to_str().filter(|path| !path.contains('\''))
}

/// The message for Claude Code about a failed `command` and its `output`.
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
        "Este comando ha fallado en mi terminal:\n\n```\n{}\n```\n\nSalida:\n{cut_note}\n```\n{tail}\n```\n",
        command.trim()
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
