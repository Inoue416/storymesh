use std::path::Path;

use crate::scanner::matching;

use super::has_excluded_suffix;

pub(super) fn is_component(path: &Path) -> bool {
    if matching::is_story_file(path) {
        return false;
    }

    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };

    extension.eq_ignore_ascii_case("vue") && !has_excluded_suffix(stem)
}
