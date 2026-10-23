use std::borrow::Cow;
use std::ops::Range;

use crate::parsec::prelude::*;

mod boolean;
pub(crate) use boolean::boolean;

mod string;
pub(crate) use string::string;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub(crate) struct Literal<'a> {
    pub span: Range<usize>,
    pub kind: LiteralKind<'a>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum LiteralKind<'a> {
    Bool(bool),
    String(Cow<'a, str>),
}

/// Parses a language literal, consuming leading whitespace but leaving trailing input.
pub(crate) fn literal<'a>() -> impl Parser<'a, Output = Literal<'a>> {
    ltrim(map(
        spanned(choice(
            map(boolean(), LiteralKind::Bool),
            map(string(), LiteralKind::String)
        )),
        |(span, kind)| Literal { span, kind }
    ))
}
