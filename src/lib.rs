use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Component library whose Storybook coverage is being inspected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framework {
    /// React components and Component Story Format files.
    React,
    /// Vue single-file components and Component Story Format files.
    Vue,
    /// Angular components and Component Story Format files.
    Angular,
}

impl Framework {
    /// Human-readable framework name used in reports.
    pub const fn name(self) -> &'static str {
        match self {
            Self::React => "React",
            Self::Vue => "Vue",
            Self::Angular => "Angular",
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

/// Error returned when a story skeleton cannot be generated.
#[derive(Debug)]
pub struct GenerateError {
    context: String,
    source: Option<io::Error>,
}

impl GenerateError {
    fn invalid_component(component: &Path, detail: &str) -> Self {
        Self {
            context: format!(
                "cannot generate a story for {}: {detail}",
                component.display()
            ),
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

impl fmt::Display for GenerateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.context)?;
        if let Some(source) = &self.source {
            write!(formatter, ": {source}")?;
        }
        Ok(())
    }
}

impl Error for GenerateError {
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

    let mut components = Vec::new();
    for component in files {
        if is_component(root, &component, framework)? {
            let story = component_keys(&component, framework)
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

/// Write a minimal CSF story beside every component missing one.
///
/// Existing files are never overwritten. Returned paths are relative to `root`.
pub fn generate_story_skeletons(
    root: &Path,
    report: &CoverageReport,
) -> Result<Vec<PathBuf>, GenerateError> {
    let skeletons = report
        .missing()
        .map(|component| story_skeleton(root, component, report.framework))
        .collect::<Result<Vec<_>, _>>()?;

    for (path, _) in &skeletons {
        let absolute_path = root.join(path);
        if absolute_path.exists() {
            return Err(GenerateError::invalid_component(
                path,
                "the target story file already exists",
            ));
        }
    }

    for (path, contents) in &skeletons {
        let absolute_path = root.join(path);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&absolute_path)
            .map_err(|error| GenerateError::io("create story skeleton", &absolute_path, error))?;
        file.write_all(contents.as_bytes())
            .map_err(|error| GenerateError::io("write story skeleton", &absolute_path, error))?;
    }

    Ok(skeletons.into_iter().map(|(path, _)| path).collect())
}

fn story_skeleton(
    root: &Path,
    component: &Path,
    framework: Framework,
) -> Result<(PathBuf, String), GenerateError> {
    let file_name = component
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| GenerateError::invalid_component(component, "file name is not UTF-8"))?;
    let stem = component
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| GenerateError::invalid_component(component, "file stem is not UTF-8"))?;
    let extension = component
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| GenerateError::invalid_component(component, "extension is not UTF-8"))?;

    let story_extension = match framework {
        Framework::React => extension,
        Framework::Vue | Framework::Angular => "ts",
    };
    let mut story_path = component.to_path_buf();
    story_path.set_file_name(format!("{stem}.stories.{story_extension}"));

    let import_stem = match framework {
        Framework::Vue => file_name.to_owned(),
        Framework::React | Framework::Angular => stem.to_owned(),
    };
    let import_path = escape_javascript_string(&format!("./{import_stem}"));
    let import = match framework {
        Framework::React => react_import(root, component, &import_path)?,
        Framework::Vue => Some(format!("import Component from '{import_path}';")),
        Framework::Angular => {
            let export_name = angular_export_name(stem);
            Some(format!(
                "import {{ {export_name} as Component }} from '{import_path}';"
            ))
        }
    };
    let title = escape_javascript_string(&component_title(component, stem));
    let contents = match import {
        Some(import) => format!(
            "{import}\n\nconst meta = {{\n  title: '{title}',\n  component: Component,\n}};\n\nexport default meta;\n\nexport const Default = {{}};\n"
        ),
        None => format!(
            "const meta = {{\n  title: '{title}',\n}};\n\nexport default meta;\n\nexport const Default = {{\n  render: () => null,\n}};\n"
        ),
    };

    Ok((story_path, contents))
}

fn react_import(
    root: &Path,
    component: &Path,
    import_path: &str,
) -> Result<Option<String>, GenerateError> {
    let absolute_path = root.join(component);
    let source = fs::read(&absolute_path)
        .map_err(|error| GenerateError::io("read React component", &absolute_path, error))?;
    if contains_default_export(&source) {
        return Ok(Some(format!("import Component from '{import_path}';")));
    }

    let component_name = component_name(component);
    let export_name = pascal_case_identifier(component_name);
    if export_name.is_empty() || !contains_named_export(&source, &export_name) {
        return Ok(None);
    }
    Ok(Some(format!(
        "import {{ {export_name} as Component }} from '{import_path}';"
    )))
}

fn contains_default_export(source: &[u8]) -> bool {
    javascript_identifiers(source)
        .windows(2)
        .any(|tokens| tokens[0] == b"export" && tokens[1] == b"default")
}

fn contains_named_export(source: &[u8], export_name: &str) -> bool {
    let identifiers = javascript_identifiers(source);
    let export_name = export_name.as_bytes();
    identifiers.windows(3).any(|tokens| {
        tokens[0] == b"export"
            && matches!(
                tokens[1],
                b"function" | b"class" | b"const" | b"let" | b"var"
            )
            && tokens[2] == export_name
    }) || identifiers.windows(4).any(|tokens| {
        tokens[0] == b"export"
            && ((tokens[1] == b"async" && tokens[2] == b"function") || tokens[2] == b"as")
            && tokens[3] == export_name
    }) || identifiers
        .windows(2)
        .any(|tokens| tokens[0] == b"export" && tokens[1] == export_name)
}

fn javascript_identifiers(source: &[u8]) -> Vec<&[u8]> {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment,
        Quoted(u8),
    }

