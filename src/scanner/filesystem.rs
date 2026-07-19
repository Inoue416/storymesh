use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use super::ScanOptions;

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

    pub(super) fn invalid_ignore(detail: impl fmt::Display) -> Self {
        Self {
            context: format!("invalid ignore rule: {detail}"),
            source: None,
        }
    }
}

pub(super) struct IgnoreMatcher {
    gitignore: Gitignore,
}

impl IgnoreMatcher {
    pub(super) fn new(root: &Path, options: &ScanOptions) -> Result<Self, ScanError> {
        let mut builder = GitignoreBuilder::new(root);
        let default_file = root.join(".storymeshignore");
        if default_file.is_file() {
            add_ignore_file(&mut builder, &default_file)?;
        }
        for ignore_file in &options.ignore_files {
            let path = if ignore_file.is_absolute() {
                ignore_file.clone()
            } else {
                root.join(ignore_file)
            };
            add_ignore_file(&mut builder, &path)?;
        }
        for pattern in &options.ignore_patterns {
            builder
                .add_line(None, pattern)
                .map_err(ScanError::invalid_ignore)?;
        }

        builder
            .build()
            .map(|gitignore| Self { gitignore })
            .map_err(ScanError::invalid_ignore)
    }

    fn is_ignored(&self, root: &Path, relative: &Path) -> bool {
        self.gitignore
            .matched_path_or_any_parents(root.join(relative), false)
            .is_ignore()
    }
}

fn add_ignore_file(builder: &mut GitignoreBuilder, path: &Path) -> Result<(), ScanError> {
    if let Some(error) = builder.add(path) {
        return Err(ScanError::invalid_ignore(error));
    }
    Ok(())
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
    ignore_matcher: &IgnoreMatcher,
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
                collect_files(root, &path, ignore_matcher, files)?;
            }
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root).map_err(|_| {
                ScanError::invalid_root(root, "encountered a path outside the scan root")
            })?;
            if !ignore_matcher.is_ignored(root, relative) {
                files.push(relative.to_path_buf());
            }
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
