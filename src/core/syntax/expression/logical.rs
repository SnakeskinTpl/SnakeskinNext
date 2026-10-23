use crate::parsec::prelude::*;

use super::binary::binary_chain;
use super::comparison::comparison;
use super::{BinaryOperator, Expression};

// Precedence rises down these layers: ||, &&, comparisons, sums, products, unary.
pub(crate) fn disjunction<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    binary_chain(conjunction, value(tag("||"), BinaryOperator::Or))
}

fn conjunction<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    binary_chain(comparison, value(tag("&&"), BinaryOperator::And))
}
