use std::cell::RefCell;

use crate::parsec::prelude::*;
use crate::parsec::{ErrorLocation};
use crate::core::shorthands::ShorthandRegistry;

use super::name::directive_content;
use super::{DirectiveCandidate, DirectiveForm, DirectiveMode};

mod err;
use err::JadeLikeBodyError;

#[cfg(test)]
mod tests;

/// Reads a marker and its separator, returning the directive mode.
/// The caller handles line boundaries and indentation.
pub(crate) fn jade_like_directive_start<'a>(
    shorthands: Option<&ShorthandRegistry<'_>>,
) -> impl Parser<'a, Output = DirectiveMode> {
    let marker = parser(move |input: ParserInput<'a>| {
        let binding = shorthands.and_then(|registry| registry.find_opening(&input));

        if let Some(binding) = binding {
            return value(tag(binding.opening.as_ref()), DirectiveMode::Regular).parse(input);
        }

        choice(
            value(tag("-"), DirectiveMode::Regular),
            value(tag("#"), DirectiveMode::Extended)
        ).parse(input)
    });

    map(
        seq(
            marker,
            hspace(),
        ),

        |(mode, _)| mode,
    )
}

/// Parses one JadeLike directive, leaving its line ending unconsumed.
/// Supports balanced delimiters, quoted strings and explicit continuation.
/// Indentation is supplied by the caller and is not consumed here.
pub(crate) fn jade_like_directive_candidate<'a>(
    indent: &'a str,
    shorthands: Option<&ShorthandRegistry<'a>>,
) -> impl Parser<'a, Output = DirectiveCandidate<'a>> {
    parser(move |input| {
        let binding = shorthands.and_then(|registry| registry.find_opening(&input));

        let parser = seq(
            spanned(jade_like_directive_start(shorthands)),
            body(),
        );

        flat_map(parser, move |((opening_span, mode), content), input| {
            let (name, body) = if let Some(binding) = binding {
                (Some(binding.target), content)

            } else {
                let (parts, _) = directive_content(shorthands).parse(ParserInput::new(content))?;
                parts
            };

            Ok((DirectiveCandidate {
                form: DirectiveForm::JadeLike { indent },
                mode,
                opening_len: indent.len() + opening_span.len(),
                closing_len: 0,
                name,
                body,
                content,
            }, input))
        }).parse(input)
    })
}

#[derive(Clone, Copy)]
enum Frame {
    Bracket(char),
    Quote { delimiter: char, multiline: bool },
    ExplicitContinuation,
}

impl Frame {
    fn closing_char(self) -> char {
        match self {
            Self::Bracket(delimiter) | Self::Quote { delimiter, .. } => delimiter,
            Self::ExplicitContinuation => '.',
        }
    }
}

#[derive(Default)]
struct BodyState {
    stack: Vec<Frame>,
    line_has_non_whitespace: bool,
}

/// Reads through open delimiters, preserving continuation markers and line endings.
/// A quote opened at the end of a line must close on a standalone line.
fn body<'a>() -> impl Parser<'a, Output = &'a str> {
    state(|| RefCell::new(BodyState::default()), |scan, input| {
        let start_byte = input.byte_position();

        let parser = try_take(|ch, escaped, input| {
            let mut scan = scan.borrow_mut();
            let newline = ch == '\n' || ch == '\r';

            if newline && scan.stack.is_empty() {
                return Ok(false);
            }

            if !escaped && ch != '\\' {
                let top = scan.stack.last().copied();

                if let Some(Frame::Quote { delimiter, multiline }) = top {
                    if ch == delimiter && (!multiline || (!scan.line_has_non_whitespace && ends_line(input))) {
                        scan.stack.pop();
                    }

                } else {
                    match ch {
                        '"' | '\'' | '`' => scan.stack.push(Frame::Quote { delimiter: ch, multiline: ends_line(input) }),

                        '{' => scan.stack.push(Frame::Bracket('}')),
                        '[' => scan.stack.push(Frame::Bracket(']')),
                        '(' => scan.stack.push(Frame::Bracket(')')),

                        '}' | ']' | ')' => {
                            if !matches!(top, Some(Frame::Bracket(expected)) if expected == ch) {
                                return Err(JadeLikeBodyError::new(
                                    ErrorLocation { start_byte, failure_byte: input.byte_position(), found_char: Some(ch) },
                                    match top {
                                        Some(Frame::Bracket(expected)) => Some(expected),
                                        _ => None,
                                    },
                                ));
                            }
                            scan.stack.pop();
                        }

                        '&' if scan.stack.is_empty() && ends_line(input) => {
                            scan.stack.push(Frame::ExplicitContinuation);
                        }

                        '.' if matches!(top, Some(Frame::ExplicitContinuation)) && !scan.line_has_non_whitespace && ends_line(input) => {
                            scan.stack.pop();
                        }

                        _ => {}
                    }
                }
            }

            if newline {
                scan.line_has_non_whitespace = false;

            } else if ch != ' ' && ch != '\t' {
                scan.line_has_non_whitespace = true;
            }

            Ok(true)
        }, 1..);

        let (content, input) = parser.parse(input)?;
        let scan = scan.borrow();

        let expected = scan.stack.last().copied().map(Frame::closing_char);

        if let Some(expected) = expected {
            return Err(JadeLikeBodyError::new(
                ErrorLocation { start_byte, failure_byte: input.byte_position(), found_char: input.peek() },
                Some(expected),
            ));
        }

        Ok((content, input))
    })
}

/// Checks whether only horizontal whitespace follows the current character.
fn ends_line(input: &ParserInput<'_>) -> bool {
    let mut remaining = input.clone();
    remaining.next();

    seq(
        opt(hspace()),
        choice(
            void(choice(tag("\n"), tag("\r"))),
            input_end()
        )
    ).parse(remaining).is_ok()
}
