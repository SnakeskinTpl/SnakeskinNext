use crate::parsec::prelude::*;

use super::super::type_syntax::type_syntax;
use super::postfix::postfix;
use super::{Expression, ExpressionKind, UnaryOperator};

pub(crate) fn unary<'a>() -> Box<dyn Parser<'a, Output = Expression<'a>> + 'a> {
    Box::new(ltrim(choice!(prefix(), cast(), postfix())))
}

/// Casts bind like prefixes: `<Int>f()` casts the call, `<Int>a + b` casts only `a`.
fn cast<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    map(
        spanned(seq(
            tag("<"),
            cut(seq!(
                type_syntax(),
                ltrim(tag(">")),
                rec(unary)
            ))
        )),

        |(span, (_, ((target, _), operand)))| Expression {
            span,
            kind: ExpressionKind::Cast { target, operand: Box::new(operand) },
        }
    )
}

/// E.g. `-+value` nests to the right; every consumed prefix requires an operand.
fn prefix<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    map(
        spanned(seq(
            spanned(unary_operator()),
            cut(rec(unary))
        )),

        |(span, ((operator_span, operator), operand))| Expression {
            span,
            kind: ExpressionKind::Unary {
                operator,
                operator_span,
                operand: Box::new(operand),
            },
        }
    )
}

fn unary_operator<'a>() -> impl Parser<'a, Output = UnaryOperator> {
    choice!(
        value(tag("!"), UnaryOperator::Not),
        value(tag("-"), UnaryOperator::Negate),
        value(tag("+"), UnaryOperator::Positive)
    )
}
