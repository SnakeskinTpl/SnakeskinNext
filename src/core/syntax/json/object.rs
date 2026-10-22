use std::borrow::Cow;

use crate::parsec::prelude::*;

use super::super::identifier;
use super::string::string;
use super::{json, JsonKind, JsonProperty};

pub(crate) fn object<'a>() -> impl Parser<'a, Output = JsonKind<'a>> {
    let properties = opt(seq!(
        ltrim(property()),
        repeat(map(seq!(
            ltrim(tag(",")),
            cut(ltrim(property()))
        ), |(_, property)| property), 0..)
    ));

    map(seq!(
        tag("{"),
        cut(seq!(properties, ltrim(tag("}"))))
    ), |(_, (properties, _))| {
        let mut values = Vec::new();

        if let Some((first, rest)) = properties {
            values.push(first);
            values.extend(rest);
        }

        // Preserve order and duplicates for directive-specific validation.
        JsonKind::Object(values)
    })
}

fn property<'a>() -> impl Parser<'a, Output = JsonProperty<'a>> {
    let key = choice(string(), map(identifier(), Cow::Borrowed));

    map(
        seq!(
            spanned(key),
            cut(seq!(ltrim(tag(":")), rec(json)))
        ),
        |((key_span, key), (_, value))| JsonProperty { key, key_span, value }
    )
}
