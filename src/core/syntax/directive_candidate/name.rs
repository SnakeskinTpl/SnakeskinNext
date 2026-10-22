use crate::core::syntax::identifier;
use crate::parsec::prelude::*;

/// Parses an identifier followed by a required whitespace separator.
pub(crate) fn directive_name<'a>() -> impl Parser<'a, Output = &'a str> {
    map(
        seq!(
            identifier(),
            // Required separator: whitespace, e.g. a space in "foo value" or a newline.
            whitespace()
        ),
        |(name, _)| name
    )
}
