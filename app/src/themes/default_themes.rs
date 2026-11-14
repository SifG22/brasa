use asset_macro::bundled_or_fetched_asset;
use brasa_core::ui::color::blend::Blend;
use brasa_core::ui::color::coloru_with_opacity;
use brasa_core::ui::theme::color::CustomDetails;
use brasa_core::ui::theme::{
    AnsiColor, AnsiColors, BrasaTheme, Details, Fill, HorizontalGradient, Image, TerminalColors,
    VerticalGradient,
};
use pathfinder_color::ColorU;

const DARK_MODE_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x3B3330FF),
    AnsiColor::from_u32(0xF2655AFF),
    AnsiColor::from_u32(0x9BD67AFF),
    AnsiColor::from_u32(0xF5C26BFF),
    AnsiColor::from_u32(0x7FB4E6FF),
    AnsiColor::from_u32(0xD79AE0FF),
    AnsiColor::from_u32(0x7CD3C6FF),
    AnsiColor::from_u32(0xE8DED8FF),
);
const DARK_MODE_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x6E625CFF),
    AnsiColor::from_u32(0xFF8A80FF),
    AnsiColor::from_u32(0xB8EB98FF),
    AnsiColor::from_u32(0xFFD98EFF),
    AnsiColor::from_u32(0xA3CDF5FF),
    AnsiColor::from_u32(0xEBB6F2FF),
    AnsiColor::from_u32(0xA0E6DBFF),
    AnsiColor::from_u32(0xFFF8F3FF),
);

const LIGHT_MODE_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x2A2320FF),
    AnsiColor::from_u32(0xC62F2FFF),
    AnsiColor::from_u32(0x3F7D1FFF),
    AnsiColor::from_u32(0xA16207FF),
    AnsiColor::from_u32(0x1D5FA8FF),
    AnsiColor::from_u32(0x8E3FA0FF),
    AnsiColor::from_u32(0x0F7C75FF),
    AnsiColor::from_u32(0xD8CEC6FF),
);
const LIGHT_MODE_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x5C514BFF),
    AnsiColor::from_u32(0xE0443CFF),
    AnsiColor::from_u32(0x4F9A2AFF),
    AnsiColor::from_u32(0xB7791FFF),
    AnsiColor::from_u32(0x2F74C0FF),
    AnsiColor::from_u32(0xA855B8FF),
    AnsiColor::from_u32(0x14958CFF),
    AnsiColor::from_u32(0xF3EDE8FF),
);

const SOLARIZED_DARK_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x073642FF),
    AnsiColor::from_u32(0xDC322FFF),
    AnsiColor::from_u32(0x859900FF),
    AnsiColor::from_u32(0xB58900FF),
    AnsiColor::from_u32(0x268BD2FF),
    AnsiColor::from_u32(0xD33682FF),
    AnsiColor::from_u32(0x2AA198FF),
    AnsiColor::from_u32(0xEEE8D5FF),
);
const SOLARIZED_DARK_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x002B36FF),
    AnsiColor::from_u32(0xCB4B16FF),
    AnsiColor::from_u32(0x586E75FF),
    AnsiColor::from_u32(0x657B83FF),
    AnsiColor::from_u32(0x839496FF),
    AnsiColor::from_u32(0x6C71C4FF),
    AnsiColor::from_u32(0x93A1A1FF),
    AnsiColor::from_u32(0xFDF6E3FF),
);

const SOLARIZED_LIGHT_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x073642FF),
    AnsiColor::from_u32(0xDC322FFF),
    AnsiColor::from_u32(0x859900FF),
    AnsiColor::from_u32(0xB58900FF),
    AnsiColor::from_u32(0x268BD2FF),
    AnsiColor::from_u32(0xD33682FF),
    AnsiColor::from_u32(0x2AA198FF),
    AnsiColor::from_u32(0xEEE8D5FF),
);
const SOLARIZED_LIGHT_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x002B36FF),
    AnsiColor::from_u32(0xCB4B16FF),
    AnsiColor::from_u32(0x586E75FF),
    AnsiColor::from_u32(0x657B83FF),
    AnsiColor::from_u32(0x839496FF),
    AnsiColor::from_u32(0x6C71C4FF),
    AnsiColor::from_u32(0x93A1A1FF),
    AnsiColor::from_u32(0xFDF6E3FF),
);

