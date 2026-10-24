use std::ops::Range;

use crate::core::directives::Directive;
use crate::core::ir::{BlockKind, Node, IrDocument};
use crate::parsec::ErrorLocation;

use super::err::{ProcessError, ProcessErrorKind};
use super::ProcessResult;

pub(crate) struct OpenDirective<'a, 'source> {
    pub name: &'a str,
    pub span: Range<usize>,
    pub directive: &'a dyn Directive,
    pub location: ErrorLocation,
    /// None leaves children transparent until this directive has an IR representation.
    pub block: Option<BlockKind<'source>>,
}

/// Handler information returned to the processor after assembling a closed block.
pub(crate) struct ClosedBlock<'a> {
    pub name: &'a str,
    pub directive: &'a dyn Directive,
}

struct OpenBlock<'a, 'source> {
    opening: OpenDirective<'a, 'source>,
    children: Vec<Node<'source>>,
}

/// Owns IR nesting; handler execution and type scopes belong to the processor.
pub(crate) struct IrBuilder<'a, 'source> {
    block_stack: Vec<OpenBlock<'a, 'source>>,
    children: Vec<Node<'source>>,
}

impl<'a, 'source> IrBuilder<'a, 'source> {
    pub fn new() -> Self {
        Self { block_stack: Vec::new(), children: Vec::new() }
    }

    pub fn current(&self) -> Option<&OpenDirective<'a, 'source>> {
        self.block_stack.last().map(|block| &block.opening)
    }

    pub fn contains_opening(&self, span: &Range<usize>) -> bool {
        self.block_stack.iter().any(|block| block.opening.span == *span)
    }

    pub fn push(&mut self, node: Node<'source>) {
        if let Some(block) = self.block_stack.last_mut() {
            block.children.push(node);

        } else {
            self.children.push(node);
        }
    }

    pub fn open_block(&mut self, opening: OpenDirective<'a, 'source>) {
        self.block_stack.push(OpenBlock { opening, children: Vec::new() });
    }

    pub fn close_block(&mut self, end: usize) -> Option<ClosedBlock<'a>> {
        let OpenBlock { opening, children } = self.block_stack.pop()?;

        match opening.block {
            Some(BlockKind::If { condition }) => self.push(Node::If {
                span: opening.span.start..end,
                condition,
                children,
            }),

            None => {
                // TODO: Emit IR for other runtime block directives when their semantics are defined.
                for node in children { self.push(node); }
            }
        }

        Some(ClosedBlock { name: opening.name, directive: opening.directive })
    }

    pub fn finish(self) -> ProcessResult<'source> {
        if let Some(block) = self.block_stack.last() {
            return Err(ProcessError::new(
                block.opening.location,
                ProcessErrorKind::UnclosedBlock { name: block.opening.name.into() }
            ));
        }

        Ok(IrDocument { nodes: self.children })
    }
}
