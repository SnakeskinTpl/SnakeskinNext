use crate::core::ir::VariableDeclaration;
use crate::core::syntax::expression::expression;
use crate::core::syntax::identifier;
use crate::core::types::infer;

use crate::parsec::prelude::*;
use crate::parsec::{ErrorLocation, ParseError};

use super::err::DirectiveTypeError;
use super::{Directive, DirectiveAction, DirectiveBeginResult, DirectiveContext, DirectiveDefinition};

pub(crate) struct Let {
    definition: DirectiveDefinition<'static>,
}

pub(crate) struct Var {
    definition: DirectiveDefinition<'static>,
}

impl Let {
    pub fn new() -> Self {
        Self {
            definition: DirectiveDefinition {
                name: "let",
                requires_body: true,
                ..Default::default()
            }
        }
    }
}

impl Var {
    pub fn new() -> Self {
        Self {
            definition: DirectiveDefinition {
                name: "var",
                requires_body: true,
                ..Default::default()
            }
        }
    }
}

impl Directive for Let {
    fn definition(&self) -> &DirectiveDefinition<'_> { &self.definition }

    fn begin<'a>(&self, context: &mut DirectiveContext<'a>, body: ParserInput<'a>) -> DirectiveBeginResult<'a> {
        declaration(context, body, false)
    }
}

impl Directive for Var {
    fn definition(&self) -> &DirectiveDefinition<'_> { &self.definition }

    fn begin<'a>(&self, context: &mut DirectiveContext<'a>, body: ParserInput<'a>) -> DirectiveBeginResult<'a> {
        declaration(context, body, true)
    }
}

/// E.g. `enabled = !disabled`; annotations and uninitialized bindings are not parsed.
fn declaration<'a>(context: &DirectiveContext<'a>, body: ParserInput<'a>, mutable: bool) -> DirectiveBeginResult<'a> {
    let source = body.clone();

    let grammar = flat_map(
        ltrim(spanned(identifier())),

        |(name_span, name), input| {
            let initializer = flat_map(
                ltrim(tag("=")),
                |_, input| expression().parse(input)
            );

            let ((initializer, ()), remaining) = seq(
                initializer,
                ltrim(input_end())
            ).parse(input)?;

            Ok(((name_span, name, initializer), remaining))
        }
    );

    let ((name_span, name, initializer), _) = cut(grammar).parse(body)?;

    let value_info = infer(&initializer, &context.types).map_err(|cause| {
        let found_char = source.source_slice(cause.span.clone()).chars().next();
        Box::new(DirectiveTypeError {
            location: ErrorLocation {
                start_byte: initializer.span.start,
                failure_byte: cause.span.start,
                found_char,
            },
            cause,
        }) as Box<dyn ParseError>
    })?;

    // TODO: Track moves and borrows when reference-containing types are implemented.
    Ok(DirectiveAction::Declare {
        declaration: VariableDeclaration { name, name_span, mutable, initializer, value_info },
    })
}
