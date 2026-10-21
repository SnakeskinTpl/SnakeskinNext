use std::cell::Cell;

use crate::parsec::{choice, flat_map, map, opt, seq, state, tag, take, Parser};
use crate::seq;

#[cfg(test)]
mod tests;

/// A syntactic candidate; name and body remain tentative until directive resolution.
#[derive(Debug)]
pub(super) struct DirectiveCandidate<'a> {
    name: Option<&'a str>,
    body: &'a str,
    /// Original content between braces, before name or shorthand resolution.
    content: &'a str,
}

/// Consumes the opening marker only; content and closing braces are not checked.
pub(super) fn classic_directive_start<'a>() -> impl Parser<'a, Output = ()> {
    map(seq(opt(tag("#")), tag("{")), |_| ())
}

pub(super) fn classic_directive_candidate<'a>() -> impl Parser<'a, Output = DirectiveCandidate<'a>> {
    let content = state(|| (), |_, input| {
        let start = input.byte_position();
        let parser = choice(
            map(seq(directive_name(), directive_body(0)), |(name, body)| (Some(name), body)),
            map(directive_body(1), |body| (None, body)),
        );

        let ((name, body), remaining) = parser.parse(input)?;
        let content = remaining.source_slice(start..remaining.byte_position());

        Ok((DirectiveCandidate { name, body, content }, remaining))
    });

    map(
        seq!(classic_directive_start(), content, tag("}")),
        |((_, directive), _)| directive,
    )
}

/// Parses an identifier followed by a required whitespace separator.
fn directive_name<'a>() -> impl Parser<'a, Output = &'a str> {
    let name = seq!(
        // First character: one letter or underscore, e.g. 'f' in "foo2".
        take(|ch, _| ch.is_alphabetic() || ch == '_', 1..=1),
        // Remaining characters: letters, digits or underscores, e.g. "oo_2" in "foo_2".
        take(|ch, _| ch.is_alphanumeric() || ch == '_', 0..),
        // Required separator: whitespace, e.g. a space in "foo value" or a newline.
        take(|ch, _| ch.is_whitespace(), 1..),
    );

    flat_map(name, |((first, rest), separator), input| {
        let end = input.byte_position() - separator.len();
        let start = end - first.len() - rest.len();

        Ok((input.source_slice(start..end), input))
    })
}

/// Leaves the outer closing brace for the enclosing parser.
fn directive_body<'a>(min: usize) -> impl Parser<'a, Output = &'a str> {
    state(|| Cell::new(0usize), move |depth, input| {
        take(|ch, escaped| {
            if escaped {
                return true;
            }

            match ch {
                '{' => {
                    depth.set(depth.get() + 1);
                    true
                }
                '}' if depth.get() == 0 => false,
                '}' => {
                    depth.set(depth.get() - 1);
                    true
                }
                _ => true,
            }
        }, min..).parse(input)
    })
}
