use crate::parsec::prelude::*;

use super::arithmetic::sum;
use super::binary::binary_expression;
use super::{BinaryOperator, Expression};

/// Exactly one comparison is permitted; `a < b < c` fails at the second operator.
pub(crate) fn comparison<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    let compared = map(
        seq(
            sum(),
            opt(seq(ltrim(spanned(comparison_operator())), cut(sum())))
        ),

        |(left, comparison)| match comparison {
            Some(((span, operator), right)) => binary_expression(left, span, operator, right),
            None => left,
        }
    );

    map(
        seq(
            compared,
            cut(peek(ltrim(not(comparison_operator()))))
        ),

        |(expression, ())| expression
    )
}

/// Longer spellings must precede their prefixes, e.g. `<=` before `<`.
fn comparison_operator<'a>() -> impl Parser<'a, Output = BinaryOperator> {
    choice!(
        value(tag("=="), BinaryOperator::Equal),
        value(tag("!="), BinaryOperator::NotEqual),
        value(tag("<="), BinaryOperator::LessOrEqual),
        value(tag(">="), BinaryOperator::GreaterOrEqual),
        value(tag("<"), BinaryOperator::Less),
        value(tag(">"), BinaryOperator::Greater)
    )
}
