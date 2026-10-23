use crate::core::syntax::identifier;
use crate::core::shorthands::ShorthandDefinition;
use crate::parsec::prelude::*;

use super::{Directive, DirectiveAction, DirectiveBeginResult, DirectiveContext, DirectiveDefinition};

pub(crate) struct End {
    definition: DirectiveDefinition<'static>,
}

impl End {
    pub fn new() -> Self {
        Self {
            definition: DirectiveDefinition {
                name: "end",
                shorthands: vec![ShorthandDefinition { opening: "/", closing: None }],
                ..DirectiveDefinition::default()
            },
        }
    }
}

impl Directive for End {
    fn definition(&self) -> &DirectiveDefinition<'_> {
        &self.definition
    }

    fn begin<'a>(
        &self,
        _context: &mut DirectiveContext<'a>,
        body: ParserInput<'a>,
    ) -> DirectiveBeginResult<'a> {
        let parser = seq(
            ltrim(opt(identifier())),
            ltrim(input_end())
        );

        let ((name, ()), _) = cut(parser).parse(body)?;

        Ok(DirectiveAction::CloseBlock { name })
    }
}
