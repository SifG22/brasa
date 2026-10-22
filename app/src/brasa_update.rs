//! Brasa's automatic updates on Windows. At most once a day, a few seconds after start, a worker
//! thread reads [`MANIFEST_URL`]; when it names a newer version than this build it asks whether to
//! install it. On yes it downloads that architecture's installer, checks its SHA-256 and starts it;
//! the installer closes Brasa, updates it in place and reopens it.
//!
//! The manifest, attached to every GitHub release next to its installers by the release workflow:
//!
//! ```json
//! {"version": "0.0.2", "notes": "…",
//!  "x64": {"url": "https://…/Brasa-Setup-0.0.2-x64.exe", "sha256": "…"},
//!  "arm64": {"url": "https://…/Brasa-Setup-0.0.2-arm64.exe", "sha256": "…"}}
//! ```
//!
//! Downloading and hashing use Windows' own `curl.exe` and `certutil.exe` (by full path, started
//! without a console window), so no network client is involved. Every failure is logged and
//! ignored: an update check must never get in the way. `BRASA_NO_UPDATE=1` turns it off.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use command::blocking::Command;
use serde_json::Value;

/// Where the latest version is announced: GitHub serves `releases/latest/download/<file>` from the
/// newest release.
const MANIFEST_URL: &str = "https://github.com/SifG22/brasa/releases/latest/download/latest.json";

const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// Wait after start, so the question never interrupts the window opening.
const START_DELAY: Duration = Duration::from_secs(10);

const INSTALLER_PREFIX: &str = "Brasa-Setup-";

/// This build's version (`GIT_RELEASE_TAG` without the leading `v`), or `None` for local builds.
fn current_version() -> Option<&'static str> {
    option_env!("GIT_RELEASE_TAG").map(|tag| tag.trim_start_matches('v'))
}

/// Starts the background check on Windows release builds that carry a version.
pub fn spawn(state_dir: PathBuf) {
    let Some(version) = current_version() else {
        return;
    };
    if !cfg!(windows) || cfg!(debug_assertions) || std::env::var_os("BRASA_NO_UPDATE").is_some() {
        return;
    }
    let spawned = std::thread::Builder::new()
        .name("brasa-update".into())
        .spawn(move || {
            std::thread::sleep(START_DELAY);
            remove_old_installers(&std::env::temp_dir(), version);
            if let Err(err) = check(&state_dir, version) {
                log::info!("Update check: {err}");
            }
        });
    if let Err(err) = spawned {
        log::warn!("Update check not started: {err}");
    }
}

fn state_path(dir: &Path) -> PathBuf {
    dir.join("brasa-update.json")
}

fn read_state(dir: &Path) -> Value {
    std::fs::read_to_string(state_path(dir))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null)
}