const DRACULA_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x000000FF),
    AnsiColor::from_u32(0xFF5555FF),
    AnsiColor::from_u32(0x50FA7BFF),
    AnsiColor::from_u32(0xF1FA8CFF),
    AnsiColor::from_u32(0xBD93F9FF),
    AnsiColor::from_u32(0xFF79C6FF),
    AnsiColor::from_u32(0x8BE9FDFF),
    AnsiColor::from_u32(0xBBBBBBFF),
);
const DRACULA_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x555555FF),
    AnsiColor::from_u32(0xFF5555FF),
    AnsiColor::from_u32(0x50FA7BFF),
    AnsiColor::from_u32(0xF1FA8CFF),
    AnsiColor::from_u32(0xCAA9FAFF),
    AnsiColor::from_u32(0xFF79C6FF),
    AnsiColor::from_u32(0x8BE9FDFF),
    AnsiColor::from_u32(0xFFFFFFFF),
);

const PHENOMENON_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x121212FF),
    AnsiColor::from_u32(0xD22D1EFF),
    AnsiColor::from_u32(0x1CA05AFF),
    AnsiColor::from_u32(0xE5A01AFF),
    AnsiColor::from_u32(0x3780E9FF),
    AnsiColor::from_u32(0xBF409DFF),
    AnsiColor::from_u32(0x799C92FF),
    AnsiColor::from_u32(0xFAF9F6FF),
);
const PHENOMENON_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x292929FF),
    AnsiColor::from_u32(0xAE756FFF),
    AnsiColor::from_u32(0x789B88FF),
    AnsiColor::from_u32(0xBD9F65FF),
    AnsiColor::from_u32(0x6F839FFF),
    AnsiColor::from_u32(0xA57899FF),
    AnsiColor::from_u32(0xBFC5C3FF),
    AnsiColor::from_u32(0xFFFFFFFF),
);

const GRUVBOX_DARK_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x282828FF),
    AnsiColor::from_u32(0xCC241DFF),
    AnsiColor::from_u32(0x98971AFF),
    AnsiColor::from_u32(0xD79921FF),
    AnsiColor::from_u32(0x458588FF),
    AnsiColor::from_u32(0xB16286FF),
    AnsiColor::from_u32(0x689D6AFF),
    AnsiColor::from_u32(0xA89984FF),
);
const GRUVBOX_DARK_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x928374FF),
    AnsiColor::from_u32(0xFB4934FF),
    AnsiColor::from_u32(0xB8BB26FF),
    AnsiColor::from_u32(0xFABD2FFF),
    AnsiColor::from_u32(0x83A598FF),
    AnsiColor::from_u32(0xD3869BFF),
    AnsiColor::from_u32(0x8EC07CFF),
    AnsiColor::from_u32(0xEBDBB2FF),
);

const GRUVBOX_LIGHT_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0xFBF1C7FF),
    AnsiColor::from_u32(0xCC241DFF),
    AnsiColor::from_u32(0x98971AFF),
    AnsiColor::from_u32(0xD79921FF),
    AnsiColor::from_u32(0x458588FF),
    AnsiColor::from_u32(0xB16286FF),
    AnsiColor::from_u32(0x689D6AFF),
    AnsiColor::from_u32(0x7C6F64FF),
);
const GRUVBOX_LIGHT_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x928374FF),
    AnsiColor::from_u32(0x9D0006FF),
    AnsiColor::from_u32(0x79740EFF),
    AnsiColor::from_u32(0xB57614FF),
    AnsiColor::from_u32(0x076678FF),
    AnsiColor::from_u32(0x8F3F71FF),
    AnsiColor::from_u32(0x427B58FF),
    AnsiColor::from_u32(0x3C3836FF),
);

