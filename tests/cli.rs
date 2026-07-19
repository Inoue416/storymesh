mod support;

use std::process::Command;

use support::TestProject;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_storymesh"))
        .args(args)
        .output()
        .expect("the storymesh binary should run")
}

#[test]
fn check_lists_missing_components_and_returns_one() {
    let project = TestProject::new();
    project.add("Button.tsx");
    project.add("Card.tsx");
    project.add("Button.stories.tsx");

    let output = run(&["check", project.root.to_str().expect("UTF-8 test path")]);

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Missing stories for 1 React component(s):"));
    assert!(stdout.contains("Card.tsx"));
    assert!(!stdout.contains("Button.tsx\n"));
}

#[test]
fn coverage_and_report_have_success_exit_codes() {
    let project = TestProject::new();
    project.add("Card.vue");

    let root = project.root.to_str().expect("UTF-8 test path");
    let coverage = run(&["coverage", root, "--framework", "vue"]);
    assert_eq!(coverage.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&coverage.stdout).contains("Vue Storybook coverage: 0.0%"));

    let report = run(&["report", root, "--framework", "vue"]);
    assert_eq!(report.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&report.stdout).contains("Missing: 1"));
}

#[test]
fn invalid_roots_return_an_actionable_error() {
    let output = run(&["check", "directory-that-does-not-exist"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("directory-that-does-not-exist"));
}

#[test]
fn check_generate_writes_story_skeletons_and_returns_zero() {
    let project = TestProject::new();
    project.add_with_contents("Card.tsx", "export default function Card() {}\n");

    let output = run(&[
        "check",
        project.root.to_str().expect("UTF-8 test path"),
        "--generate",
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(project.root.join("Card.stories.tsx").is_file());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generated 1 story skeleton(s):"));
    assert!(stdout.contains("Card.stories.tsx"));
}

#[test]
fn check_generate_returns_two_when_a_story_target_exists() {
    let project = TestProject::new();
    project.add_with_contents("Card.tsx", "export default function Card() {}\n");
    std::fs::create_dir(project.root.join("Card.stories.tsx"))
        .expect("the conflicting directory should be created");

    let output = run(&[
        "check",
        project.root.to_str().expect("UTF-8 test path"),
        "--generate",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
}

#[test]
fn check_excludes_patterns_and_additional_ignore_files() {
    let project = TestProject::new();
    project.add("Button.tsx");
    project.add("Generated.tsx");
    project.add("Legacy.tsx");
    project.add_with_contents("custom.ignore", "Legacy.tsx\n");

    let output = run(&[
        "check",
        project.root.to_str().expect("UTF-8 test path"),
        "--ignore",
        "Generated.tsx",
        "--ignore-file",
        "custom.ignore",
    ]);

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Button.tsx"));
    assert!(!stdout.contains("Generated.tsx"));
    assert!(!stdout.contains("Legacy.tsx"));
}