fn write_state(dir: &Path, state: &Value) {
    let path = state_path(dir);
    if let Err(err) =
        std::fs::create_dir_all(dir).and_then(|_| std::fs::write(&path, state.to_string()))
    {
        log::info!("Update state not saved to {}: {err}", path.display());
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

/// The question shown to the user.
fn question(current: &str, latest: &str, notes: &str) -> String {
    let mut text = format!("A new version of Brasa is available: {latest} (you have {current}).");
    if !notes.is_empty() {
        text.push_str("\n\n");
        text.push_str(notes);
    }
    text.push_str("\n\nDownload and install it now? Brasa will close to update and reopen.");
    text
}

fn check(state_dir: &Path, current: &str) -> Result<(), String> {
    let mut state = read_state(state_dir);
    let last = state.get("lastCheck").and_then(Value::as_u64).unwrap_or(0);
    if now_secs().saturating_sub(last) < CHECK_EVERY.as_secs() {
        return Ok(());
    }
    let skipped = state
        .get("skipped")
        .and_then(Value::as_str)
        .map(str::to_owned);
    state = serde_json::json!({"lastCheck": now_secs(), "skipped": skipped});
    write_state(state_dir, &state);

    let body = curl(current, &["--max-time", "20", MANIFEST_URL])?;
    let body = body
        .strip_prefix([0xEF, 0xBB, 0xBF].as_slice())
        .unwrap_or(&body);
    let manifest: Value =
        serde_json::from_slice(body).map_err(|err| format!("bad manifest: {err}"))?;
    let latest = manifest
        .get("version")
        .and_then(Value::as_str)
        .ok_or("manifest has no version")?;
    if !is_newer(latest, current) || skipped.as_deref() == Some(latest) {
        return Ok(());
    }
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    let asset = manifest
        .get(arch)
        .ok_or_else(|| format!("no {arch} installer in the manifest"))?;
    let url = asset
        .get("url")
        .and_then(Value::as_str)
        .filter(|url| url.starts_with("https://"))
        .ok_or("installer url missing or not https")?;
    let sha = asset
        .get("sha256")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
        .filter(|sha| sha.len() == 64)
        .ok_or("installer sha256 missing")?;
    let notes: String = manifest
        .get("notes")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .chars()
        .take(600)
        .collect();

    if !ask_yes_no("Update Brasa", &question(current, latest, &notes)) {
        if let Some(fields) = state.as_object_mut() {
            fields.insert("skipped".into(), Value::String(latest.to_owned()));
        }
        write_state(state_dir, &state);
        return Ok(());
    }

    let file = std::env::temp_dir().join(installer_name(latest, arch));
    let path = file.to_str().ok_or("temp path is not UTF-8")?;
    let downloaded = curl(current, &["--max-time", "900", "-o", path, url])
        .and_then(|_| sha256(&file))
        .and_then(|got| {
            if got == sha {
                Ok(())
            } else {
                Err(format!(
                    "installer checksum mismatch: expected {sha}, got {got}"
                ))
            }
        });
    if let Err(err) = downloaded {
        let _ = std::fs::remove_file(&file);
        return Err(err);
    }
    Command::new(&file)
        .args([
            "/SILENT",
            "/NORESTART",
            "/CLOSEAPPLICATIONS",
            "/RESTARTAPPLICATIONS",
        ])
        .spawn()
        .map_err(|err| format!("could not start the installer: {err}"))?;
    Ok(())
}

fn installer_name(version: &str, arch: &str) -> String {
    format!("{INSTALLER_PREFIX}{}-{arch}.exe", sanitize(version))
}

/// Deletes installers this updater downloaded into `temp` for `current` or older versions. One
/// still running (Windows locks it) stays and goes on a later start.
fn remove_old_installers(temp: &Path, current: &str) {
    let Ok(entries) = std::fs::read_dir(temp) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(version) = name.to_str().and_then(installer_version) else {
            continue;
        };
        if !is_newer(version, current)
            && let Err(err) = std::fs::remove_file(entry.path())
        {
            log::info!(
                "Old installer {} not removed yet: {err}",
                entry.path().display()
            );
        }
    }
}

/// The version in a name made by [`installer_name`], or `None` for any other file.
fn installer_version(name: &str) -> Option<&str> {
    let rest = name.strip_prefix(INSTALLER_PREFIX)?;
    let rest = rest
        .strip_suffix("-x64.exe")
        .or_else(|| rest.strip_suffix("-arm64.exe"))?;
    (!rest.is_empty()
        && rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'))
    .then_some(rest)
}

/// Whether `candidate` (`a.b.c`) is newer than `current`. Missing or non-numeric parts count as 0.
fn is_newer(candidate: &str, current: &str) -> bool {
    let parts = |version: &str| -> Vec<u64> {
        version
            .trim()
            .trim_start_matches('v')
            .split(['.', '-'])
            .take(3)
            .map(|part| part.parse().unwrap_or(0))
            .collect()
    };
    let (a, b) = (parts(candidate), parts(current));
    (0..3)
        .map(|i| {
            (
                a.get(i).copied().unwrap_or(0),
                b.get(i).copied().unwrap_or(0),
            )
        })
        .find(|(x, y)| x != y)
        .is_some_and(|(x, y)| x > y)
}

fn sanitize(version: &str) -> String {
    version
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
        .take(32)
        .collect()
}

/// `exe` from System32 (started without a console window, like every [`Command`]).
fn system_tool(exe: &str) -> Command {
    let root =
        std::env::var_os("SystemRoot").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    Command::new(root.join("System32").join(exe))
}

/// Runs Windows' curl (`-fsSL`, so HTTP errors fail) and returns what it printed.
fn curl(current: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let agent = format!("Brasa/{current}");
    let output = system_tool("curl.exe")
        .args(["-fsSL", "-A", agent.as_str()])
        .args(args)
        .output()
        .map_err(|err| format!("curl: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "curl failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

/// SHA-256 of `file` (lower-case hex) through `certutil -hashfile`.
fn sha256(file: &Path) -> Result<String, String> {
    let output = system_tool("certutil.exe")
        .arg("-hashfile")
        .arg(file)
        .arg("SHA256")
        .output()
        .map_err(|err| format!("certutil: {err}"))?;
    if !output.status.success() {
        return Err(format!("certutil failed ({})", output.status));
    }
    parse_certutil(&String::from_utf8_lossy(&output.stdout))
        .ok_or_else(|| "no hash in certutil's output".to_owned())
}

/// The hash line of `certutil -hashfile` (older Windows separates the bytes with spaces).
fn parse_certutil(text: &str) -> Option<String> {
    text.lines()
        .map(|line| line.replace(' ', "").to_ascii_lowercase())
        .find(|line| line.len() == 64 && line.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(windows)]
fn ask_yes_no(title: &str, text: &str) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        IDYES, MB_ICONINFORMATION, MB_SETFOREGROUND, MB_YESNO, MessageBoxW,
    };
    use windows::core::HSTRING;

    // SAFETY: both strings outlive the call and no owner window is passed.
    let answer = unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(text),
            &HSTRING::from(title),
            MB_YESNO | MB_ICONINFORMATION | MB_SETFOREGROUND,
        )
    };
    answer == IDYES
}

#[cfg(not(windows))]
fn ask_yes_no(_title: &str, _text: &str) -> bool {
    false
}

#[cfg(test)]
#[path = "brasa_update_tests.rs"]
mod tests;
