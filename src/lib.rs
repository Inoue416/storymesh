mod domain;
mod scanner;

pub use domain::{ComponentCoverage, CoverageReport, Framework};
pub use scanner::{
    GenerateError, ScanError, ScanOptions, generate_story_skeletons, scan, scan_with_options,
};
