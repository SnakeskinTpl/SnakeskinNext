use crate::core::ir::{Node, ProcessedDocument};
use crate::core::syntax::document::{SyntaxDocument, SyntaxNode};
use crate::core::syntax::identifier;
use crate::parsec::{ErrorLocation, Parser, ParserInput};

use super::action::{ActionContext, ActionOutcome};
use super::blocks::{finish_output, push_output, InferredClose, OpenDirective};
use super::err::{ProcessError, ProcessErrorKind};
use super::{Processor, ProcessResult};

impl<'a> Processor<'a> {
    pub fn process(&mut self, document: SyntaxDocument<'a>) -> ProcessResult<'a> {
        // File-local bindings must not escape, including after an error.
        let previous_types = self.context.types.clone();
        let result = self.process_document(document);
        self.context.types = previous_types;
        result
    }

    fn process_document(&mut self, document: SyntaxDocument<'a>) -> ProcessResult<'a> {
        let mut open_directives: Vec<OpenDirective<'_, 'a>> = Vec::new();
        let mut nodes = Vec::new();
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
                        self.context.types.pop_scope();
                        inferred_close = Some(InferredClose { position: span.start, name: opening.name });
                        finish_output(opening, span.start, &mut open_directives, &mut nodes);
                    }
                }

                continue;
            }

            if let SyntaxNode::Text { span, content } = &node {
                push_output(Node::Text { span: span.clone(), content }, &mut open_directives, &mut nodes);
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

            let outcome = action.apply(&mut ActionContext {
                span: span.clone(),
                types: &mut self.context.types,
                open_directives: &mut open_directives,
                nodes: &mut nodes,
            })?;

            let closing_name = match outcome {
                ActionOutcome::CloseBlock { name } => Some(name),
                _ => None,
            };

            let condition = match outcome {
                ActionOutcome::Conditional { condition } => Some(condition),
                _ => None,
            };

            let directive_end = span.end;

            if directive.definition().block {
                self.context.types.push_scope();
                open_directives.push(OpenDirective {
                    name,
                    span,
                    directive: directive.as_ref(),
                    location: name_location,
                    condition,
                    children: Vec::new(),
                });

            } else {
                directive.end(&mut self.context)?;
            }

            if let Some(name) = closing_name {
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
                        self.context.types.pop_scope();
                        finish_output(opening, directive_end, &mut open_directives, &mut nodes);
                    }
                }
            }
        }

        if let Some(opening) = open_directives.last() {
            return Err(ProcessError::new(opening.location, ProcessErrorKind::UnclosedBlock { name: opening.name.into() }));
        }

        Ok(ProcessedDocument { nodes })
    }
}
