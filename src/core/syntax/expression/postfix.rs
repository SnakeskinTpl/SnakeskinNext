use std::ops::Range;

use crate::parsec::prelude::*;

use super::super::identifier;
use super::primary::primary;
use super::{expression, Expression, ExpressionKind};

enum Suffix<'a> {
    Field { name: &'a str, name_span: Range<usize> },
    Call { arguments: Vec<Expression<'a>> },
    Index { index: Expression<'a> },
}

/// Postfix operations associate left: `user.check()[index].name`.
pub(crate) fn postfix<'a>() -> impl Parser<'a, Output = Expression<'a>> {
    map(
        seq(
            primary(),
            repeat(ltrim(spanned(choice!(field(), call(), index()))), 0..)
        ),

        |(base, suffixes)| suffixes.into_iter().fold(base, |base, (span, suffix)| {
            let full_span = base.span.start..span.end;

            let kind = match suffix {
                Suffix::Field { name, name_span } => ExpressionKind::Field {
                    object: Box::new(base), name, name_span,
                },

                Suffix::Call { arguments } => ExpressionKind::Call {
                    callee: Box::new(base), arguments, arguments_span: span,
                },

                Suffix::Index { index } => ExpressionKind::Index {
                    object: Box::new(base), index: Box::new(index), brackets_span: span,
                },
            };

            Expression { span: full_span, kind }
        })
    )
}

/// Named fields and tuple positions, e.g. `.name` or `.0`; `..` remains untouched.
fn field<'a>() -> impl Parser<'a, Output = Suffix<'a>> {
    map(
        seq!(
            tag("."),
            not(tag(".")),
            cut(spanned(choice(identifier(), take(|ch, _, _| ch.is_ascii_digit(), 1..))))
        ),

        |((_, ()), (name_span, name))| Suffix::Field { name, name_span }
    )
}

/// Empty calls or comma-separated arguments, e.g. `()` or `(a, b + c)`.
fn call<'a>() -> impl Parser<'a, Output = Suffix<'a>> {
    let arguments = map(
        seq(
            rec(expression),
            repeat(seq(ltrim(tag(",")), cut(rec(expression))), 0..)
        ),

        |(first, rest)| std::iter::once(first)
            .chain(rest.into_iter().map(|(_, argument)| argument))
            .collect()
    );

    map(
        seq(
            tag("("),
            cut(seq(opt(arguments), ltrim(tag(")"))))
        ),

        |(_, (arguments, _))| Suffix::Call { arguments: arguments.unwrap_or_default() }
    )
}

/// An index always contains one expression, e.g. `[offset + 1]`.
fn index<'a>() -> impl Parser<'a, Output = Suffix<'a>> {
    map(
        seq(
            tag("["),
            cut(seq(rec(expression), ltrim(tag("]"))))
        ),

        |(_, (index, _))| Suffix::Index { index }
    )
}
