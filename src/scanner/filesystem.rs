use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Error returned when a project tree cannot be inspected.
#[derive(Debug)]
pub struct ScanError {
    context: String,
    source: Option<io::Error>,
}

impl ScanError {
    pub(super) fn invalid_root(root: &Path, detail: &str) -> Self {
        Self {
            context: format!("cannot scan {}: {detail}", root.display()),
            source: None,
        }
    }

    pub(super) fn io(operation: &str, path: &Path, source: io::Error) -> Self {
        Self {
            context: format!("failed to {operation} {}", path.display()),
            source: Some(source),
        }
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.context)?;
        if let Some(source) = &self.source {
            write!(formatter, ": {source}")?;
        }
        Ok(())
    }
}

impl Error for ScanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

pub(super) fn validate_root(root: &Path) -> Result<(), ScanError> {
    let metadata = fs::metadata(root).map_err(|error| ScanError::io("inspect", root, error))?;
    if metadata.is_dir() {
        Ok(())
    } else {
        Err(ScanError::invalid_root(root, "path is not a directory"))
    }
}

pub(super) fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), ScanError> {
    let entries = fs::read_dir(directory)
        .map_err(|error| ScanError::io("read directory", directory, error))?;

    for entry in entries {
        let entry = entry.map_err(|error| ScanError::io("read an entry in", directory, error))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| ScanError::io("inspect", &path, error))?;

        if file_type.is_dir() {
            if !is_ignored_directory(&entry.file_name()) {
                collect_files(root, &path, files)?;
            }
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root).map_err(|_| {
                ScanError::invalid_root(root, "encountered a path outside the scan root")
            })?;
            files.push(relative.to_path_buf());
        }
    }

    Ok(())
}

fn is_ignored_directory(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(
            ".git"
                | ".next"
                | ".storybook"
                | "build"
                | "coverage"
                | "dist"
                | "node_modules"
                | "target"
        )
    )
}
