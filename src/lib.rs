mod domain;
mod scanner;

pub use domain::{ComponentCoverage, CoverageReport, Framework};
pub use scanner::{ScanError, scan};
