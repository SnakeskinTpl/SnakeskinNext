use std::ops::Range;

use crate::parsec::{choice, cut, lookup, map, not, repeat, seq, state, tag, take, take_match, Parser};

use super::directive_candidate::{classic_directive_candidate, classic_directive_start, DirectiveCandidate};

#[cfg(test)]
mod tests;

/// Preliminary source structure; explicit end directives remain ordinary candidates.
#[derive(Debug)]
pub(super) struct SyntaxTree<'a> {
    pub children: Vec<SyntaxNode<'a>>,
}

#[derive(Debug)]
pub(super) enum SyntaxNode<'a> {
    Text {
        span: Range<usize>,
        content: &'a str,
    },
    Directive {
        span: Range<usize>,
        candidate: DirectiveCandidate<'a>,
    },
}

/// Parses a flat document in braced syntax, preserving text and byte ranges.
/// A recognized opening commits to directive parsing, so malformed input is not hidden.
pub(super) fn document<'a>() -> impl Parser<'a, Output = SyntaxTree<'a>> {
    let directive = state(|| (), |_, input| {
        let start = input.byte_position();
        let parser = seq(lookup(candidate_start()), cut(classic_directive_candidate()));

        let (((), candidate), remaining) = parser.parse(input)?;
        let span = start..remaining.byte_position();

        Ok((SyntaxNode::Directive { span, candidate }, remaining))
    });

    let text = state(|| (), |_, input| {
        let start = input.byte_position();
        let (content, remaining) = take_match(not(candidate_start()), 1..).parse(input)?;
        let span = start..remaining.byte_position();

        Ok((SyntaxNode::Text { span, content }, remaining))
    });

    let nodes = repeat(choice(directive, text), 0..);
    let end = not(take(|_, _| true, 1..=1));

    map(seq(nodes, end), |(children, ())| SyntaxTree { children })
}

/// Empty braces are text; all other openings are candidates, even if unfinished.
fn candidate_start<'a>() -> impl Parser<'a, Output = ()> {
    map(seq(classic_directive_start(), not(tag("}"))), |_| ())
}
