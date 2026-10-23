use std::ops::Range;

use crate::core::directives::DirectiveAction;
use crate::core::ir::Node;
use crate::core::syntax::expression::Expression;
use crate::core::types::TypeEnvironment;
use crate::parsec::{ErrorLocation, ParseError};

use super::blocks::{push_output, OpenDirective};
use super::err::{ProcessError, ProcessErrorKind};

/// Only the state needed to apply an action; registry access stays in the processor.
pub(crate) struct ActionContext<'borrow, 'directive, 'source> {
    pub span: Range<usize>,
    pub types: &'borrow mut TypeEnvironment<'source>,
    pub open_directives: &'borrow mut [OpenDirective<'directive, 'source>],
    pub nodes: &'borrow mut Vec<Node<'source>>,
}

pub(crate) enum ActionOutcome<'a> {
    Continue,
    Conditional { condition: Expression<'a> },
    CloseBlock { name: Option<&'a str> },
}

impl<'a> DirectiveAction<'a> {
    /// Applies immediate effects; block opening and closing remain with the document traversal.
    pub fn apply(self, context: &mut ActionContext<'_, '_, 'a>) -> Result<ActionOutcome<'a>, Box<dyn ParseError>> {
        match self {
            Self::Conditional { condition } => Ok(ActionOutcome::Conditional { condition }),
            Self::CloseBlock { name } => Ok(ActionOutcome::CloseBlock { name }),
            Self::None => Ok(ActionOutcome::Continue),
            Self::Declare { declaration } => {
                if !context.types.declare(declaration.name, declaration.value_info.ty, declaration.mutable) {
                    return Err(ProcessError::new(
                        ErrorLocation {
                            start_byte: context.span.start,
                            failure_byte: declaration.name_span.start,
                            found_char: declaration.name.chars().next(),
                        },
                        ProcessErrorKind::DuplicateVariable { name: declaration.name.into() }
                    ));
                }

                push_output(
                    Node::Declaration { span: context.span.clone(), declaration },
                    context.open_directives,
                    context.nodes
                );

                Ok(ActionOutcome::Continue)
            }
        }
    }
}
