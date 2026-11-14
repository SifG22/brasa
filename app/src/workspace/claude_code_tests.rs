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

#[test]
fn reads_model_names_from_ollama_list() {
    let list = "NAME                ID              SIZE      MODIFIED\nllama3.2:latest     a80c4f17acd5    2.0 GB    3 days ago\nqwen2.5-coder:3b    e7149271c296    1.9 GB    5 weeks ago\n";
    let models = super::parse_ollama_list(list);
    assert_eq!(models, vec!["llama3.2:latest", "qwen2.5-coder:3b"]);
    assert_eq!(super::pick_local_model(&models), "qwen2.5-coder:3b");
    assert_eq!(super::pick_local_model(&models[..1]), "llama3.2:latest");
    assert_eq!(super::pick_local_model(&[]), super::DEFAULT_LOCAL_MODEL);
}

#[test]
fn hands_the_prompt_file_to_ollama_in_each_shell() {
    let file = Path::new(r"C:\Temp\brasa\claude-1.md");
    assert_eq!(
        super::local_ai_command(
            Some(Path::new(r"C:\src")),
            Some(file),
            "qwen2.5-coder:3b",
            false
        ),
        r"cd 'C:\src'; ollama run 'qwen2.5-coder:3b' < 'C:\Temp\brasa\claude-1.md'"
    );
    let powershell = super::local_ai_command(None, Some(file), "qwen2.5-coder:3b", true);
    assert!(powershell.ends_with(
        r"Get-Content -Raw -Encoding UTF8 'C:\Temp\brasa\claude-1.md' | ollama run 'qwen2.5-coder:3b'"
    ));
    assert_eq!(
        super::local_ai_command(None, None, "llama3.2", true),
        "ollama run 'llama3.2'"
    );
}
