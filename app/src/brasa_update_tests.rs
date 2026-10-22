use super::{
    MANIFEST_URL, installer_name, installer_version, is_newer, parse_certutil, question,
    remove_old_installers, sanitize,
};

#[test]
fn versions_compare_numerically() {
    assert!(is_newer("0.0.2", "0.0.1"));
    assert!(is_newer("0.1.0", "0.0.9"));
    assert!(is_newer("v1.0.0", "0.9.9"));
    assert!(is_newer("0.0.10", "0.0.9"));
    assert!(!is_newer("0.0.1", "0.0.1"));
    assert!(!is_newer("0.0.1", "0.0.2"));
    assert!(!is_newer("nonsense", "0.0.1"));
    assert!(!is_newer("", ""));
}

#[test]
fn certutil_output_parses_with_or_without_spaces() {
    let hex = "e34e8e0c16119c453cd7b11a88aeda8e9fc0cf6d389f54ca56b0dbaadb9557f6";
    let output = format!(
        "SHA256 hash of x.exe:\r\n{hex}\r\nCertUtil: -hashfile command completed successfully.\r\n"
    );
    assert_eq!(parse_certutil(&output).as_deref(), Some(hex));
    let spaced = hex
        .as_bytes()
        .chunks(2)
        .map(|pair| String::from_utf8_lossy(pair).to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(
        parse_certutil(&format!("SHA256 hash:\n{spaced}\n")).as_deref(),
        Some(hex)
    );
    assert_eq!(parse_certutil("CertUtil: error"), None);
}

#[test]
fn only_our_installers_up_to_this_version_are_removed() -> std::io::Result<()> {
    assert_eq!(
        installer_version(&installer_name("0.0.2", "x64")),
        Some("0.0.2")
    );
    assert_eq!(
        installer_version("Brasa-Setup-1.2.3-arm64.exe"),
        Some("1.2.3")
    );
    assert_eq!(installer_version("Brasa-Setup--x64.exe"), None);
    assert_eq!(installer_version("Brasa-Setup-0.0.2-x64.exe.part"), None);
    assert_eq!(installer_version("Other-Setup-0.0.2-x64.exe"), None);

    let dir = tempfile::tempdir()?;
    let old = dir.path().join(installer_name("0.0.0", "x64"));
    let current = dir.path().join(installer_name("0.0.1", "arm64"));
    let newer = dir.path().join(installer_name("999.0.0", "x64"));
    let other = dir.path().join("notes.txt");
    for file in [&old, &current, &newer, &other] {
        std::fs::write(file, b"x")?;
    }
    remove_old_installers(dir.path(), "0.0.1");
    assert!(!old.exists() && !current.exists(), "installed versions go");
    assert!(
        newer.exists() && other.exists(),
        "a pending update and other files stay"
    );
    Ok(())
}

#[test]
fn file_names_keep_only_safe_characters() {
    assert_eq!(sanitize("0.0.2"), "0.0.2");
    assert_eq!(sanitize("../../evil\\name"), "....evilname");
}

#[test]
fn the_question_names_both_versions() {
    let text = question("0.0.1", "0.0.2", "Notes");
    assert!(text.contains("0.0.2") && text.contains("you have 0.0.1") && text.contains("Notes"));
    assert!(MANIFEST_URL.starts_with("https://github.com/SifG22/brasa/releases/latest/download/"));
}
