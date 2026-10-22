use crate::parsec::prelude::*;

use super::{json, JsonKind};

pub(crate) fn array<'a>() -> impl Parser<'a, Output = JsonKind<'a>> {
    let elements = opt(seq!(
        rec(json),
        repeat(map(seq!(
            ltrim(tag(",")),
            cut(rec(json))
        ), |(_, value)| value), 0..)
    ));

    map(seq!(
        tag("["),
        cut(seq!(elements, ltrim(tag("]"))))
    ), |(_, (elements, _))| {
        let mut values = Vec::new();

        if let Some((first, rest)) = elements {
            values.push(first);
            values.extend(rest);
        }

        JsonKind::Array(values)
    })
}
