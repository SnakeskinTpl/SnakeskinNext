use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::ops::Range;

use crate::core::shorthands::ShorthandBinding;
use crate::core::syntax::identifier;
use crate::core::syntax::json::{json, JsonKind, JsonValue};

use crate::parsec::prelude::*;
use crate::parsec::{ErrorLocation, ParseError};

use super::{Directive, DirectiveAction, DirectiveBeginResult, DirectiveContext, DirectiveDefinition};

mod err;
use err::{DuplicateShorthandError, ShorthandConfigError};

#[cfg(test)]
mod tests;

pub(crate) struct Shorthand {
    definition: DirectiveDefinition<'static>,
}

impl Shorthand {
    pub fn new() -> Self {
        Self {
            definition: DirectiveDefinition {
                name: "shorthand",
                requires_body: true,
                ..Default::default()
            },
        }
    }
}

impl Directive for Shorthand {
    fn definition(&self) -> &DirectiveDefinition<'_> {
        &self.definition
    }

    fn begin<'a>(
        &self,
        context: &mut DirectiveContext<'a>,
        body: ParserInput<'a>,
    ) -> DirectiveBeginResult<'a> {
        let (binding, remaining) = declaration().parse(body)?;

        // Register only after the complete declaration has been validated.
        match context.shorthands.entry(binding.opening.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(binding);
                Ok(DirectiveAction::None)
            }

            Entry::Occupied(entry) => Err(Box::new(DuplicateShorthandError {
                location: ErrorLocation {
                    start_byte: binding.span.start,
                    failure_byte: binding.opening_span.start,
                    found_char: remaining.source_slice(binding.opening_span.clone()).chars().next(),
                },

                previous_span: entry.get().span.clone(),
            })),
        }
    }
}

/// Parses target { opening: "...", closing: "..." }; closing is optional.
fn declaration<'a>() -> impl Parser<'a, Output = ShorthandBinding<'a>> {
    let body_syntax = seq!(
        ltrim(identifier()),
        ltrim1(json()),
        ltrim(input_end())
    );

    flat_map(spanned(body_syntax), |(span, ((target, options), ())), remaining| {
        let options_span = options.span.clone();

        let JsonKind::Object(properties) = options.kind else {
            return Err(config_error(&remaining, options_span, "Expected a shorthand options object"));
        };

        let mut opening = None;
        let mut closing = None;

        for property in properties {
            let slot = match property.key.as_ref() {
                "opening" => &mut opening,
                "closing" => &mut closing,
                _ => return Err(config_error(&remaining, property.key_span, "Unknown shorthand option")),
            };

            if slot.is_some() {
                return Err(config_error(&remaining, property.key_span, "Duplicate shorthand option"));
            }

            let value_span = property.value.span.clone();
            let value = delimiter(property.value, &remaining)?;
            *slot = Some((value, value_span));
        }

        let Some((opening, opening_span)) = opening else {
            return Err(config_error(&remaining, options_span, "Missing opening shorthand option"));
        };

        Ok((ShorthandBinding {
            opening,
            closing: closing.map(|(value, _)| value),
            target,
            span,
            opening_span,
        }, remaining))
    })
}

fn delimiter<'a>(value: JsonValue<'a>, input: &ParserInput<'_>) -> Result<Cow<'a, str>, Box<dyn ParseError>> {
    match value.kind {
        JsonKind::String(text) if !text.is_empty() => Ok(text),
        _ => Err(config_error(input, value.span, "Shorthand delimiter must be a nonempty string")),
    }
}

fn config_error(input: &ParserInput<'_>, span: Range<usize>, message: &'static str) -> Box<dyn ParseError> {
    Box::new(ShorthandConfigError {
        location: ErrorLocation {
            start_byte: span.start,
            failure_byte: span.start,
            found_char: input.source_slice(span).chars().next(),
        },
        message,
    })
}
