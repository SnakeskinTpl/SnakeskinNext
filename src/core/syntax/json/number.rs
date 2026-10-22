use crate::parsec::{choice, cut, map, opt, flat_map, recognize, tag, take, ErrorLocation, Parser};
use crate::seq;

use super::err::JsonError;
use super::JsonKind;

pub(crate) fn number<'a>() -> impl Parser<'a, Output = JsonKind<'a>> {
    let integer = choice(
        tag("0"),
        map(seq!(
            take(|ch, _, _| matches!(ch, '1'..='9'), 1..=1),
            take(|ch, _, _| ch.is_ascii_digit(), 0..)
        ), |(first, _)| first)
    );
    
    let fraction = seq!(
        tag("."),
        cut(take(|ch, _, _| ch.is_ascii_digit(), 1..))
    );
    
    let exponent = seq!(
        choice(tag("e"), tag("E")),
        opt(choice(tag("+"), tag("-"))),
        cut(take(|ch, _, _| ch.is_ascii_digit(), 1..))
    );
    
    let grammar = seq!(opt(tag("-")), integer, opt(fraction), opt(exponent));
    flat_map(recognize(grammar), |raw, remaining| {
        let start_byte = remaining.byte_position() - raw.len();

        match raw.parse::<f64>() {
            Ok(value) if value.is_finite() => Ok((JsonKind::Number(value), remaining)),
            _ => Err(JsonError::new(ErrorLocation {
                start_byte,
                failure_byte: remaining.byte_position(),
                found_char: remaining.peek(),
            }, "Number is outside the finite f64 range")),
        }
    })
}
