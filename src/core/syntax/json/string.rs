use std::borrow::Cow;

use crate::parsec::{cut, flat_map, spanned, tag, take, ErrorLocation, ParseError, Parser};
use crate::seq;

use super::err::JsonError;

pub(crate) fn string<'a>() -> impl Parser<'a, Output = Cow<'a, str>> {
    let grammar = seq!(
        tag("\""),
        cut(seq!(
            take(|ch, escaped, _| escaped || ch != '"', 0..),
            tag("\"")
        ))
    );

    flat_map(spanned(grammar), |(span, (_, (raw, _))), input| {
        let value = decode(raw, span.start + 1)?;

        Ok((value, input))
    })
}

/// Decode escapes separately from finding the closing quote; plain strings stay borrowed.
fn decode(raw: &str, start_byte: usize) -> Result<Cow<'_, str>, Box<dyn ParseError>> {
    let mut chars = raw.char_indices();
    let mut decoded = String::new();
    let has_escapes = raw.contains('\\');

    while let Some((offset, ch)) = chars.next() {
        let failure_byte = start_byte + offset;

        let error = |message| -> Box<dyn ParseError> { JsonError::new(ErrorLocation {
            start_byte: start_byte - 1,
            failure_byte,
            found_char: Some(ch),
        }, message) };

        if ch <= '\u{1f}' {
            return Err(error("Unescaped control character in string"));
        }

        if ch != '\\' {
            if has_escapes {
                decoded.push(ch);
            }

            continue;
        }

        let escaped = match chars.next() {
            Some((_, '"')) => '"',
            Some((_, '\\')) => '\\',
            Some((_, '/')) => '/',
            Some((_, 'b')) => '\u{8}',
            Some((_, 'f')) => '\u{c}',
            Some((_, 'n')) => '\n',
            Some((_, 'r')) => '\r',
            Some((_, 't')) => '\t',
            Some((_, 'u')) => {
                let first = hex_quad(&mut chars)
                    .ok_or_else(|| error("Expected four hexadecimal digits after \\u"))?;

                let scalar = if (0xd800..=0xdbff).contains(&first) {
                    if !matches!(chars.next(), Some((_, '\\')))
                        || !matches!(chars.next(), Some((_, 'u'))) {
                        return Err(error("Expected a low surrogate after a high surrogate"));
                    }

                    let second = hex_quad(&mut chars)
                        .filter(|value| (0xdc00..=0xdfff).contains(value))
                        .ok_or_else(|| error("Invalid low surrogate"))?;

                    0x10000 + ((first - 0xd800) << 10) + second - 0xdc00

                } else {
                    first
                };

                char::from_u32(scalar).ok_or_else(|| error("Invalid Unicode escape"))?
            }
            _ => return Err(error("Invalid string escape")),
        };

        decoded.push(escaped);
    }

    Ok(if has_escapes { Cow::Owned(decoded) } else { Cow::Borrowed(raw) })
}

fn hex_quad(chars: &mut std::str::CharIndices<'_>) -> Option<u32> {
    let mut value = 0;

    for _ in 0..4 {
        let (_, ch) = chars.next()?;
        value = value * 16 + ch.to_digit(16)?;
    }

    Some(value)
}