const SOLARFLARE_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x2E333DFF),
    AnsiColor::from_u32(0xD66060FF),
    AnsiColor::from_u32(0x64AF86FF),
    AnsiColor::from_u32(0xCAA358FF),
    AnsiColor::from_u32(0x5C80B2FF),
    AnsiColor::from_u32(0xB766A1FF),
    AnsiColor::from_u32(0x8069A1FF),
    AnsiColor::from_u32(0xF0F4F7FF),
);
const SOLARFLARE_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x37404AFF),
    AnsiColor::from_u32(0xEB8282FF),
    AnsiColor::from_u32(0x64AF86FF),
    AnsiColor::from_u32(0xCAA358FF),
    AnsiColor::from_u32(0x5C80B2FF),
    AnsiColor::from_u32(0xB766A1FF),
    AnsiColor::from_u32(0x8069A1FF),
    AnsiColor::from_u32(0xFFFFFFFF),
);

const ADEBERRY_NORMAL_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x121212FF),
    AnsiColor::from_u32(0xC76156FF),
    AnsiColor::from_u32(0x57C78AFF),
    AnsiColor::from_u32(0xC8A35AFF),
    AnsiColor::from_u32(0x5785C7FF),
    AnsiColor::from_u32(0xC756A9FF),
    AnsiColor::from_u32(0x57C7C3FF),
    AnsiColor::from_u32(0xEEEDEBFF),
);
const ADEBERRY_BRIGHT_COLORS: AnsiColors = AnsiColors::new(
    AnsiColor::from_u32(0x292929FF),
    AnsiColor::from_u32(0xD22D1EFF),
    AnsiColor::from_u32(0x1CA05AFF),
    AnsiColor::from_u32(0xE5A01AFF),
    AnsiColor::from_u32(0x1458B8FF),
    AnsiColor::from_u32(0xA43787FF),
    AnsiColor::from_u32(0x4D9989FF),
    AnsiColor::from_u32(0xFFFFFFFF),
);

pub(super) fn light_mode_colors() -> TerminalColors {
    TerminalColors::new(LIGHT_MODE_NORMAL_COLORS, LIGHT_MODE_BRIGHT_COLORS)
}

pub(super) fn dark_mode_colors() -> TerminalColors {
    TerminalColors::new(DARK_MODE_NORMAL_COLORS, DARK_MODE_BRIGHT_COLORS)
}

pub(super) fn solarized_light_colors() -> TerminalColors {
    TerminalColors::new(SOLARIZED_LIGHT_NORMAL_COLORS, SOLARIZED_LIGHT_BRIGHT_COLORS)
}

pub(super) fn solarized_dark_colors() -> TerminalColors {
    TerminalColors::new(SOLARIZED_DARK_NORMAL_COLORS, SOLARIZED_DARK_BRIGHT_COLORS)
}

pub(super) fn dracula_colors() -> TerminalColors {
    TerminalColors::new(DRACULA_NORMAL_COLORS, DRACULA_BRIGHT_COLORS)
}

pub(super) fn phenomenon_colors() -> TerminalColors {
    TerminalColors::new(PHENOMENON_NORMAL_COLORS, PHENOMENON_BRIGHT_COLORS)
}

pub(super) fn gruvbox_dark_colors() -> TerminalColors {
    TerminalColors::new(GRUVBOX_DARK_NORMAL_COLORS, GRUVBOX_DARK_BRIGHT_COLORS)
}

pub(super) fn gruvbox_light_colors() -> TerminalColors {
    TerminalColors::new(GRUVBOX_LIGHT_NORMAL_COLORS, GRUVBOX_LIGHT_BRIGHT_COLORS)
}

pub(super) fn solarflare_colors() -> TerminalColors {
    TerminalColors::new(SOLARFLARE_NORMAL_COLORS, SOLARFLARE_BRIGHT_COLORS)
}

pub(super) fn adeberry_colors() -> TerminalColors {
    TerminalColors::new(ADEBERRY_NORMAL_COLORS, ADEBERRY_BRIGHT_COLORS)
}

