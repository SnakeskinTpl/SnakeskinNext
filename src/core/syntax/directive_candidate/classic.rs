use std::cell::Cell;

use crate::parsec::{choice, map, opt, flat_map, seq, spanned, state, tag, take, Parser};
use crate::seq;

use super::name::directive_name;
use super::{DirectiveCandidate, DirectiveForm};

#[cfg(test)]
mod tests;

/// Consumes the opening marker only; content and closing braces are not checked.
/// Returns whether extended syntax is used.
pub(crate) fn classic_directive_start<'a>() -> impl Parser<'a, Output = bool> {
    map(seq(opt(tag("#")), tag("{")), |(hash, _)| hash.is_some())
}

pub(crate) fn classic_directive_candidate<'a>() -> impl Parser<'a, Output = DirectiveCandidate<'a>> {
    let content = choice(
        map(seq(directive_name(), body(0)), |(name, body)| (Some(name), body)),
        map(body(1), |body| (None, body))
    );
    let content = flat_map(spanned(content), |(span, (name, body)), input| {
        let content = input.source_slice(span);

        Ok((DirectiveCandidate { form: DirectiveForm::Classic, extended: false, name, body, content }, input))
    });

    map(
        seq!(classic_directive_start(), content, tag("}")),
        |((extended, mut directive), _)| {
            directive.extended = extended;
            directive
        },
    )
}

/// Leaves the outer closing brace for the enclosing parser.
fn body<'a>(min: usize) -> impl Parser<'a, Output = &'a str> {
    state(|| Cell::new(0usize), move |depth, input| {
        take(|ch, escaped, _| {
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