    let mut state = State::Code;
    let mut identifiers = Vec::new();
    let mut index = 0;
    while index < source.len() {
        match state {
            State::Code => {
                if source[index..].starts_with(b"//") {
                    state = State::LineComment;
                    index += 2;
                    continue;
                }
                if source[index..].starts_with(b"/*") {
                    state = State::BlockComment;
                    index += 2;
                    continue;
                }
                if matches!(source[index], b'\'' | b'"' | b'`') {
                    state = State::Quoted(source[index]);
                    index += 1;
                    continue;
                }
                if source[index].is_ascii_alphabetic() || matches!(source[index], b'_' | b'$') {
                    let start = index;
                    index += 1;
                    while source.get(index).is_some_and(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
                    }) {
                        index += 1;
                    }
                    identifiers.push(&source[start..index]);
                    continue;
                }
            }
            State::LineComment => {
                if source[index] == b'\n' {
                    state = State::Code;
                }
            }
            State::BlockComment => {
                if source[index..].starts_with(b"*/") {
                    state = State::Code;
                    index += 2;
                    continue;
                }
            }
            State::Quoted(quote) => {
                if source[index] == b'\\' {
                    index = (index + 2).min(source.len());
                    continue;
                }
                if source[index] == quote {
                    state = State::Code;
                }
            }
        }
        index += 1;
    }
    identifiers
}

fn component_title(component: &Path, stem: &str) -> String {
    let name = if stem == "index" {
        component_name(component)
    } else {
        stem.strip_suffix(".component").unwrap_or(stem)
    };
    format!("Components/{name}")
}

fn component_name(component: &Path) -> &str {
    let stem = component
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    if stem == "index" {
        component
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or(stem)
    } else {
        stem
    }
}

fn angular_export_name(stem: &str) -> String {
    let (name, suffix) = match stem.strip_suffix(".component") {
        Some(name) => (name, "Component"),
        None => (stem, ""),
    };
    let pascal_case = pascal_case_identifier(name);
    format!("{pascal_case}{suffix}")
}

fn pascal_case_identifier(name: &str) -> String {
    name.split(|character: char| !character.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().chain(characters).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn escape_javascript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
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

fn is_component(root: &Path, path: &Path, framework: Framework) -> Result<bool, ScanError> {
    match framework {
        Framework::React => Ok(is_react_component(path)),
        Framework::Vue => Ok(is_vue_component(path)),
        Framework::Angular => is_angular_component(root, path),
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
        "jsx" | "tsx" => stem != "main" && (stem != "index" || is_component_index(path)),
        "js" | "ts" => is_pascal_case(stem),
        _ => false,
    }
}

fn is_vue_component(path: &Path) -> bool {
    is_component_with_suffix(path, "vue", "")
}

fn is_angular_component(root: &Path, path: &Path) -> Result<bool, ScanError> {
    if story_keys(path).is_some() || has_declaration_extension(path) {
        return Ok(false);
    }

    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return Ok(false);
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Ok(false);
    };

    if !extension.eq_ignore_ascii_case("ts") || has_excluded_suffix(stem) {
        return Ok(false);
    }
    if stem.ends_with(".component") && stem.len() > ".component".len() {
        return Ok(true);
    }

    let absolute_path = root.join(path);
    let source = fs::read(&absolute_path)
        .map_err(|error| ScanError::io("read Angular source", &absolute_path, error))?;
    Ok(contains_angular_component_decorator(&source))
}

fn contains_angular_component_decorator(source: &[u8]) -> bool {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment,
        Quoted(u8),
    }

    const DECORATOR: &[u8] = b"@Component";
    let mut state = State::Code;
    let mut index = 0;
    while index < source.len() {
        match state {
            State::Code => {
                if source[index..].starts_with(b"//") {
                    state = State::LineComment;
                    index += 2;
                    continue;
                }
                if source[index..].starts_with(b"/*") {
                    state = State::BlockComment;
                    index += 2;
                    continue;
                }
                if matches!(source[index], b'\'' | b'"' | b'`') {
                    state = State::Quoted(source[index]);
                    index += 1;
                    continue;
                }
                if source[index..].starts_with(DECORATOR) {
                    let remainder = &source[index + DECORATOR.len()..];
                    if remainder
                        .iter()
                        .copied()
                        .find(|byte| !byte.is_ascii_whitespace())
                        == Some(b'(')
                    {
                        return true;
                    }
                }
            }
            State::LineComment => {
                if source[index] == b'\n' {
                    state = State::Code;
                }
            }
            State::BlockComment => {
                if source[index..].starts_with(b"*/") {
                    state = State::Code;
                    index += 2;
                    continue;
                }
            }
            State::Quoted(quote) => {
                if source[index] == b'\\' {
                    index = (index + 2).min(source.len());
                    continue;
                }
                if source[index] == quote {
                    state = State::Code;
                }
            }
        }
        index += 1;
    }
    false
}

