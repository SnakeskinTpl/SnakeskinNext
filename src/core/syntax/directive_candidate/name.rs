use crate::parsec::{flat_map, take, Parser};
use crate::seq;

/// Parses an identifier followed by a required whitespace separator.
pub(crate) fn directive_name<'a>() -> impl Parser<'a, Output = &'a str> {
    let name = seq!(
        // First character: one letter or underscore, e.g. 'f' in "foo2".
        take(|ch, _, _| ch.is_alphabetic() || ch == '_', 1..=1),
        // Remaining characters: letters, digits or underscores, e.g. "oo_2" in "foo_2".
        take(|ch, _, _| ch.is_alphanumeric() || ch == '_', 0..),
        // Required separator: whitespace, e.g. a space in "foo value" or a newline.
        take(|ch, _, _| ch.is_whitespace(), 1..),
    );

    flat_map(name, |((first, rest), separator), input| {
        let end = input.byte_position() - separator.len();
        let start_byte = end - first.len() - rest.len();

        Ok((input.source_slice(start_byte..end), input))
    })
}
