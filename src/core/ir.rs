use std::ops::Range;

use super::syntax::expression::Expression;
use super::types::ValueInfo;

/// Initial backend-independent structure; conditions are preserved, not executed.
#[derive(Debug)]
pub(crate) struct IrDocument<'a> {
    pub nodes: Vec<Node<'a>>,
}

#[derive(Debug)]
pub(crate) enum Node<'a> {
    Text {
        span: Range<usize>,
        content: &'a str
    },

    Declaration {
        span: Range<usize>,
        declaration: VariableDeclaration<'a>,
    },

    If {
        /// Full block range, including an explicit end when present.
        span: Range<usize>,
        condition: Expression<'a>,
        children: Vec<Node<'a>>,
    },
}

#[derive(Debug)]
pub(crate) struct VariableDeclaration<'a> {
    pub name: &'a str,
    pub name_span: Range<usize>,
    pub mutable: bool,
    pub initializer: Expression<'a>,
    pub value_info: ValueInfo<'a>,
}

/// Data needed to assemble a block once its children have been processed.
#[derive(Debug)]
pub(crate) enum BlockKind<'a> {
    If { condition: Expression<'a> },
}
