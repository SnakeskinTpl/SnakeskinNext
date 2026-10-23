use std::cell::RefCell;

use crate::parsec::prelude::*;
use crate::core::shorthands::ShorthandRegistry;

use super::name::directive_content;
use super::{DirectiveCandidate, DirectiveForm, DirectiveMode};

#[cfg(test)]
mod tests;

/// Consumes the opening marker only; content and closing braces are not checked.
/// Returns the directive mode.
pub(crate) fn classic_directive_start<'a>(
    shorthands: Option<&ShorthandRegistry<'_>>,
) -> impl Parser<'a, Output = DirectiveMode> {
    let builtin = map(seq(opt(tag("#")), tag("{")), |(hash, _)| {
        match hash {
            Some(_) => DirectiveMode::Extended,
            None => DirectiveMode::Regular,
        }
    });

    parser(move |input| {
        if let Some(binding) = shorthands.and_then(|registry| registry.find_classic_opening(&input)) {
            return value(tag(binding.opening.as_ref()), DirectiveMode::Regular).parse(input);
        }

        builtin.parse(input)
    })
}

pub(crate) fn classic_directive_candidate<'a>(
    shorthands: Option<&ShorthandRegistry<'a>>,
) -> impl Parser<'a, Output = DirectiveCandidate<'a>> {
    parser(move |input| {
        let binding = shorthands.and_then(|registry| registry.find_classic_opening(&input));

        let closing = binding
            .and_then(|binding| binding.closing.as_deref())
            .unwrap_or("}");

        let content = flat_map(spanned(body(1, closing)), |(span, _), input| {
            let content = input.source_slice(span);

            let (name, body) = if let Some(binding) = binding {
                (Some(binding.target), content)

            } else {
                let (parts, _) = directive_content(shorthands).parse(ParserInput::new(content))?;
                parts
            };

            Ok((DirectiveCandidate {
                form: DirectiveForm::Classic,
                mode: DirectiveMode::Regular,
                opening_len: 0,
                closing_len: 0,
                name,
                body,
                content,
            }, input))
        });

        map(
            seq!(spanned(classic_directive_start(shorthands)), content, tag(closing)),
            |(((opening_span, mode), mut directive), closing)| {
                directive.mode = mode;
                directive.opening_len = opening_span.len();
                directive.closing_len = closing.len();

                directive
            },
        ).parse(input)
    })
}

/// Leaves the closing delimiter unconsumed outside balanced brackets and quotes.
fn body<'a>(min: usize, closing: &str) -> impl Parser<'a, Output = &'a str> {
    state(|| RefCell::new(Vec::new()), move |stack, input| {
        take(|ch, escaped, input| {
            if escaped {
                return true;
            }

            let mut stack = stack.borrow_mut();
            if let Some(&quote @ ('"' | '\'' | '`')) = stack.last() {
                if ch == quote {
                    stack.pop();
                }
                return true;
            }

            if stack.is_empty() && tag(closing).parse(input.clone()).is_ok() {
                return false;
            }

            match ch {
                '{' => stack.push('}'),
                '[' => stack.push(']'),
                '(' => stack.push(')'),
                '"' | '\'' | '`' => stack.push(ch),
                '}' | ']' | ')' if stack.last() == Some(&ch) => { stack.pop(); }
                _ => {}
            }

            true
        }, min..).parse(input)
    })
}
