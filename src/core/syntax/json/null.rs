use crate::parsec::prelude::*;

use super::JsonKind;

pub(crate) fn null<'a>() -> impl Parser<'a, Output = JsonKind<'a>> {
    value(tag("null"), JsonKind::Null)
}
