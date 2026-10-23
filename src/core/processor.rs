use std::collections::hash_map::{Entry, HashMap};

use crate::parsec::{ErrorLocation, ParseError, ParserInput};

use super::directives::{Directive, DirectiveContext};
use super::syntax::document::{SyntaxDocument, SyntaxNode};

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
        match self.directives.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert(directive);
                Ok(())
            }

            Entry::Occupied(_) => Err(RegistrationError { name: name.into() }),
        }
    }

    fn process(&mut self, document: SyntaxDocument<'a>) -> ProcessResult {
        for node in document.nodes {
            let SyntaxNode::Directive { span, candidate } = node else {
                continue;
            };

            let content_start = span.start + candidate.opening_len;
            let content_end = span.end - candidate.closing_len;

            let name_location = ErrorLocation {
                start_byte: span.start,
                failure_byte: content_start,
                found_char: candidate.content.chars().next(),
            };

            let Some(name) = candidate.name else {
                return Err(ProcessError::new(name_location, ProcessErrorKind::UnresolvedDirective));
            };

            let Some(directive) = self.directives.get(name) else {
                return Err(ProcessError::new(
                    name_location,
                    ProcessErrorKind::UnknownDirective { name: name.into() }
                ));
            };

            let body_start = content_end - candidate.body.len();

            if directive.definition().requires_body && candidate.body.trim().is_empty() {
                return Err(ProcessError::new(
                    ErrorLocation {
                        start_byte: span.start,
                        failure_byte: body_start,
                        found_char: candidate.body.chars().next(),
                    },

                    ProcessErrorKind::MissingBody { name: name.into() }
                ));
            }

            let body = ParserInput::with_offset(candidate.body, body_start);
            directive.begin(&mut self.context, body)?;
        }

        Ok(())
    }
}

type ProcessResult = Result<(), Box<dyn ParseError>>;
