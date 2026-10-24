use std::ops::Range;

use crate::core::directives::{Directive, DirectiveAction, DirectiveContext};
use crate::core::ir::{BlockKind, Node};
use crate::parsec::{ErrorLocation, ParseError};

use super::builder::{IrBuilder, OpenDirective};
use super::err::{ProcessError, ProcessErrorKind};
use super::blocks::Blocks;

/// State needed to apply an action; registry lookup remains in the document traversal.
pub(crate) struct ActionContext<'borrow, 'directive, 'source> {
    pub span: Range<usize>,
    pub name: &'directive str,
    pub location: ErrorLocation,
    pub directive: &'directive dyn Directive,
    /// A close already performed by indentation on this line.
    pub inferred_close: Option<&'directive str>,
    pub context: &'borrow mut DirectiveContext<'source>,
    pub builder: &'borrow mut IrBuilder<'directive, 'source>,
}

impl<'directive, 'source> ActionContext<'_, 'directive, 'source> {
    /// Applies IR, scope and handler effects without an intermediate action representation.
    pub fn apply(&mut self, action: DirectiveAction<'source>) -> Result<(), Box<dyn ParseError>> {
        // The declaration controls block lifetime; actions must respect that contract.
        let invalid = match &action {
            DirectiveAction::OpenBlock { .. } if !self.directive.definition().block => {
                Some("An inline directive cannot open a block")
            }

            DirectiveAction::CloseBlock { .. } if self.directive.definition().block => {
                Some("A block directive cannot act as an end directive")
            }

            _ => None,
        };

        if let Some(reason) = invalid {
            return Err(ProcessError::new(
                self.location,
                ProcessErrorKind::InvalidDirectiveAction { name: self.name.into(), reason }
            ));
        }

        match action {
            DirectiveAction::OpenBlock { kind } => self.complete(Some(kind)),

            DirectiveAction::CloseBlock { name } => {
                self.complete(None)?;
                self.close(name)
            }

            DirectiveAction::None => self.complete(None),

            DirectiveAction::Declare { declaration } => {
                if !self.context.types.declare(declaration.name, declaration.value_info.ty, declaration.mutable) {
                    return Err(ProcessError::new(
                        ErrorLocation {
                            start_byte: self.span.start,
                            failure_byte: declaration.name_span.start,
                            found_char: declaration.name.chars().next(),
                        },

                        ProcessErrorKind::DuplicateVariable { name: declaration.name.into() }
                    ));
                }

                self.builder.push(Node::Declaration {
                    span: self.span.clone(),
                    declaration,
                });

                self.complete(None)
            }
        }
    }

    /// Block handlers finish on close; inline handlers finish immediately.
    fn complete(&mut self, block: Option<BlockKind<'source>>) -> Result<(), Box<dyn ParseError>> {
        if self.directive.definition().block {
            Blocks::open(self.context, self.builder, OpenDirective {
                name: self.name,
                span: self.span.clone(),
                directive: self.directive,
                location: self.location,
                block,
            });

            Ok(())

        } else {
            self.directive.end(self.context)
        }
    }

    fn close(&mut self, name: Option<&str>) -> Result<(), Box<dyn ParseError>> {
        let expected_name = self.inferred_close
            .or_else(|| self.builder.current().map(|opening| opening.name));

        let Some(expected_name) = expected_name else {
            return Err(ProcessError::new(self.location, ProcessErrorKind::UnexpectedEnd));
        };

        if let Some(name) = name {
            if name != expected_name {
                return Err(ProcessError::new(
                    self.location,
                    ProcessErrorKind::MismatchedEnd {
                        expected: expected_name.into(),
                        found: name.into(),
                    }
                ));
            }
        }

        if self.inferred_close.is_none() {
            Blocks::close(self.context, self.builder, self.span.end)?;
        }

        Ok(())
    }
}
