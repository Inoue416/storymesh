mod support;

use std::{fs, path::PathBuf};

use storymesh::{ComponentCoverage, CoverageReport, Framework, generate_story_skeletons, scan};

use support::TestProject;

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

        let generated =
            generate_story_skeletons(&project.root, &report).expect("the story should generate");

        assert_eq!(generated, [PathBuf::from(story)]);
        let contents =
            fs::read_to_string(project.root.join(story)).expect("the generated story should read");
        assert!(contents.contains(expected_import));
        assert!(contents.contains("export const Default = {}"));

        let rescanned = scan(&project.root, framework).expect("the project should rescan");
        assert_eq!(rescanned.covered_count(), 1);
        assert!(rescanned.missing().next().is_none());
    }
}

#[test]
fn refuses_to_overwrite_existing_story_files() {
    let project = TestProject::new();
    project.add("src/Button.tsx");
    project.add_with_contents("src/Button.stories.tsx", "// keep me\n");
    let report = CoverageReport {
        framework: Framework::React,
        components: vec![ComponentCoverage {
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

    generate_story_skeletons(&project.root, &report).expect("the placeholder should generate");

    let contents = fs::read_to_string(project.root.join("src/Sample2.stories.tsx"))
        .expect("the placeholder should read");
    assert!(!contents.contains("import"));
    assert!(contents.contains("render: () => null"));
}
