use std::borrow::Cow;
use std::ops::Range;

use crate::parsec::prelude::*;

mod boolean;
pub(crate) use boolean::boolean;

mod null;
pub(crate) use null::null;

mod string;
pub(crate) use string::string;

mod number;
pub(crate) use number::{number, NumberLiteral, NumberForm, NumberSuffix};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Literal<'a> {
    pub span: Range<usize>,
    pub kind: LiteralKind<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LiteralKind<'a> {
    Null,
    Bool(bool),
    String(Cow<'a, str>),
    Number(NumberLiteral<'a>),
}

/// Parses a language literal, consuming leading whitespace but leaving trailing input.
pub(crate) fn literal<'a>() -> impl Parser<'a, Output = Literal<'a>> {
    ltrim(map(
        spanned(choice!(
            value(null(), LiteralKind::Null),
            map(boolean(), LiteralKind::Bool),
            map(string(), LiteralKind::String),
            map(number(), LiteralKind::Number)
        )),

        |(span, kind)| Literal { span, kind }
    ))
}
