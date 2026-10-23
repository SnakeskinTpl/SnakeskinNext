use crate::parsec::prelude::*;

use super::binary::binary_chain;
use super::unary::unary;
use super::{BinaryOperator, Expression};

pub(crate) fn sum<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    binary_chain(product, choice(
        value(tag("+"), BinaryOperator::Add),
        value(tag("-"), BinaryOperator::Subtract)
    ))
}

fn product<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    binary_chain(unary, product_operator())
}

fn product_operator<'a>() -> impl Parser<'a, Output = BinaryOperator> {
    choice!(
        value(tag("*"), BinaryOperator::Multiply),
        value(tag("/"), BinaryOperator::Divide),
        value(tag("%"), BinaryOperator::Remainder)
    )
}
