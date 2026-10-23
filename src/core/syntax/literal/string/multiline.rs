use std::borrow::Cow;
use std::ops::Range;

use crate::parsec::prelude::*;
use crate::parsec::{line_break, ParseError};

use super::decode_string;

#[cfg(test)]
mod tests;

struct StringLine<'a> {
    indent: &'a str,
    /// The full line before its line_break, including indentation.
    text: &'a str,
    line_break: &'a str,
    span: Range<usize>,
}

/// Removes framing line breaks and common indentation, preserving relative layout.
pub(crate) fn multiline_value(content: &str, start_byte: usize, raw: bool) -> Result<Cow<'_, str>, Box<dyn ParseError>> {
    let (mut lines, _) = lines().parse(ParserInput::with_offset(content, start_byte))?;
    let indent = common_indent(&lines);

    // The last line line_break belongs to the closing delimiter's separate line.
    if let Some(last) = lines.last_mut() {
        last.line_break = "";
    }

    render_lines(&lines, indent, raw).map(Cow::Owned)
}

/// E.g. `"\n  foo\n  "` yields one line; delimiter-line whitespace is discarded.
fn lines<'a>() -> impl Parser<'a, Output = Vec<StringLine<'a>>> {
    map(
        seq!(
            // Discard whitespace after the opening quote and its framing line break.
            void(seq(opt(horizontal_whitespace()), line_break())),
            repeat(physical_line(), 0..),
            opt(horizontal_whitespace()),
            input_end()
        ),

        |((((), lines), _), ())| lines
    )
}

/// E.g. `"  foo\r\n"` yields indent `"  "`, text `"  foo"`, and line_break `"\r\n"`.
fn physical_line<'a>() -> impl Parser<'a, Output = StringLine<'a>> {
    flat_map(
        seq(
            spanned(seq(
                recognize(opt(horizontal_whitespace())),
                take_match(not(line_break()), 0..)
            )),

            line_break()
        ),

        |((span, (indent, _)), line_break), input| {
            let text = input.source_slice(span.clone());
            Ok((StringLine { indent, text, line_break, span }, input))
        }
    )
}

/// Blank lines do not determine indentation; tabs and spaces remain distinct characters.
fn common_indent<'a>(lines: &[StringLine<'a>]) -> &'a str {
    lines.iter()
        .filter(|line| line.text.len() != line.indent.len())
        .map(|line| line.indent)
        .reduce(|common, indent| {
            let length = common.bytes().zip(indent.bytes()).take_while(|(a, b)| a == b).count();
            &common[..length]
        })
        .unwrap_or("")
}

/// A line join is a backslash followed only by spaces or tabs before the physical line ends.
/// E.g. `"\\ \t"` succeeds; `"\\ text"` fails. Input excludes the line line_break.
fn line_join<'a>() -> impl Parser<'a, Output = ()> {
    void(seq!(
        tag("\\"),
        opt(horizontal_whitespace()),
        input_end()
    ))
}

/// Escaped backslashes stay in the text; only an unescaped trailing backslash joins lines.
/// E.g. `"foo \\"` yields `("foo ", true)`; `"foo"` yields `("foo", false)`.
fn line_content<'a>() -> impl Parser<'a, Output = (&'a str, bool)> {
    map(
        seq!(
            take(|ch, escaped, input| {
                ch != '\\' || escaped || line_join().parse(input.clone()).is_err()
            }, 0..),
            opt(line_join()),
            input_end()
        ),

        |((text, line_join), ())| (text, line_join.is_some())
    )
}

/// Removes common indentation, decodes escapes and applies explicit line joins; raw mode preserves escapes and line breaks.
fn render_lines(lines: &[StringLine<'_>], indent: &str, raw: bool) -> Result<String, Box<dyn ParseError>> {
    let (value, _) = lines.iter().try_fold(
        (String::new(), false),
        |(mut value, joining_previous_line), line| -> Result<_, Box<dyn ParseError>> {
            let mut text = if line.text.len() == line.indent.len() {
                ""

            } else {
                &line.text[indent.len()..]
            };

            // When joining the previous line, skip blank lines and the next line's remaining indentation.
            if joining_previous_line {
                text = text.trim_start_matches([' ', '\t']);

                if text.is_empty() {
                    return Ok((value, true));
                }
            }

            let start_byte = line.span.end - text.len();

            let (text, joins_next_line) = if raw {
                (text, false)

            } else {
                let (parts, _) = line_content().parse(ParserInput::with_offset(text, start_byte))?;
                parts
            };

            if raw {
                value.push_str(text);

            } else {
                value.push_str(&decode_string(text, start_byte)?);
            }

            if !joins_next_line {
                value.push_str(line.line_break);
            }

            Ok((value, joins_next_line))
        }
    )?;

    Ok(value)
}
