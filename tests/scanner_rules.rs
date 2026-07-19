mod support;

use std::path::Path;

use storymesh::{Framework, ScanOptions, scan, scan_with_options};

use support::TestProject;

#[test]
fn matches_index_components_with_stories_in_story_directories() {
    let project = TestProject::new();
    project.add("src/Avatar/index.tsx");
    project.add("src/Avatar/__stories__/Avatar.stories.ts");

    let report = scan(&project.root, Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}

#[test]
fn matches_angular_component_names_with_plain_story_names() {
    let project = TestProject::new();
    project.add("src/card/card.component.ts");
    project.add("src/card/card.stories.ts");

    let report = scan(&project.root, Framework::Angular).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}

#[test]
fn ignores_decorators_in_comments_and_strings() {
    let project = TestProject::new();
    project.add_with_contents(
        "src/app/app.config.ts",
        "// @Component({ commentedOut: true })\nexport const note = '@Component({ text: true })';\n",
    );

    let report = scan(&project.root, Framework::Angular).expect("the project should scan");

    assert!(report.components.is_empty());
}

#[test]
fn ignores_paths_listed_in_the_default_storymeshignore_file() {
    let project = TestProject::new();
    project.add("src/Button.tsx");
    project.add("src/generated/GeneratedCard.tsx");
    project.add_with_contents("src/.storymeshignore", "generated/\n");

    let report =
        scan(&project.root.join("src"), Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.components[0].component, Path::new("Button.tsx"));
}

#[test]
fn supports_gitignore_negation_rules() {
    let project = TestProject::new();
    project.add("src/generated/HiddenCard.tsx");
    project.add("src/generated/DocumentedButton.tsx");
    project.add_with_contents(
        "src/.storymeshignore",
        "generated/\n!generated/DocumentedButton.tsx\n",
    );

    let report =
        scan(&project.root.join("src"), Framework::React).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(
        report.components[0].component,
        Path::new("generated/DocumentedButton.tsx")
    );
}

#[test]
fn excludes_components_and_stories_with_explicit_ignore_patterns() {
    let project = TestProject::new();
    project.add("Button.tsx");
    project.add("Ignored.tsx");
    project.add("Ignored.stories.tsx");

    let report = scan_with_options(
        &project.root,
        Framework::React,
        &ScanOptions {
            ignore_patterns: vec!["Ignored*".into()],
            ignore_files: Vec::new(),
        },
    )
    .expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.components[0].component, Path::new("Button.tsx"));
}
