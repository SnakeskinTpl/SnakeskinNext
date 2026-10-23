use std::borrow::Cow;
use std::ops::Range;

use crate::parsec::prelude::*;

use super::literal::{boolean, number, string, NumberLiteral};

mod array;
mod null;
mod object;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JsonValue<'a> {
    pub span: Range<usize>,
    pub kind: JsonKind<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum JsonKind<'a> {
    Null,
    Bool(bool),
    Number(NumberLiteral<'a>),
    String(Cow<'a, str>),
    Array(Vec<JsonValue<'a>>),
    Object(Vec<JsonProperty<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JsonProperty<'a> {
    pub key: Cow<'a, str>,
    pub key_span: Range<usize>,
    pub value: JsonValue<'a>,
}

/// Parses one JSON-like value with SS literals and identifier keys in objects.
/// Leading whitespace is consumed; trailing input remains for the enclosing grammar.
pub(crate) fn json<'a>() -> Box<dyn Parser<'a, Output = JsonValue<'a>> + 'a> {
    let value = choice!(
        null::null(),
        map(boolean(), JsonKind::Bool),
        map(string(), JsonKind::String),
        map(number(), JsonKind::Number),
        object::object(),
        array::array()
    );

    Box::new(ltrim(map(spanned(value), |(span, kind)| JsonValue { span, kind })))
}
