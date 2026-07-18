mod angular;
mod react;
mod vue;

use std::path::Path;

use crate::{domain::Framework, scanner::ScanError};

pub(super) fn is_component(
    root: &Path,
    path: &Path,
    framework: Framework,
) -> Result<bool, ScanError> {
    match framework {
        Framework::React => Ok(react::is_component(path)),
        Framework::Vue => Ok(vue::is_component(path)),
        Framework::Angular => angular::is_component(root, path),
    }
}

fn has_declaration_extension(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

fn has_excluded_suffix(stem: &str) -> bool {
    [".test", ".spec", ".stories", ".story"]
        .iter()
        .any(|suffix| stem.ends_with(suffix))
}

fn is_component_index(path: &Path) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .is_some_and(is_pascal_case)
}

fn is_pascal_case(stem: &str) -> bool {
    stem.chars().next().is_some_and(char::is_uppercase)
}
