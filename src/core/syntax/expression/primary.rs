use crate::parsec::prelude::*;

use super::super::identifier;
use super::super::literal::literal;
use super::{expression, Expression, ExpressionKind};

pub(crate) fn primary<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    choice!(
        group(),
        map(literal(), |literal| Expression {
            span: literal.span,
            kind: ExpressionKind::Literal(literal.kind),
        }),

        map(spanned(identifier()), |(span, name)| Expression {
            span,
            kind: ExpressionKind::Identifier(name),
        })
    )
}

/// E.g. `( !flag )` retains both the group range and the inner expression range.
fn group<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    map(
        spanned(seq(
            tag("("),
            cut(seq(rec(expression), ltrim(tag(")"))))
        )),

        |(span, (_, (inner, _)))| Expression { span, kind: ExpressionKind::Group(Box::new(inner)) }
    )
}
