use std::borrow::Cow;
use std::ops::Range;

use crate::parsec::prelude::*;

mod array;
mod boolean;
mod err;
mod null;
mod number;
mod object;
mod string;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub(crate) struct JsonValue<'a> {
    pub span: Range<usize>,
    pub kind: JsonKind<'a>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum JsonKind<'a> {
    Null,
    Bool(bool),
    Number(f64),
    String(Cow<'a, str>),
    Array(Vec<JsonValue<'a>>),
    Object(Vec<JsonProperty<'a>>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct JsonProperty<'a> {
    pub key: Cow<'a, str>,
    pub key_span: Range<usize>,
    pub value: JsonValue<'a>,
}

/// Parses one JSON-like value, allowing identifier keys in objects.
/// Leading whitespace is consumed; trailing input remains for the enclosing grammar.
pub(crate) fn json<'a>() -> Box<dyn Parser<'a, Output = JsonValue<'a>> + 'a> {
    let value = choice!(
        null::null(),
        boolean::boolean(),
        map(string::string(), JsonKind::String),
        number::number(),
        object::object(),
        array::array()
    );

    Box::new(ltrim(map(spanned(value), |(span, kind)| JsonValue { span, kind })))
}
