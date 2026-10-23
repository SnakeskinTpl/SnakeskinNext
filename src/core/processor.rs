use std::borrow::Cow;
use std::collections::hash_map::{Entry, HashMap};
use std::ops::Range;

use crate::parsec::{ErrorLocation, ParseError, Parser, ParserInput};

use super::directives::{Directive, DirectiveAction, DirectiveContext};
use super::syntax::document::{SyntaxDocument, SyntaxNode};
use super::syntax::identifier;
use super::shorthands::ShorthandBinding;

mod err;
use err::{ProcessError, ProcessErrorKind, RegistrationError};

#[cfg(test)]
mod tests;

struct Processor<'a> {
    directives: HashMap<&'a str, Box<dyn Directive + 'a>>,
    context: DirectiveContext<'a>,
}

impl<'a> Processor<'a> {
    fn new() -> Self {
        Self {
            directives: HashMap::new(),
            context: DirectiveContext::default(),
        }
    }

    fn register_directive(
        &mut self,
        name: &'a str,
        directive: Box<dyn Directive + 'a>,
    ) -> Result<(), RegistrationError> {
        if self.directives.contains_key(name) {
            return Err(RegistrationError { name: name.into(), reason: "Directive is already registered" });
        }

        let shorthands = &directive.definition().shorthands;

        for (index, shorthand) in shorthands.iter().enumerate() {
            if shorthand.opening.is_empty() || shorthand.closing == Some("") {
                return Err(RegistrationError { name: name.into(), reason: "Empty shorthand delimiter" });
            }

            if
                self.context.shorthands.get(shorthand.opening).is_some() ||
                shorthands[..index].iter().any(|previous| previous.opening == shorthand.opening)
            {
                return Err(RegistrationError { name: shorthand.opening.into(), reason: "Shorthand is already registered" });
            }
        }

        let bindings: Vec<_> = shorthands.iter().map(|definition| ShorthandBinding {
            opening: Cow::Owned(definition.opening.into()),
            closing: definition.closing.map(|closing| Cow::Owned(closing.into())),
            target: name,
            span: 0..0,
            opening_span: 0..0,
        }).collect();

        match self.directives.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert(directive);

                for binding in bindings {
                    self.context.shorthands.entry(binding.opening.clone()).or_insert(binding);
                }

                Ok(())
            }

            Entry::Occupied(_) => Err(RegistrationError { name: name.into(), reason: "Directive is already registered" }),
        }
    }

    fn process(&mut self, document: SyntaxDocument<'a>) -> ProcessResult {
        let mut open_directives: Vec<OpenDirective<'_>> = Vec::new();
        let mut inferred_close: Option<InferredClose<'_>> = None;

        for node in document.nodes {
            if let SyntaxNode::ImplicitEnd { span, opening_span } = &node {
                // Inline and explicitly closed candidates have no remaining block.
                if open_directives.iter().any(|opening| opening.span == *opening_span) {
                    let location = ErrorLocation {
                        start_byte: span.start,
                        failure_byte: span.start,
                        found_char: None,
                    };

                    if open_directives.last().is_some_and(|opening| opening.span != *opening_span) {
                        return Err(ProcessError::new(location, ProcessErrorKind::CrossedImplicitEnd));
                    }

                    if let Some(opening) = open_directives.pop() {
                        opening.directive.end(&mut self.context)?;
                        inferred_close = Some(InferredClose { position: span.start, name: opening.name });
                    }
                }

                continue;
            }

            let SyntaxNode::Directive { span, candidate } = node else {
                inferred_close = None;
                continue;
            };

            // An explicit end on this line may confirm its indentation-based close.
            let inferred_close_at_start = inferred_close.take()
                .filter(|closing| closing.position == span.start);

            let content_start = span.start + candidate.opening_len;
            let content_end = span.end - candidate.closing_len;

            let name_location = ErrorLocation {
                start_byte: span.start,
                failure_byte: content_start,
                found_char: candidate.content.chars().next(),
            };

            let bare_name = candidate.content.trim();

            let is_bare_name = candidate.name.is_none() &&
                self.directives.contains_key(bare_name) &&
                identifier().parse(ParserInput::new(bare_name))
                    .is_ok_and(|(_, remaining)| remaining.is_at_end());

            let name = candidate.name.or_else(|| is_bare_name.then_some(bare_name));

            let Some(name) = name else {
                return Err(ProcessError::new(name_location, ProcessErrorKind::UnresolvedDirective));
            };

            let Some(directive) = self.directives.get(name) else {
                return Err(ProcessError::new(
                    name_location,
                    ProcessErrorKind::UnknownDirective { name: name.into() }
                ));
            };

            let body_text = if is_bare_name { "" } else { candidate.body };

            let body_start = content_end - body_text.len();

            if directive.definition().requires_body && body_text.trim().is_empty() {
                return Err(ProcessError::new(
                    ErrorLocation {
                        start_byte: span.start,
                        failure_byte: body_start,
                        found_char: body_text.chars().next(),
                    },

                    ProcessErrorKind::MissingBody { name: name.into() }
                ));
            }

            let body = ParserInput::with_offset(body_text, body_start);
            let action = directive.begin(&mut self.context, body)?;

            if directive.definition().block {
                open_directives.push(OpenDirective {
                    name,
                    span,
                    directive: directive.as_ref(),
                    location: name_location
                });

            } else {
                directive.end(&mut self.context)?;
            }

            if let DirectiveAction::CloseBlock { name } = action {
                let expected_name = inferred_close_at_start.as_ref().map(|closing| closing.name)
                    .or_else(|| open_directives.last().map(|opening| opening.name));

                let Some(expected_name) = expected_name else {
                    return Err(ProcessError::new(name_location, ProcessErrorKind::UnexpectedEnd));
                };

                if let Some(name) = name {
                    if name != expected_name {
                        return Err(ProcessError::new(
                            name_location,
                            ProcessErrorKind::MismatchedEnd {
                                expected: expected_name.into(),
                                found: name.into(),
                            }
                        ));
                    }
                }

                if inferred_close_at_start.is_none() {
                    if let Some(opening) = open_directives.pop() {
                        opening.directive.end(&mut self.context)?;
                    }
                }
            }
        }

        if let Some(opening) = open_directives.last() {
            return Err(ProcessError::new(opening.location, ProcessErrorKind::UnclosedBlock { name: opening.name.into() }));
        }

        Ok(())
    }
}

type ProcessResult = Result<(), Box<dyn ParseError>>;

struct OpenDirective<'a> {
    name: &'a str,
    span: Range<usize>,
    directive: &'a dyn Directive,
    location: ErrorLocation,
}

struct InferredClose<'a> {
    position: usize,
    name: &'a str,
}
