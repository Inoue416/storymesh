use std::{fs, path::Path};

use crate::scanner::{ScanError, matching};

use super::{has_declaration_extension, has_excluded_suffix};

pub(super) fn is_component(root: &Path, path: &Path) -> Result<bool, ScanError> {
    if matching::is_story_file(path) || has_declaration_extension(path) {
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
    Ok(contains_component_decorator(&source))
}

pub(super) fn contains_component_decorator(source: &[u8]) -> bool {
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