/// Default bundled themes
pub fn dark_theme() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x141110FF)),
        ColorU::from_u32(0xEFE6E1FF),
        Fill::Solid(ColorU::from_u32(0xF97316FF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        None,
        Some("Brasa Dark".to_string()),
    )
}

pub fn light_theme() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xFAF7F4FF)),
        ColorU::from_u32(0x1F1A17FF),
        Fill::Solid(ColorU::from_u32(0xEA580CFF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        None,
        Some("Brasa Light".to_string()),
    )
}

pub(super) fn dracula() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x282A36FF)),
        ColorU::from_u32(0xF8F8F2FF),
        Fill::Solid(ColorU::from_u32(0xFF79C6FF)),
        None,
        Some(Details::Darker),
        dracula_colors(),
        None,
        Some("Dracula".to_string()),
    )
}

pub(super) fn solarized_light() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xFDF6E3FF)),
        ColorU::from_u32(0x586E75FF),
        Fill::Solid(ColorU::from_u32(0x66B5A9FF)),
        None,
        Some(Details::Lighter),
        solarized_light_colors(),
        None,
        Some("Solarized Light".to_string()),
    )
}

pub(super) fn solarized_dark() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x002B36FF)),
        ColorU::from_u32(0xF8F8F2FF),
        Fill::Solid(ColorU::from_u32(0xCB4B16FF)),
        None,
        Some(Details::Darker),
        solarized_dark_colors(),
        None,
        Some("Solarized Dark".to_string()),
    )
}

pub(super) fn gruvbox_dark() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x282828FF)),
        ColorU::from_u32(0xEBDBB2FF),
        Fill::Solid(ColorU::from_u32(0xFC802DFF)),
        None,
        Some(Details::Darker),
        gruvbox_dark_colors(),
        None,
        Some("Gruvbox Dark".to_string()),
    )
}

pub(super) fn gruvbox_light() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xFBF1C7FF)),
        ColorU::from_u32(0x3C3836FF),
        Fill::Solid(ColorU::from_u32(0xAD3B14FF)),
        None,
        Some(Details::Lighter),
        gruvbox_light_colors(),
        None,
        Some("Gruvbox Light".to_string()),
    )
}

/// Bundled gradient themes
pub(super) fn cyber_wave() -> BrasaTheme {
    BrasaTheme::new(
        Fill::VerticalGradient(VerticalGradient::new(
            ColorU::black().blend(&coloru_with_opacity(ColorU::from_u32(0x00C2FFFF), 20)),
            ColorU::black(),
        )),
        ColorU::white(),
        Fill::HorizontalGradient(HorizontalGradient::new(
            ColorU::from_u32(0x007972FF),
            ColorU::from_u32(0x7B008FFF),
        )),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        None,
        Some("Cyber Wave".to_string()),
    )
}

pub(super) fn willow_dream() -> BrasaTheme {
    BrasaTheme::new(
        Fill::VerticalGradient(VerticalGradient::new(
            ColorU::from_u32(0x206169FF),
            ColorU::from_u32(0x022F27FF),
        )),
        ColorU::white(),
        Fill::HorizontalGradient(HorizontalGradient::new(
            ColorU::from_u32(0xF9AEA8FF),
            ColorU::from_u32(0xDD6258FF),
        )),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        None,
        Some("Willow Dream".to_string()),
    )
}

pub(super) fn fancy_dracula() -> BrasaTheme {
    BrasaTheme::new(
        Fill::VerticalGradient(VerticalGradient::new(
            ColorU::from_u32(0x252630FF),
            ColorU::from_u32(0x3D3F4FFF),
        )),
        ColorU::white(),
        Fill::HorizontalGradient(HorizontalGradient::new(
            ColorU::from_u32(0xBCA1F6FF),
            ColorU::from_u32(0xA3E7FCFF),
        )),
        None,
        Some(Details::Darker),
        dracula_colors(),
        None,
        Some("Fancy Dracula".to_string()),
    )
}

