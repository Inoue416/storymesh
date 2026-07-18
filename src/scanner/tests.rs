use std::path::{Path, PathBuf};

use crate::domain::Framework;

use super::matching;

#[test]
fn matches_index_and_story_directory_aliases() {
    assert_eq!(
        matching::story_keys(Path::new("src/Avatar/__stories__/Avatar.stories.ts")),
        Some(vec![
            PathBuf::from("src/Avatar/Avatar"),
            PathBuf::from("src/Avatar/__stories__/Avatar"),
        ])
    );
    assert_eq!(
        matching::component_keys(Path::new("src/Avatar/index.tsx"), Framework::React),
        vec![
            PathBuf::from("src/Avatar/index"),
            PathBuf::from("src/Avatar/Avatar"),
        ]
    );
}

#[test]
fn provides_both_angular_component_name_variants() {
    assert_eq!(
        matching::component_keys(Path::new("src/card/card.component.ts"), Framework::Angular),
        vec![
            PathBuf::from("src/card/card.component"),
            PathBuf::from("src/card/card"),
        ]
    );
}
