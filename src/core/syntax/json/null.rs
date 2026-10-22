use crate::parsec::{map, tag, Parser};

use super::JsonKind;

pub(crate) fn null<'a>() -> impl Parser<'a, Output = JsonKind<'a>> {
    map(tag("null"), |_| JsonKind::Null)
}
