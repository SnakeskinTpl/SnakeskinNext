use crate::core::syntax::expression::expression;
use crate::core::types::{infer, Type, TypeError, TypeErrorKind};

use crate::parsec::prelude::*;
use crate::parsec::{ErrorLocation, ParseError};

use super::{Directive, DirectiveAction, DirectiveBeginResult, DirectiveContext, DirectiveDefinition};

use super::err::DirectiveTypeError;

pub(crate) struct If {
    definition: DirectiveDefinition<'static>,
}

impl If {
    pub fn new() -> Self {
        Self {
            definition: DirectiveDefinition {
                name: "if",
                block: true,
                requires_body: true,
                allows_self_nesting: true,
                ..Default::default()
            },
        }
    }
}

impl Directive for If {
    fn definition(&self) -> &DirectiveDefinition<'_> {
        &self.definition
    }

    fn begin<'a>(&self, context: &mut DirectiveContext<'a>, body: ParserInput<'a>) -> DirectiveBeginResult<'a> {
        let source = body.clone();
        let ((condition, ()), _) = cut(seq(expression(), ltrim(input_end()))).parse(body)?;

        let checked = infer(&condition, &context.types).and_then(|value_info| {
            if value_info.ty == Type::Bool {
                Ok(())

            } else {
                Err(TypeError {
                    span: condition.span.clone(),
                    kind: TypeErrorKind::MismatchedType { expected: Type::Bool, found: value_info.ty },
                })
            }
        });

        checked.map_err(|cause| {
            let found_char = source.source_slice(cause.span.clone()).chars().next();
            
            Box::new(DirectiveTypeError {
                location: ErrorLocation {
                    start_byte: condition.span.start,
                    failure_byte: cause.span.start,
                    found_char,
                },
                cause,
            }) as Box<dyn ParseError>
        })?;

        Ok(DirectiveAction::Conditional { condition })
    }
}
