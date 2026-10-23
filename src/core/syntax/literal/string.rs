use std::borrow::Cow;
use std::cell::Cell;

use crate::parsec::{line_break, line_end};
use crate::parsec::prelude::*;

mod decode;
use decode::decode_string;

mod multiline;
use multiline::multiline_value;

mod err;

#[cfg(test)]
mod tests;

/// Reads quoted strings, optional raw tag `r`, and standalone multiline quotes.
pub(crate) fn string<'a>() -> impl Parser<'a, Output = Cow<'a, str>> {
    flat_map(
        seq(opt(tag("r")), choice(tag("\""), tag("'"))),
        |(raw, quote), input| {
            let raw = raw.is_some();
            let start_byte = input.byte_position();

            // An opening quote followed only by spaces or tabs and a newline starts multiline mode.
            let (multiline, _) = opt(peek(seq(opt(hspace()), line_break())))
                .parse(input.clone())?;

            let multiline = multiline.is_some();

            let content = state(
                || Cell::new(false),
                |line_has_content, input| {
                    take(
                        |ch, escaped, input| {
                            let closes = quote.starts_with(ch) && (raw || !escaped);

                            if closes {
                                if !multiline {
                                    return false;
                                }

                                // A multiline closing quote must be alone on its line, apart from indentation.
                                let standalone_end = seq!(
                                    tag(quote),
                                    opt(hspace()),
                                    line_end()
                                );

                                // Check both sides of the quote without consuming its trailing line break.
                                if !line_has_content.get() &&
                                    standalone_end.parse(input.clone()).is_ok()
                                {
                                    return false;
                                }
                            }

                            if ch == '\n' {
                                line_has_content.set(false);

                            } else if ch != ' ' && ch != '\t' && ch != '\r' {
                                line_has_content.set(true);
                            }

                            true
                        },
                        0..,
                    )
                    .parse(input)
                },
            );

            let ((content, _), remaining) = cut(seq(content, tag(quote))).parse(input)?;

            let value = if multiline {
                multiline_value(content, start_byte, raw)?

            } else if raw {
                Cow::Borrowed(content)

            } else {
                decode_string(content, start_byte)?
            };

            Ok((value, remaining))
        },
    )
}
