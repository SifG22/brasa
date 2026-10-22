use brasa_completer::completer::{CommandExitStatus, CommandOutput};

use super::*;

#[test]
fn test_prompt_chip_log_filename_uses_channel_logfile_stem() {
    assert_eq!(
        prompt_chip_log_filename("brasa_dev.log"),
        "brasa_dev.prompt_chips.log"
    );
    assert_eq!(
        prompt_chip_log_filename("brasa_local"),
        "brasa_local.prompt_chips.log"
    );
}

#[test]
fn prompt_chip_log_path_stays_beside_resolved_frontend_log() {
    for (active_log, expected_sidecar) in [
        (
            "/tmp/brasa-logs/brasa_dev.log",
            "/tmp/brasa-logs/brasa_dev.prompt_chips.log",
        ),
        (
            "/tmp/brasa-logs/brasa-cli/brasa_preview.log",
            "/tmp/brasa-logs/brasa-cli/brasa_preview.prompt_chips.log",
        ),
        (
            "/tmp/brasa-logs/oz/brasa.log",
            "/tmp/brasa-logs/oz/brasa.prompt_chips.log",
        ),
    ] {
        assert_eq!(
            prompt_chip_log_file_path(std::path::Path::new(active_log)).unwrap(),
            std::path::PathBuf::from(expected_sidecar)
        );
    }
}

#[test]
fn test_format_log_entry_uses_explicit_empty_and_missing_markers() {
    let entry = format_log_entry(&ChipCommandLogEntry {
        chip_kind: &ContextChipKind::GithubPullRequest,
        chip_title: "GitHub Pull Request",
        phase: PromptChipExecutionPhase::Value,
        shell_type: ShellType::Zsh,
        working_directory: None,
        command: "",
        output: None,
        timed_out: true,
    });

    assert!(entry.contains("status: timed_out"));
    assert!(entry.contains("working_directory: <none>"));
    assert!(entry.contains("exit_code: <none>"));
    assert!(entry.contains("command:\n<<<COMMAND\n<empty>\n>>>COMMAND"));
    assert!(entry.contains("stdout:\n<<<STDOUT\n<empty>\n>>>STDOUT"));
    assert!(entry.contains("stderr:\n<<<STDERR\n<empty>\n>>>STDERR"));
}

#[test]
fn test_format_log_entry_preserves_stdout_and_stderr_sections() {
    let output = CommandOutput {
        stdout: b"https://github.com/SifG22/brasa\n".to_vec(),
        stderr: b"warning output\n".to_vec(),
        status: CommandExitStatus::Success,
        exit_code: Some(brasa_core::command::ExitCode::from(0)),
    };

    let entry = format_log_entry(&ChipCommandLogEntry {
        chip_kind: &ContextChipKind::GithubPullRequest,
        chip_title: "GitHub Pull Request",
        phase: PromptChipExecutionPhase::OnClick,
        shell_type: ShellType::Zsh,
        working_directory: Some("/tmp/project"),
        command: "gh pr view --json url --jq .url",
        output: Some(&output),
        timed_out: false,
    });

    assert!(entry.contains("phase: on_click"));
    assert!(entry.contains("status: success"));
    assert!(entry.contains("working_directory: /tmp/project"));
    assert!(entry.contains("command:\n<<<COMMAND\ngh pr view --json url --jq .url\n>>>COMMAND"));
    assert!(entry.contains("stdout:\n<<<STDOUT\nhttps://github.com/SifG22/brasa\n>>>STDOUT"));
    assert!(entry.contains("stderr:\n<<<STDERR\nwarning output\n>>>STDERR"));
}
