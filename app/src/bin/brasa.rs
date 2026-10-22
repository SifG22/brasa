// On Windows, never show a console window, not even in debug builds.
// See https://doc.rust-lang.org/reference/runtime.html#the-windows_subsystem-attribute.
#![windows_subsystem = "windows"]

use anyhow::Result;
use warp_core::AppId;
use warp_core::channel::{Channel, ChannelConfig, ChannelState, OzConfig, WarpServerConfig};

// Brasa: wrapper around warp::run() for the public build.
fn main() -> Result<()> {
    let state = ChannelState::new(
        Channel::Oss,
        ChannelConfig {
            app_id: AppId::new("net", "sifg", "Brasa"),
            logfile_name: "brasa.log".into(),
            server_config: WarpServerConfig::offline(),
            oz_config: OzConfig::offline(),
            telemetry_config: None,
            crash_reporting_config: None,
            autoupdate_config: None,
            mcp_static_config: None,
        },
    );
    ChannelState::set(state);

    warp::run()
}

// If we're not using an external plist, embed the following as the Info.plist.
#[cfg(all(not(feature = "extern_plist"), target_os = "macos"))]
embed_plist::embed_info_plist_bytes!(r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleDisplayName</key>
    <string>Brasa</string>
    <key>CFBundleExecutable</key>
    <string>brasa</string>
    <key>CFBundleIdentifier</key>
    <string>net.sifg.Brasa</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Brasa</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>UIDesignRequiresCompatibility</key>
    <true/>
    <key>CFBundleURLTypes</key>
    <array><dict><key>CFBundleURLName</key><string>Custom App</string><key>CFBundleURLSchemes</key><array><string>brasa</string></array></dict></array>
    <key>NSHumanReadableCopyright</key>
    <string>© 2026 SifG22. Based on Warp (AGPLv3)</string>
    </dict>
    </plist>
"#.as_bytes());
