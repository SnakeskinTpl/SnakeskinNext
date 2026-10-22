use crate::parsec::prelude::*;

#[cfg(test)]
mod tests;

/// Matches a letter or underscore followed by letters, digits or underscores.
/// Returns one source slice without consuming a separator.
pub(crate) fn identifier<'a>() -> impl Parser<'a, Output = &'a str> {
    recognize(
        seq!(
            // First character, e.g. "t" in "tag_2".
            take(|ch, _, _| ch.is_alphabetic() || ch == '_', 1..=1),
            // Remaining characters, e.g. "ag_2".
            take(|ch, _, _| ch.is_alphanumeric() || ch == '_', 0..)
        )
    )
}
