use crate::parsec::prelude::*;

/// Boolean keywords must not be followed by an identifier character.
pub(crate) fn boolean<'a>() -> impl Parser<'a, Output = bool> {
    map(
        seq(
            choice(tag("true"), tag("false")),
            not(take(|ch, _, _| ch.is_alphanumeric() || ch == '_', 1..=1))
        ),
        |(keyword, ())| keyword == "true"
    )
}
