use std::cell::RefCell;

use crate::parsec::prelude::*;
use crate::parsec::{ErrorLocation};

use super::name::directive_name;
use super::{DirectiveCandidate, DirectiveForm};

mod err;
use err::JadeLikeBodyError;

#[cfg(test)]
mod tests;

/// Reads a marker and its separator, returning whether extended syntax is used.
/// The caller handles line boundaries and indentation.
pub(crate) fn jade_like_directive_start<'a>() -> impl Parser<'a, Output = bool> {
    map(
        seq(
            choice(tag("-"), tag("#")),
            take(|ch, _, _| ch == ' ' || ch == '\t', 1..),
        ),

        |(marker, _)| marker == "#",
    )
}

/// Parses one JadeLike directive, leaving its line ending unconsumed.
/// Supports balanced delimiters, quoted strings and explicit continuation.
/// Indentation is supplied by the caller and is not consumed here.
pub(crate) fn jade_like_directive_candidate<'a>(indent: &'a str) -> impl Parser<'a, Output = DirectiveCandidate<'a>> {
    let parser = seq(
        jade_like_directive_start(),
        body(),
    );

    flat_map(parser, move |(extended, content), input| {
        let (name, remaining) = opt(directive_name()).parse(ParserInput::new(content))?;
        let body = &content[remaining.byte_position()..];

        Ok((DirectiveCandidate {
            form: DirectiveForm::JadeLike { indent },
            extended,
            name,
            body,
            content,
        }, input))
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

    while matches!(remaining.peek(), Some(' ' | '\t')) {
        remaining.next();
    }

    matches!(remaining.peek(), None | Some('\n' | '\r'))
}
