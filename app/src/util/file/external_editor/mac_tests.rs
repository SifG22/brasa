use super::is_brasa_bundle;

#[test]
fn is_brasa_bundle_recognises_brasa_channels() {
    assert!(is_brasa_bundle("dev.brasa.Brasa"));
    assert!(is_brasa_bundle("dev.brasa.BrasaDev"));
    assert!(is_brasa_bundle("dev.brasa.BrasaPreview"));
    assert!(is_brasa_bundle("dev.brasa.BrasaOss"));
}

#[test]
fn is_brasa_bundle_rejects_other_apps() {
    assert!(!is_brasa_bundle("com.microsoft.VSCode"));
    assert!(!is_brasa_bundle("com.apple.TextEdit"));
    assert!(!is_brasa_bundle("dev.zed.Zed"));
    assert!(!is_brasa_bundle("invalid"));
    assert!(!is_brasa_bundle(""));
}
