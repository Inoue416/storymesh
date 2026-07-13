use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Component library whose Storybook coverage is being inspected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framework {
    /// React components and Component Story Format files.
    React,
}

impl Framework {
    /// Human-readable framework name used in reports.
    pub const fn name(self) -> &'static str {
        match self {
            Self::React => "React",
        }
    }
}

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

/// Error returned when a project tree cannot be inspected.
#[derive(Debug)]
pub struct ScanError {
    context: String,
    source: Option<io::Error>,
}

impl ScanError {
    fn invalid_root(root: &Path, detail: &str) -> Self {
        Self {
            context: format!("cannot scan {}: {detail}", root.display()),
            source: None,
        }
    }

    fn io(operation: &str, path: &Path, source: io::Error) -> Self {
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

/// Scan a directory for components and corresponding Storybook stories.
pub fn scan(root: &Path, framework: Framework) -> Result<CoverageReport, ScanError> {
    let metadata = fs::metadata(root).map_err(|error| ScanError::io("inspect", root, error))?;
    if !metadata.is_dir() {
        return Err(ScanError::invalid_root(root, "path is not a directory"));
    }

    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;

    let mut stories = BTreeMap::new();
    for relative_path in &files {
        if let Some(keys) = story_keys(relative_path) {
            for key in keys {
                stories.entry(key).or_insert_with(|| relative_path.clone());
            }
        }
    }

    let mut components = files
        .into_iter()
        .filter(|path| is_component(path, framework))
        .map(|component| {
            let story = component_keys(&component)
                .into_iter()
                .find_map(|key| stories.get(&key).cloned());
            ComponentCoverage { component, story }
        })
        .collect::<Vec<_>>();
    components.sort_by(|left, right| left.component.cmp(&right.component));

    Ok(CoverageReport {
        framework,
        components,
    })
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), ScanError> {
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

fn is_component(path: &Path, framework: Framework) -> bool {
    match framework {
        Framework::React => is_react_component(path),
    }
}

fn is_react_component(path: &Path) -> bool {
    if story_keys(path).is_some() {
        return false;
    }

    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
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
        "jsx" | "tsx" => stem != "index" || is_component_index(path),
        "js" | "ts" => is_pascal_case(stem),
        _ => false,
    }
}

fn is_component_index(path: &Path) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .is_some_and(is_pascal_case)
}

fn has_excluded_suffix(stem: &str) -> bool {
    [".test", ".spec", ".stories", ".story"]
        .iter()
        .any(|suffix| stem.ends_with(suffix))
}

fn is_pascal_case(stem: &str) -> bool {
    stem.chars().next().is_some_and(char::is_uppercase)
}

fn story_keys(path: &Path) -> Option<Vec<PathBuf>> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx"
    ) {
        return None;
    }

    let stem = path.file_stem()?.to_str()?;
    let component_name = stem.strip_suffix(".stories")?;
    if component_name.is_empty() {
        return None;
    }

    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let normalized_parent = match parent.file_name().and_then(|name| name.to_str()) {
        Some("__stories__" | "stories") => parent.parent().unwrap_or_else(|| Path::new("")),
        _ => parent,
    };
    let key = normalized_parent.join(component_name);
    Some(index_aliases(key))
}

fn component_keys(path: &Path) -> Vec<PathBuf> {
    let mut key = path.to_path_buf();
    key.set_extension("");
    index_aliases(key)
}

fn index_aliases(key: PathBuf) -> Vec<PathBuf> {
    if key.file_name().and_then(|name| name.to_str()) != Some("index") {
        return vec![key];
    }

    let Some(parent) = key.parent() else {
        return vec![key];
    };
    let Some(directory_name) = parent.file_name() else {
        return vec![key];
    };
    vec![key.clone(), parent.join(directory_name)]
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{Framework, scan};

    struct TestProject {
        root: PathBuf,
    }

    impl TestProject {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("the system clock should be after the Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir()
                .join(format!("storymesh-test-{}-{unique}", std::process::id()));
            fs::create_dir_all(&root).expect("the test project should be created");
            Self { root }
        }

        fn add(&self, relative: &str) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("the test parent should be created");
            }
            fs::write(path, "// test fixture\n").expect("the test file should be written");
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn finds_covered_and_missing_react_components() {
        let project = TestProject::new();
        project.add("src/Button.tsx");
        project.add("src/Button.stories.tsx");
        project.add("src/Card.jsx");

        let report = scan(&project.root, Framework::React).expect("the project should scan");

        assert_eq!(report.components.len(), 2);
        assert_eq!(report.covered_count(), 1);
        assert_eq!(
            report.missing().collect::<Vec<_>>(),
            [Path::new("src/Card.jsx")]
        );
        assert_eq!(report.percentage(), 50.0);
    }

    #[test]
    fn matches_story_directories_and_index_components() {
        let project = TestProject::new();
        project.add("src/components/Avatar/index.tsx");
        project.add("src/components/Avatar/__stories__/Avatar.stories.ts");
        project.add("src/components/Badge/Badge.tsx");
        project.add("src/components/Badge/stories/Badge.stories.js");

        let report = scan(&project.root, Framework::React).expect("the project should scan");

        assert_eq!(report.components.len(), 2);
        assert_eq!(report.covered_count(), 2);
        assert!(report.missing().next().is_none());
    }

    #[test]
    fn supports_pascal_case_javascript_components() {
        let project = TestProject::new();
        project.add("src/Alert.js");
        project.add("src/Alert.stories.mjs");
        project.add("src/AlertProps.d.ts");
        project.add("src/helpers.js");
        project.add("src/types.ts");

        let report = scan(&project.root, Framework::React).expect("the project should scan");

        assert_eq!(report.components.len(), 1);
        assert_eq!(report.covered_count(), 1);
    }

    #[test]
    fn ignores_generated_dependencies_and_test_files() {
        let project = TestProject::new();
        project.add("node_modules/package/External.tsx");
        project.add("dist/Bundle.jsx");
        project.add("src/Button.test.tsx");
        project.add("src/Button.spec.jsx");
        project.add("src/real-component.tsx");

        let report = scan(&project.root, Framework::React).expect("the project should scan");

        assert_eq!(report.components.len(), 1);
        assert_eq!(
            report.missing().collect::<Vec<_>>(),
            [Path::new("src/real-component.tsx")]
        );
    }

    #[test]
    fn empty_projects_have_full_coverage() {
        let project = TestProject::new();

        let report = scan(&project.root, Framework::React).expect("the project should scan");

        assert_eq!(report.percentage(), 100.0);
    }

    #[test]
    fn rejects_a_file_as_the_scan_root() {
        let project = TestProject::new();
        project.add("src/Button.tsx");

        let error = scan(&project.root.join("src/Button.tsx"), Framework::React)
            .expect_err("a file is not a valid scan root");

        assert!(error.to_string().contains("path is not a directory"));
        assert!(error.to_string().contains("Button.tsx"));
    }
}
