mod detector;
mod filesystem;
mod generation;
mod matching;

use std::{collections::BTreeMap, path::Path};

use crate::domain::{ComponentCoverage, CoverageReport, Framework};

pub use filesystem::ScanError;
pub use generation::{GenerateError, generate_story_skeletons};

/// Scan a directory for components and corresponding Storybook stories.
pub fn scan(root: &Path, framework: Framework) -> Result<CoverageReport, ScanError> {
    filesystem::validate_root(root)?;

    let mut files = Vec::new();
    filesystem::collect_files(root, root, &mut files)?;

    let mut stories = BTreeMap::new();
    for relative_path in &files {
        if let Some(keys) = matching::story_keys(relative_path) {
            for key in keys {
                stories.entry(key).or_insert_with(|| relative_path.clone());
            }
        }
    }

    let mut components = Vec::new();
    for component in files {
        if detector::is_component(root, &component, framework)? {
            let story = matching::component_keys(&component, framework)
                .into_iter()
                .find_map(|key| stories.get(&key).cloned());
            components.push(ComponentCoverage { component, story });
        }
    }
    components.sort_by(|left, right| left.component.cmp(&right.component));

    Ok(CoverageReport {
        framework,
        components,
    })
}

#[cfg(test)]
mod tests;
