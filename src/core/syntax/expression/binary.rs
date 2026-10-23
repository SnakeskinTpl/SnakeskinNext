use std::ops::Range;

use crate::parsec::prelude::*;

use super::{BinaryOperator, Expression, ExpressionKind};

/// Fold equal-precedence operators left: `a - b - c` becomes `(a - b) - c`.
pub(crate) fn binary_chain<'a, F, P, O>(operand: F, operator: O) -> impl Parser<'a, Output = Expression<'a>>
where
    F: Fn() -> P,
    P: Parser<'a, Output = Expression<'a>>,
    O: Parser<'a, Output = BinaryOperator>,
{
    map(
        seq(
            operand(),
            repeat(seq(
                ltrim(spanned(operator)),
                cut(operand())
            ), 0..)
        ),

        |(first, rest)| rest.into_iter().fold(first, |left, ((span, operator), right)| {
            binary_expression(left, span, operator, right)
        })
    )
}

pub(crate) fn binary_expression<'a>(
    left: Expression<'a>,
    operator_span: Range<usize>,
    operator: BinaryOperator,
    right: Expression<'a>,
) -> Expression<'a> {
    Expression {
        span: left.span.start..right.span.end,
        kind: ExpressionKind::Binary {
            operator,
            operator_span,
            left: Box::new(left),
            right: Box::new(right),
        },
    }
}
