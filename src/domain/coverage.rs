use std::path::{Path, PathBuf};

use super::Framework;

/// One component and the story file that covers it, when present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentCoverage {
    pub component: PathBuf,
    pub story: Option<PathBuf>,
}

/// Story coverage discovered below a scan root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageReport {
    pub framework: Framework,
    pub components: Vec<ComponentCoverage>,
}

impl CoverageReport {
    /// Number of discovered components with a corresponding story.
    pub fn covered_count(&self) -> usize {
        self.components
            .iter()
            .filter(|component| component.story.is_some())
            .count()
    }

    /// Components for which no corresponding story was found.
    pub fn missing(&self) -> impl Iterator<Item = &Path> {
        self.components
            .iter()
            .filter(|component| component.story.is_none())
            .map(|component| component.component.as_path())
    }

    /// Covered components as a percentage from 0 to 100.
    pub fn percentage(&self) -> f64 {
        if self.components.is_empty() {
            100.0
        } else {
            self.covered_count() as f64 / self.components.len() as f64 * 100.0
        }
    }
}
