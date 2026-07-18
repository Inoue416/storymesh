mod support;

use std::path::Path;

use storymesh::{Framework, scan};

use support::TestProject;

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
fn matches_angular_component_named_stories_in_story_directories() {
    let project = TestProject::new();
    project.add("src/banner/banner.component.ts");
    project.add("src/banner/stories/banner.component.stories.ts");

    let report = scan(&project.root, Framework::Angular).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}

#[test]
fn detects_suffix_less_angular_components_and_ignores_comments_and_strings() {
    let project = TestProject::new();
    project.add_with_contents(
        "src/app/app.ts",
        "import { Component } from '@angular/core';\n@Component({ selector: 'app-root' })\nexport class App {}\n",
    );
    project.add("src/app/app.stories.ts");
    project.add_with_contents(
        "src/app/app.config.ts",
        "// @Component({ commentedOut: true })\nexport const note = '@Component({ text: true })';\n",
    );

    let report = scan(&project.root, Framework::Angular).expect("the project should scan");

    assert_eq!(report.components.len(), 1);
    assert_eq!(report.covered_count(), 1);
}
