mod support;

use storymesh::{Framework, scan};

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
