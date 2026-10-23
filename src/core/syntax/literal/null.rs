use crate::parsec::prelude::*;

#[cfg(test)]
mod tests;

/// Matches the absence literal without accepting identifier prefixes such as `nullable`.
pub(crate) fn null<'a>() -> impl Parser<'a, Output = ()> {
    void(seq(
        tag("null"),
        not(take(|ch, _, _| ch.is_alphanumeric() || ch == '_', 1..=1))
    ))
}
