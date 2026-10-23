use std::ops::Range;

use crate::core::directives::Directive;
use crate::core::ir::Node;
use crate::core::syntax::expression::Expression;
use crate::parsec::ErrorLocation;

pub(crate) struct OpenDirective<'a, 'source> {
    pub name: &'a str,
    pub span: Range<usize>,
    pub directive: &'a dyn Directive,
    pub location: ErrorLocation,
    pub condition: Option<Expression<'source>>,
    pub children: Vec<Node<'source>>,
}

pub(crate) struct InferredClose<'a> {
    pub position: usize,
    pub name: &'a str,
}

pub(crate) fn push_output<'a>(node: Node<'a>, stack: &mut [OpenDirective<'_, 'a>], nodes: &mut Vec<Node<'a>>) {
    if let Some(opening) = stack.last_mut() {
        opening.children.push(node);

    } else {
        nodes.push(node);
    }
}

pub(crate) fn finish_output<'a>(opening: OpenDirective<'_, 'a>, end: usize, stack: &mut [OpenDirective<'_, 'a>], nodes: &mut Vec<Node<'a>>) {
    if let Some(condition) = opening.condition {
        push_output(Node::If {
            span: opening.span.start..end,
            condition,
            children: opening.children,
        }, stack, nodes);

    } else {
        // TODO: Emit IR for other runtime block directives when their semantics are defined.
        for node in opening.children {
            push_output(node, stack, nodes);
        }
    }
}