pub(super) fn phenomenon() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x121212FF)),
        ColorU::from_u32(0xFAF9F6FF),
        Fill::Solid(ColorU::from_u32(0x2E5D9EFF)),
        None,
        Some(Details::Darker),
        phenomenon_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/phenomenon_bg.jpg"),
            opacity: 100,
        }),
        Some("Phenomenon".to_string()),
    )
}

/// Bundled themes with background images
pub(super) fn jellyfish() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x1B1718FF)),
        ColorU::white(),
        Fill::Solid(ColorU::from_u32(0x538682FF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/jellyfish_bg.jpg"),
            opacity: 30,
        }),
        Some("Jellyfish".to_string()),
    )
}

pub(super) fn koi() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x211719FF)),
        ColorU::white(),
        Fill::Solid(ColorU::from_u32(0xFF3131FF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/koi_bg.jpg"),
            opacity: 30,
        }),
        Some("Koi".to_string()),
    )
}

pub(super) fn leafy() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::black()),
        ColorU::white(),
        Fill::Solid(ColorU::from_u32(0x55972DFF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/leafy_bg.jpg"),
            opacity: 30,
        }),
        Some("Leafy".to_string()),
    )
}

pub(super) fn marble() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xE3E3E3FF)),
        ColorU::black(),
        Fill::Solid(ColorU::from_u32(0x585858FF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/marble_bg.jpg"),
            opacity: 50,
        }),
        Some("Marble".to_string()),
    )
}

pub(super) fn pink_city() -> BrasaTheme {
    let details = CustomDetails {
        ..CustomDetails::lighter_details()
    };
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xFBEFF6FF)),
        ColorU::black(),
        Fill::Solid(ColorU::from_u32(0xE10087FF)),
        None,
        Some(Details::Custom(details)),
        light_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/pink_city_bg.jpg"),
            opacity: 40,
        }),
        Some("Pink City".to_string()),
    )
}

pub(super) fn snowy() -> BrasaTheme {
    BrasaTheme::new(
        Fill::VerticalGradient(VerticalGradient::new(
            ColorU::from_u32(0xFFFFFFFF),
            ColorU::from_u32(0xDEE6EBFF),
        )),
        ColorU::black(),
        Fill::Solid(ColorU::from_u32(0x647E90FF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/snowy_bg.jpg"),
            opacity: 20,
        }),
        Some("Snowy".to_string()),
    )
}

pub(super) fn red_rock() -> BrasaTheme {
    BrasaTheme::new(
        Fill::VerticalGradient(VerticalGradient::new(
            ColorU::from_u32(0x211719FF)
                .blend(&coloru_with_opacity(ColorU::from_u32(0x4C3435FF), 45)),
            ColorU::from_u32(0x211719FF)
                .blend(&coloru_with_opacity(ColorU::from_u32(0xD3032FF), 45)),
        )),
        ColorU::white(),
        Fill::Solid(ColorU::from_u32(0x9F4147FF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/red_rock_bg.jpg"),
            opacity: 30,
        }),
        Some("Red Rock".to_string()),
    )
}

pub(super) fn dark_city() -> BrasaTheme {
    BrasaTheme::new(
        Fill::VerticalGradient(VerticalGradient::new(
            ColorU::from_u32(0x01181FFF)
                .blend(&coloru_with_opacity(ColorU::from_u32(0x1A363FFF), 45)),
            ColorU::from_u32(0x01181FFF)
                .blend(&coloru_with_opacity(ColorU::from_u32(0x1A4551FF), 45)),
        )),
        ColorU::white(),
        Fill::Solid(ColorU::from_u32(0xE9072DFF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/dark_city_bg.jpg"),
            opacity: 20,
        }),
        Some("Dark City".to_string()),
    )
}

pub(super) fn sent_referral_reward() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x334567FF)),
        ColorU::white(),
        Fill::Solid(ColorU::from_u32(0xCD51FFFF)),
        None,
        Some(Details::Darker),
        dark_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/sent_referral_reward_bg.jpg"),
            opacity: 100,
        }),
        Some("Brasa Referral".to_string()),
    )
}

