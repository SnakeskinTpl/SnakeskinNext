use crate::core::directives::DirectiveContext;
use crate::parsec::ParseError;

use super::builder::{ClosedBlock, IrBuilder, OpenDirective};

/// Coordinates IR nesting, handler completion and matching type scopes.
pub(crate) struct Blocks;

impl Blocks {
    pub fn open<'directive, 'source>(
        context: &mut DirectiveContext<'source>,
        builder: &mut IrBuilder<'directive, 'source>,
        opening: OpenDirective<'directive, 'source>,
    ) {
        context.types.push_scope();
        builder.open_block(opening);
    }

    /// The handler sees its own scope; the scope is removed after successful completion.
    pub fn close<'directive, 'source>(
        context: &mut DirectiveContext<'source>,
        builder: &mut IrBuilder<'directive, 'source>,
        position: usize,
    ) -> Result<Option<ClosedBlock<'directive>>, Box<dyn ParseError>> {
        let Some(closed) = builder.close_block(position) else {
            return Ok(None);
        };

        closed.directive.end(context)?;
        context.types.pop_scope();
        Ok(Some(closed))
    }
}
