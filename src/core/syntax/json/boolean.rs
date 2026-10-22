use crate::parsec::{choice, map, tag, Parser};

use super::JsonKind;

pub(crate) fn boolean<'a>() -> impl Parser<'a, Output = JsonKind<'a>> {
    map(choice(tag("true"), tag("false")), |value| JsonKind::Bool(value == "true"))
}
