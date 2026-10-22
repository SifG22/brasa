//! OSS-channel `brasa-tui` binary and `default-run` target.
//!
//! This is what bare `cargo run -p brasa_tui` builds, so it hand-builds a
//! production config and needs no internal `brasa-channel-config` generator
//! (mirrors `app/src/bin/oss.rs`). It is a console application (no GUI window,
//! no app bundle), so unlike the GUI binaries it sets no `windows_subsystem`
//! attribute and embeds no `Info.plist`.

use anyhow::Result;
use brasa_core::AppId;
use brasa_core::channel::{BrasaServerConfig, Channel, ChannelConfig, ChannelState, OzConfig};

fn main() -> Result<()> {
    let mut state = ChannelState::new(
        Channel::Oss,
        ChannelConfig {
            app_id: AppId::new("dev", "brasa", "BrasaTui"),
            logfile_name: "brasa-tui.log".into(),
            server_config: BrasaServerConfig::production(),
            oz_config: OzConfig::production(),
            telemetry_config: None,
            crash_reporting_config: None,
            autoupdate_config: None,
            mcp_static_config: None,
        },
    );
    if cfg!(debug_assertions) {
        state = state.with_additional_features(brasa_core::features::DEBUG_FLAGS);
    }
    ChannelState::set(state);

    brasa_tui::run()
}
