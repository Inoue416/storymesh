use std::path::Path;

use crate::scanner::matching;

use super::{has_declaration_extension, has_excluded_suffix, is_component_index, is_pascal_case};

pub(super) fn is_component(path: &Path) -> bool {
    if matching::is_story_file(path) || has_declaration_extension(path) {
        return false;
    }

    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };

    if has_excluded_suffix(stem) {
        return false;
    }

    match extension.to_ascii_lowercase().as_str() {
        "jsx" | "tsx" => stem != "main" && (stem != "index" || is_component_index(path)),
        "js" | "ts" => is_pascal_case(stem),
        _ => false,
    }
}
