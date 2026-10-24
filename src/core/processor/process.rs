use crate::core::ir::Node;
use crate::core::syntax::document::{SyntaxDocument, SyntaxNode};
use crate::core::syntax::identifier;
use crate::parsec::{ErrorLocation, Parser, ParserInput};

use super::action::ActionContext;
use super::builder::IrBuilder;
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
        let mut builder = IrBuilder::new();
        let mut inferred_close: Option<InferredClose<'_>> = None;

        for node in document.nodes {
            if let SyntaxNode::ImplicitEnd { span, opening_span } = &node {
                // Inline and explicitly closed candidates have no remaining block.
                if builder.contains_opening(opening_span) {
                    let location = ErrorLocation {
                        start_byte: span.start,
                        failure_byte: span.start,
                        found_char: None,
                    };

                    if builder.current().is_some_and(|opening| opening.span != *opening_span) {
                        return Err(ProcessError::new(location, ProcessErrorKind::CrossedImplicitEnd));
                    }

                    if let Some(opening) = builder.close_block(span.start) {
                        opening.directive.end(&mut self.context)?;
                        self.context.types.pop_scope();
                        inferred_close = Some(InferredClose { position: span.start, name: opening.name });
                    }
                }

                continue;
            }

            if let SyntaxNode::Text { span, content } = &node {
                builder.push(Node::Text { span: span.clone(), content });
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

            let mut action_context = ActionContext {
                span,
                name,
                location: name_location,
                directive: directive.as_ref(),
                inferred_close: inferred_close_at_start.map(|closing| closing.name),
                context: &mut self.context,
                builder: &mut builder,
            };

            action_context.apply(action)?;
        }

        builder.finish()
    }
}

struct InferredClose<'a> {
    position: usize,
    name: &'a str,
}