pub(super) fn solar_flare() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x1B1C18FF)),
        ColorU::from_u32(0xDDE6EEFF),
        Fill::Solid(ColorU::from_u32(0x34895CFF)),
        None,
        Some(Details::Darker),
        solarflare_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/solarflare_bg.jpg"),
            opacity: 20,
        }),
        Some("Solar Flare".to_string()),
    )
}

pub(super) fn adeberry() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x1D2022FF)),
        ColorU::from_u32(0xE4EEF5FF),
        Fill::Solid(ColorU::from_u32(0x6C96B4FF)),
        None,
        Some(Details::Darker),
        adeberry_colors(),
        None,
        Some("Adeberry".to_string()),
    )
}

pub(super) fn received_referral_reward() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xFFFFFFFF)),
        ColorU::black(),
        Fill::Solid(ColorU::from_u32(0xCD51FFFF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        Some(Image {
            source: bundled_or_fetched_asset!("jpg/received_referral_reward_bg.jpg"),
            opacity: 100,
        }),
        Some("Received Referral Reward".to_string()),
    )
}

// Temas de Brasa Code, con los mismos colores que en el editor.
pub(super) fn brasa_ash() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x292725FF)),
        ColorU::from_u32(0xE3E2E0FF),
        Fill::Solid(ColorU::from_u32(0xFAAD3AFF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x3A3835FF),
                AnsiColor::from_u32(0xFBC36FFF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xFAB44CFF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xD3D1CFFF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x75706AFF),
                AnsiColor::from_u32(0xFCCC83FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xFFD07AFF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF6F6F5FF),
            ),
        ),
        None,
        Some("Brasa Ash".to_string()),
    )
}

pub(super) fn brasa_midnight() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x151A28FF)),
        ColorU::from_u32(0xD8DEEBFF),
        Fill::Solid(ColorU::from_u32(0xFF7835FF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x212A3FFF),
                AnsiColor::from_u32(0xFF9C6BFF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xFF8447FF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xC2CBE0FF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x485D8CFF),
                AnsiColor::from_u32(0xFFAA80FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xFFD07AFF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF2F4F9FF),
            ),
        ),
        None,
        Some("Brasa Midnight".to_string()),
    )
}

pub(super) fn brasa_coal() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x060606FF)),
        ColorU::from_u32(0xE2E2E2FF),
        Fill::Solid(ColorU::from_u32(0xFF6435FF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x0D0D0DFF),
                AnsiColor::from_u32(0xFF8E6BFF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xFF7247FF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xD1D1D1FF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x222222FF),
                AnsiColor::from_u32(0xFF9E80FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xFFD07AFF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF6F6F6FF),
            ),
        ),
        None,
        Some("Brasa Coal".to_string()),
    )
}

pub(super) fn brasa_forest() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x182721FF)),
        ColorU::from_u32(0xDAE9E3FF),
        Fill::Solid(ColorU::from_u32(0xA7D95BFF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x253D33FF),
                AnsiColor::from_u32(0xBEE387FF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xAEDC6AFF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xC6DCD3FF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x51856FFF),
                AnsiColor::from_u32(0xC7E798FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xFFD07AFF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF3F8F6FF),
            ),
        ),
        None,
        Some("Brasa Forest".to_string()),
    )
}

pub(super) fn brasa_plum() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x241928FF)),
        ColorU::from_u32(0xE5DBE8FF),
        Fill::Solid(ColorU::from_u32(0xF53F9AFF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x38273DFF),
                AnsiColor::from_u32(0xF872B5FF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xF650A3FF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xD6C7DBFF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x785484FF),
                AnsiColor::from_u32(0xF986C0FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xFFD07AFF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF7F3F8FF),
            ),
        ),
        None,
        Some("Brasa Plum".to_string()),
    )
}

pub(super) fn brasa_sand() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xF5F0E7FF)),
        ColorU::from_u32(0x3B3226FF),
        Fill::Solid(ColorU::from_u32(0xD5461AFF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        None,
        Some("Brasa Sand".to_string()),
    )
}

