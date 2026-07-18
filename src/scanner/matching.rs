use std::path::{Path, PathBuf};

use crate::domain::Framework;

pub(super) fn is_story_file(path: &Path) -> bool {
    story_keys(path).is_some()
}

pub(super) fn story_keys(path: &Path) -> Option<Vec<PathBuf>> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx"
    ) {
        return None;
    }

    let stem = path.file_stem()?.to_str()?;
    let component_name = stem.strip_suffix(".stories")?;
    if component_name.is_empty() {
        return None;
    }

    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let key = parent.join(component_name);
    let mut keys = index_aliases(key);
    if matches!(
        parent.file_name().and_then(|name| name.to_str()),
        Some("__stories__" | "stories")
    ) {
        let normalized_parent = parent.parent().unwrap_or_else(|| Path::new(""));
        keys.extend(index_aliases(normalized_parent.join(component_name)));
    }
    keys.sort();
    keys.dedup();
    Some(keys)
}

pub(super) fn component_keys(path: &Path, framework: Framework) -> Vec<PathBuf> {
    let mut key = path.to_path_buf();
    key.set_extension("");
    let mut keys = index_aliases(key.clone());
    if framework == Framework::Angular {
        let component_name = key
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|stem| stem.strip_suffix(".component"))
            .map(str::to_owned);
        if let Some(component_name) = component_name {
            key.set_file_name(component_name);
            keys.extend(index_aliases(key));
        }
    }
    keys
}

fn index_aliases(key: PathBuf) -> Vec<PathBuf> {
    if key.file_name().and_then(|name| name.to_str()) != Some("index") {
        return vec![key];
    }

    let Some(parent) = key.parent() else {
        return vec![key];
    };
    let Some(directory_name) = parent.file_name() else {
        return vec![key];
    };
    vec![key.clone(), parent.join(directory_name)]
}
