use crate::parsec::prelude::*;

use super::postfix::postfix;
use super::{Expression, ExpressionKind, UnaryOperator};

pub(crate) fn unary<'a>() -> Box<dyn Parser<'a, Output = Expression<'a>> + 'a> {
    Box::new(ltrim(choice(prefix(), postfix())))
}

/// E.g. `-+value` nests to the right; every consumed prefix requires an operand.
fn prefix<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    map(
        seq(
            spanned(unary_operator()),
            cut(rec(unary))
        ),

        |((operator_span, operator), operand)| Expression {
            span: operator_span.start..operand.span.end,
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