fn is_component_with_suffix(path: &Path, expected_extension: &str, suffix: &str) -> bool {
    if story_keys(path).is_some() {
        return false;
    }

    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };

    extension.eq_ignore_ascii_case(expected_extension)
        && stem.ends_with(suffix)
        && stem.len() > suffix.len()
        && !has_excluded_suffix(stem)
}

fn has_declaration_extension(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
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
    let key = parent.join(component_name);
    let mut keys = index_aliases(key);
    if matches!(
        parent.file_name().and_then(|name| name.to_str()),
        Some("__stories__" | "stories")
    ) {
        let normalized_parent = parent.parent().unwrap_or_else(|| Path::new(""));
        keys.extend(index_aliases(normalized_parent.join(component_name)));
    }
    keys.sort();
    keys.dedup();
    Some(keys)
}

fn component_keys(path: &Path, framework: Framework) -> Vec<PathBuf> {
    let mut key = path.to_path_buf();
    key.set_extension("");
    let mut keys = index_aliases(key.clone());
    if framework == Framework::Angular {
        let component_name = key
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|stem| stem.strip_suffix(".component"))
            .map(str::to_owned);
        if let Some(component_name) = component_name {
            key.set_file_name(component_name);
            keys.extend(index_aliases(key));
        }
    }
    keys
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
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{Framework, generate_story_skeletons, scan};

    struct TestProject {
        root: PathBuf,
    }

    static NEXT_PROJECT_ID: AtomicU64 = AtomicU64::new(0);

    impl TestProject {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("the system clock should be after the Unix epoch")
                .as_nanos();
            let project_id = NEXT_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "storymesh-test-{}-{unique}-{project_id}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("the test project should be created");
            Self { root }
        }

        fn add(&self, relative: &str) {
            self.add_with_contents(relative, "// test fixture\n");
        }

        fn add_with_contents(&self, relative: &str, contents: &str) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("the test parent should be created");
            }
            fs::write(path, contents).expect("the test file should be written");
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
    fn matches_stories_kept_beside_components_in_a_stories_directory() {
        let project = TestProject::new();
        project.add("src/stories/Button.tsx");
        project.add("src/stories/Button.stories.ts");

        let report = scan(&project.root, Framework::React).expect("the project should scan");

        assert_eq!(report.components.len(), 1);
        assert_eq!(report.covered_count(), 1);
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
    fn ignores_react_entrypoints_but_keeps_main_components() {
        let project = TestProject::new();
        project.add("src/entry/main.tsx");
        project.add("src/Main.tsx");
        project.add("src/Main.stories.tsx");

        let report = scan(&project.root, Framework::React).expect("the project should scan");

        assert_eq!(report.components.len(), 1);
        assert_eq!(report.covered_count(), 1);
    }

    #[test]
    fn finds_covered_and_missing_vue_components() {
        let project = TestProject::new();
        project.add("src/Button.vue");
        project.add("src/Button.stories.ts");
        project.add("src/Card.vue");
        project.add("src/Card.test.vue");
        project.add("src/helpers.ts");

        let report = scan(&project.root, Framework::Vue).expect("the project should scan");

        assert_eq!(report.framework, Framework::Vue);
        assert_eq!(report.components.len(), 2);
        assert_eq!(report.covered_count(), 1);
        assert_eq!(
            report.missing().collect::<Vec<_>>(),
            [Path::new("src/Card.vue")]
        );
    }

    #[test]
    fn matches_vue_story_directories_and_index_components() {
        let project = TestProject::new();
        project.add("src/Avatar/index.vue");
        project.add("src/Avatar/__stories__/Avatar.stories.js");

        let report = scan(&project.root, Framework::Vue).expect("the project should scan");

        assert_eq!(report.components.len(), 1);
        assert_eq!(report.covered_count(), 1);
    }

    #[test]
    fn finds_covered_and_missing_angular_components() {
        let project = TestProject::new();
        project.add("src/button/button.component.ts");
        project.add("src/button/button.stories.ts");
        project.add("src/card/card.component.ts");
        project.add("src/card/card.component.spec.ts");
        project.add("src/card/card.service.ts");

        let report = scan(&project.root, Framework::Angular).expect("the project should scan");

        assert_eq!(report.framework, Framework::Angular);
        assert_eq!(report.components.len(), 2);
        assert_eq!(report.covered_count(), 1);
        assert_eq!(
            report.missing().collect::<Vec<_>>(),
            [Path::new("src/card/card.component.ts")]
        );
    }

    #[test]
    fn matches_angular_component_named_stories_and_story_directories() {
        let project = TestProject::new();
        project.add("src/banner/banner.component.ts");
        project.add("src/banner/stories/banner.component.stories.ts");

        let report = scan(&project.root, Framework::Angular).expect("the project should scan");

        assert_eq!(report.components.len(), 1);
        assert_eq!(report.covered_count(), 1);
    }

    #[test]
    fn finds_modern_angular_components_without_a_component_suffix() {
        let project = TestProject::new();
        project.add_with_contents(
            "src/app/app.ts",
            "import { Component } from '@angular/core';\n@Component({ selector: 'app-root' })\nexport class App {}\n",
        );
        project.add("src/app/app.stories.ts");
        project.add_with_contents(
            "src/app/app.config.ts",
            "// @Component({ commentedOut: true })\nexport const note = '@Component({ text: true })';\nexport const appConfig = { providers: [] };\n",
        );

        let report = scan(&project.root, Framework::Angular).expect("the project should scan");

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

    #[test]
    fn generates_framework_specific_story_skeletons_without_overwriting() {
        let cases = [
            (
                Framework::React,
                "src/Button.tsx",
                "export default function Button() {}\n",
                "src/Button.stories.tsx",
                "import Component from './Button';",
            ),
            (
                Framework::React,
                "src/MissingCard.tsx",
                "// export default is only an example\nexport function MissingCard() {}\n",
                "src/MissingCard.stories.tsx",
                "import { MissingCard as Component } from './MissingCard';",
            ),
            (
                Framework::Vue,
                "src/Card.vue",
                "<template><div>Card</div></template>\n",
                "src/Card.stories.ts",
                "import Component from './Card.vue';",
            ),
            (
                Framework::Angular,
                "src/user-card.component.ts",
                "export class UserCardComponent {}\n",
                "src/user-card.component.stories.ts",
                "import { UserCardComponent as Component } from './user-card.component';",
            ),
        ];

        for (framework, component, source, story, expected_import) in cases {
            let project = TestProject::new();
            project.add_with_contents(component, source);
            let report = scan(&project.root, framework).expect("the project should scan");

            let generated = generate_story_skeletons(&project.root, &report)
                .expect("the story skeleton should be generated");

            assert_eq!(generated, [PathBuf::from(story)]);
            let contents = fs::read_to_string(project.root.join(story))
                .expect("the generated story should be readable");
            assert!(contents.contains(expected_import));
            assert!(contents.contains("export const Default = {}"));

            let rescanned = scan(&project.root, framework).expect("the project should rescan");
            assert_eq!(rescanned.covered_count(), 1);
            assert!(rescanned.missing().next().is_none());
        }
    }

    #[test]
    fn refuses_to_overwrite_an_existing_story_skeleton() {
        let project = TestProject::new();
        project.add("src/Button.tsx");
        project.add_with_contents("src/Button.stories.tsx", "// keep me\n");
        let report = super::CoverageReport {
            framework: Framework::React,
            components: vec![super::ComponentCoverage {
                component: PathBuf::from("src/Button.tsx"),
                story: None,
            }],
        };

        let error = generate_story_skeletons(&project.root, &report)
            .expect_err("an existing story must not be overwritten");

        assert!(error.to_string().contains("already exists"));
        assert_eq!(
            fs::read_to_string(project.root.join("src/Button.stories.tsx"))
                .expect("the existing story should remain readable"),
            "// keep me\n"
        );
    }

    #[test]
    fn generates_a_buildable_placeholder_when_a_react_export_cannot_be_found() {
        let project = TestProject::new();
        project.add_with_contents("src/Sample2.tsx", "");
        let report = scan(&project.root, Framework::React).expect("the project should scan");

        generate_story_skeletons(&project.root, &report)
            .expect("the placeholder story should be generated");

        let contents = fs::read_to_string(project.root.join("src/Sample2.stories.tsx"))
            .expect("the placeholder story should be readable");
        assert!(!contents.contains("import"));
        assert!(contents.contains("render: () => null"));
    }
}
