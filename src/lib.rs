mod domain;
mod scanner;

pub use domain::{ComponentCoverage, CoverageReport, Framework};
pub use scanner::{GenerateError, ScanError, generate_story_skeletons, scan};
