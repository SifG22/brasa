use brasa_core::features::FeatureFlag;
use settings::macros::define_settings_group;
use settings::{RespectUserSyncSetting, SupportedPlatforms, SyncToCloud};

use super::DriveSortOrder;

pub const HAS_AUTO_OPENED_WELCOME_FOLDER: &str = "HasAutoOpenedWelcomeFolder";

define_settings_group!(BrasaDriveSettings, settings: [
    sorting_choice: BrasaDriveSortingChoice {
        type: DriveSortOrder,
        default: DriveSortOrder::ByObjectType,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "brasa_drive.sorting_choice",
        description: "The sort order for items in Brasa Drive.",
    },
    sharing_onboarding_block_shown: BrasaDriveSharingOnboardingBlockShown {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    },
    // Controls whether Brasa Drive appears in the tools panel, command palette, and command search.
    enable_brasa_drive: EnableBrasaDrive {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "brasa_drive.enabled",
        description: "Whether Brasa Drive is enabled.",
    },
]);

impl BrasaDriveSettings {
    /// Returns whether Brasa Drive is available for the current auth state.
    ///
    /// This is intentionally separate from the stored `enable_brasa_drive`
    /// preference. Logged-out and anonymous users can retain their onboarding
    /// preference so Brasa Drive appears automatically after signup, while the
    /// feature remains unavailable until then.
    pub fn is_brasa_drive_available(app: &brasaui::AppContext) -> bool {
        use brasaui::SingletonEntity as _;
        !FeatureFlag::SkipFirebaseAnonymousUser.is_enabled()
            || !crate::auth::AuthStateProvider::as_ref(app)
                .get()
                .is_anonymous_or_logged_out()
    }
    /// Returns whether Brasa Drive should be considered enabled.
    /// Returns `false` when the user is anonymous or fully logged out,
    /// regardless of the user setting.
    pub fn is_brasa_drive_enabled(app: &brasaui::AppContext) -> bool {
        use brasaui::SingletonEntity as _;
        *Self::as_ref(app).enable_brasa_drive && Self::is_brasa_drive_available(app)
    }
}
