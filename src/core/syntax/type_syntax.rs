use std::ops::Range;

use crate::parsec::prelude::*;

use super::identifier;
use super::literal::{literal, Literal};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TypeSyntax<'a> {
    /// Original byte range of the complete type, excluding surrounding whitespace.
    pub span: Range<usize>,
    pub kind: TypeSyntaxKind<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TypeSyntaxKind<'a> {
    /// Unresolved name, e.g. `Bool` or `User`.
    Named(&'a str),
    /// Ordered arguments, e.g. `Res<Str, Error>` or `Array<Int, 3>`.
    Applied {
        name: &'a str,
        arguments: Vec<TypeArgumentSyntax<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TypeArgumentSyntax<'a> {
    Type(TypeSyntax<'a>),
    Value(Literal<'a>),
}

/// Parses capitalized names and nonempty argument lists; type validity is checked later.
/// Leading whitespace is skipped; separators remain for the enclosing grammar.
pub(crate) fn type_syntax<'a>() -> Box<dyn Parser<'a, Output = TypeSyntax<'a>> + 'a> {
    Box::new(map(
        ltrim(spanned(seq(type_name(), opt(arguments())))),

        |(span, (name, arguments))| TypeSyntax {
            span,
            kind: match arguments {
                Some(arguments) => TypeSyntaxKind::Applied { name, arguments },
                None => TypeSyntaxKind::Named(name),
            },
        }
    ))
}

fn type_name<'a>() -> impl Parser<'a, Output = &'a str> {
    recognize(seq(
        peek(take(|ch, _, _| ch.is_uppercase(), 1..=1)),
        identifier()
    ))
}

/// Once `<` is consumed, missing arguments or `>` are committed errors.
fn arguments<'a>() -> impl Parser<'a, Output = Vec<TypeArgumentSyntax<'a>>> {
    map(
        seq(
            ltrim(tag("<")),
            cut(seq!(
                argument(),
                repeat(seq(ltrim(tag(",")), cut(argument())), 0..),
                ltrim(tag(">"))
            ))
        ),

        |(_, ((first, rest), _))| std::iter::once(first)
            .chain(rest.into_iter().map(|(_, argument)| argument))
            .collect()
    )
}

fn argument<'a>() -> impl Parser<'a, Output = TypeArgumentSyntax<'a>> {
    choice(
        map(rec(type_syntax), TypeArgumentSyntax::Type),
        map(literal(), TypeArgumentSyntax::Value)
    )
}
