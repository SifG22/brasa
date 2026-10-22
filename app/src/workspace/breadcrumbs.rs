use std::path::{Component, Path};

const SEPARATOR: &str = " › ";

/// Splits `path` into the leading breadcrumb trail (ending in a separator) and the last folder, so
/// the title bar can draw the folder you are in stronger than its parents. The home directory is
/// shown as `~`.
pub(crate) fn breadcrumbs(path: &Path, home: Option<&Path>) -> (String, String) {
    let (mut parts, rest) = match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) => (vec!["~".to_owned()], rest),
        None => (Vec::new(), path),
    };
    parts.extend(rest.components().filter_map(|component| match component {
        Component::Prefix(prefix) => Some(prefix.as_os_str().to_string_lossy().into_owned()),
        Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
        Component::RootDir | Component::CurDir | Component::ParentDir => None,
    }));
    match parts.pop() {
        Some(last) if parts.is_empty() => (String::new(), last),
        Some(last) => (format!("{}{SEPARATOR}", parts.join(SEPARATOR)), last),
        None => (String::new(), path.to_string_lossy().into_owned()),
    }
}

#[cfg(test)]
#[path = "breadcrumbs_tests.rs"]
mod tests;
