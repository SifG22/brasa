use std::path::Path;

use super::{MAX_OUTPUT_CHARS, claude_code_command, fix_with_claude_prompt};

#[test]
fn moves_to_the_directory_before_starting_claude() {
    assert_eq!(
        claude_code_command(Some(Path::new(r"C:\Users\rodri\mi proyecto")), None),
        r"cd 'C:\Users\rodri\mi proyecto'; claude"
    );
}

#[test]
fn starts_claude_in_place_without_a_usable_directory() {
    assert_eq!(claude_code_command(None, None), "claude");
    assert_eq!(
        claude_code_command(Some(Path::new("/tmp/it's")), None),
        "claude"
    );
}

#[test]
fn points_claude_at_the_prompt_file() {
    let command = claude_code_command(None, Some(Path::new(r"C:\Temp\brasa\claude-1.md")));
    assert!(command.starts_with(r"claude 'Lee el archivo C:\Temp\brasa\claude-1.md:"));
    assert!(command.ends_with("arréglalo.'"));
    assert_eq!(
        claude_code_command(None, Some(Path::new("/tmp/it's.md"))),
        "claude"
    );
}

#[test]
fn the_prompt_keeps_the_command_and_the_end_of_the_output() {
    let prompt = fix_with_claude_prompt(" cargo build \n", "error[E0308]: mismatched types\n");
    assert!(prompt.contains("```\ncargo build\n```"));
    assert!(prompt.contains("error[E0308]: mismatched types\n```"));
    assert!(!prompt.contains("recortado"));

    let long = format!("{}FIN", "x".repeat(MAX_OUTPUT_CHARS + 10));
    let prompt = fix_with_claude_prompt("make", &long);
    assert!(prompt.contains("recortado"));
    assert!(prompt.contains("FIN\n```"));
    assert!(prompt.chars().count() < MAX_OUTPUT_CHARS + 300);
}
