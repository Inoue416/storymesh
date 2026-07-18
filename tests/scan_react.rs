mod support;

use std::path::Path;

use storymesh::{Framework, scan};

use support::TestProject;

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
fn supports_pascal_case_javascript_components_and_ignores_entrypoints() {
    let project = TestProject::new();
    project.add("src/Alert.js");
    project.add("src/Alert.stories.mjs");
    project.add("src/AlertProps.d.ts");
    project.add("src/helpers.js");
    project.add("src/types.ts");
    project.add("src/entry/main.tsx");
    project.add("src/Main.tsx");
    project.add("src/Main.stories.tsx");

    let report = scan(&project.root, Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 2);
    assert_eq!(report.covered_count(), 2);
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
fn empty_projects_have_full_coverage_and_files_are_not_scan_roots() {
    let project = TestProject::new();

    let report = scan(&project.root, Framework::React).expect("the project should scan");
    assert_eq!(report.percentage(), 100.0);

    project.add("src/Button.tsx");
    let error = scan(&project.root.join("src/Button.tsx"), Framework::React)
        .expect_err("a file is not a valid scan root");
    assert!(error.to_string().contains("path is not a directory"));
}
