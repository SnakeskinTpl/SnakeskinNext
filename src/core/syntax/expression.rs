use std::ops::Range;

use crate::parsec::Parser;

use super::literal::LiteralKind;

mod primary;
mod postfix;
mod unary;
mod arithmetic;
mod comparison;
mod logical;
mod binary;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Expression<'a> {
    /// Original byte range, excluding leading and trailing whitespace.
    pub span: Range<usize>,
    pub kind: ExpressionKind<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExpressionKind<'a> {
    Literal(LiteralKind<'a>),
    Identifier(&'a str),
    /// Parentheses remain explicit for diagnostics and tooling.
    Group(Box<Expression<'a>>),
    Field {
        object: Box<Expression<'a>>,
        name: &'a str,
        name_span: Range<usize>,
    },
    Call {
        callee: Box<Expression<'a>>,
        arguments: Vec<Expression<'a>>,
        arguments_span: Range<usize>,
    },
    Index {
        object: Box<Expression<'a>>,
        index: Box<Expression<'a>>,
        brackets_span: Range<usize>,
    },
    Unary {
        operator: UnaryOperator,
        operator_span: Range<usize>,
        operand: Box<Expression<'a>>,
    },
    Binary {
        operator: BinaryOperator,
        operator_span: Range<usize>,
        left: Box<Expression<'a>>,
        right: Box<Expression<'a>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOperator {
    Not,
    Negate,
    Positive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOperator {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    And,
    Or,
}

/// Parses arithmetic, non-associative comparisons and logical expressions.
/// Leading whitespace is skipped; trailing input is left to the enclosing grammar.
pub(crate) fn expression<'a>() -> Box<dyn Parser<'a, Output = Expression<'a>> + 'a> {
    Box::new(logical::disjunction())
}
