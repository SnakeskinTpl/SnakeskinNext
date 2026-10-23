use super::{tag, Tag};

/// Always succeeds without consuming input.
pub fn succeed() -> Tag<&'static str> {
    tag("")
}
