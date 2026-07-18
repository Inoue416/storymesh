use std::path::Path;

use super::{angular, react, vue};

#[test]
fn angular_decorator_parser_ignores_comments_and_quoted_text() {
    let source =
        b"// @Component()\nconst note = '@Component()';\n/* @Component() */\n@Component ({})";

    assert!(angular::contains_component_decorator(source));
}

#[test]
fn component_detectors_apply_framework_specific_naming_rules() {
    assert!(react::is_component(Path::new("Button.tsx")));
    assert!(!react::is_component(Path::new("button.ts")));
    assert!(vue::is_component(Path::new("Button.vue")));
    assert!(!vue::is_component(Path::new("Button.test.vue")));
}