pub(super) fn brasa_mist() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xEEF1F5FF)),
        ColorU::from_u32(0x27323FFF),
        Fill::Solid(ColorU::from_u32(0x156CD1FF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        None,
        Some("Brasa Mist".to_string()),
    )
}

pub(super) fn brasa_ocean() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x13232AFF)),
        ColorU::from_u32(0xD6E6EDFF),
        Fill::Solid(ColorU::from_u32(0x2CC8E0FF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x1D3743FF),
                AnsiColor::from_u32(0xFF7B86FF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xFFC46BFF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xBED8E4FF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x407B94FF),
                AnsiColor::from_u32(0xFF9AA3FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0x68C7E9FF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF1F7FAFF),
            ),
        ),
        None,
        Some("Brasa Ocean".to_string()),
    )
}

pub(super) fn brasa_aurora() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x181528FF)),
        ColorU::from_u32(0xDBD8EBFF),
        Fill::Solid(ColorU::from_u32(0xA78BFAFF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x25213FFF),
                AnsiColor::from_u32(0xFF7B86FF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xFFC46BFF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xC6C2E0FF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x5F54A4FF),
                AnsiColor::from_u32(0xFF9AA3FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xDFCDFDFF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF3F2F9FF),
            ),
        ),
        None,
        Some("Brasa Aurora".to_string()),
    )
}

pub(super) fn brasa_neon() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x1B0F27FF)),
        ColorU::from_u32(0xC9B3E2FF),
        Fill::Solid(ColorU::from_u32(0xFF4FB6FF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x2A183EFF),
                AnsiColor::from_u32(0xFF7B86FF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xFFC46BFF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xBA9EDBFF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x7341ABFF),
                AnsiColor::from_u32(0xFF9AA3FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xFF94C5FF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xDCCBEFFF),
            ),
        ),
        None,
        Some("Brasa Neon".to_string()),
    )
}

pub(super) fn brasa_glacier() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0x222730FF)),
        ColorU::from_u32(0xDEE2E8FF),
        Fill::Solid(ColorU::from_u32(0x88C0D0FF)),
        None,
        Some(Details::Darker),
        TerminalColors::new(
            AnsiColors::new(
                AnsiColor::from_u32(0x2F3743FF),
                AnsiColor::from_u32(0xFF7B86FF),
                AnsiColor::from_u32(0x9BD67EFF),
                AnsiColor::from_u32(0xEBCB8BFF),
                AnsiColor::from_u32(0x7EB6FFFF),
                AnsiColor::from_u32(0xF29BC9FF),
                AnsiColor::from_u32(0x6FD6CFFF),
                AnsiColor::from_u32(0xCBD1DBFF),
            ),
            AnsiColors::new(
                AnsiColor::from_u32(0x63748EFF),
                AnsiColor::from_u32(0xFF9AA3FF),
                AnsiColor::from_u32(0xB8E79EFF),
                AnsiColor::from_u32(0xB9D5E4FF),
                AnsiColor::from_u32(0xA5CBFFFF),
                AnsiColor::from_u32(0xF7BCDBFF),
                AnsiColor::from_u32(0x97E6E0FF),
                AnsiColor::from_u32(0xF4F6F8FF),
            ),
        ),
        None,
        Some("Brasa Glacier".to_string()),
    )
}

pub(super) fn brasa_sakura() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xF4E8EBFF)),
        ColorU::from_u32(0x3A2A30FF),
        Fill::Solid(ColorU::from_u32(0xD6336CFF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        None,
        Some("Brasa Sakura".to_string()),
    )
}

pub(super) fn brasa_matcha() -> BrasaTheme {
    BrasaTheme::new(
        Fill::Solid(ColorU::from_u32(0xEEF3E9FF)),
        ColorU::from_u32(0x2A3320FF),
        Fill::Solid(ColorU::from_u32(0x437F18FF)),
        None,
        Some(Details::Lighter),
        light_mode_colors(),
        None,
        Some("Brasa Matcha".to_string()),
    )
}
