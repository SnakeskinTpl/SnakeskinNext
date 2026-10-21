use std::cell::RefCell;
use std::ops::Range;

use crate::parsec::{choice, cut, flat_map, line_start, lookup, map, not, parser, repeat, seq, state, tag, take, take_match, Parser};

use super::directive_candidate::{classic_directive_candidate, classic_directive_start, jade_like_directive_candidate, jade_like_directive_start, DirectiveCandidate};

#[cfg(test)]
mod tests;

/// Preliminary source structure; explicit end directives remain ordinary candidates.
#[derive(Debug)]
pub(crate) struct SyntaxTree<'a> {
    pub children: Vec<SyntaxNode<'a>>,
}

#[derive(Debug)]
pub(crate) enum SyntaxNode<'a> {
    Text {
        span: Range<usize>,
        content: &'a str,
    },

    /// An inferred end linked to the opening directive by its source range.
    ImplicitEnd {
        span: Range<usize>,
        opening_span: Range<usize>,
    },

    Directive {
        span: Range<usize>,
        candidate: DirectiveCandidate<'a>,
    },
}

/// Parses a flat document in braced and JadeLike syntax, preserving text and byte ranges.
/// A recognized opening commits to directive parsing, so malformed input is not hidden.
/// JadeLike directives close before a sibling or ancestor directive, or at end of input.
/// Text and classic directives do not change the indentation stack.
pub(crate) fn document<'a>() -> impl Parser<'a, Output = SyntaxTree<'a>> {
    state(|| RefCell::new(DocumentState::default()), |document, input| {
        let classic = parser(|input| {
            let start_byte = input.byte_position();

            let parser = seq(
                lookup(classic_candidate_start()),
                cut(classic_directive_candidate())
            );

            let (((), candidate), remaining) = parser.parse(input)?;
            let span = start_byte..remaining.byte_position();

            document.borrow_mut().children.push(SyntaxNode::Directive { span, candidate });

            Ok(((), remaining))
        });

        let jade_like = flat_map(jade_like_line_start(), |indent, input| {
            let start_byte = input.byte_position() - indent.len();
            let (candidate, remaining) = cut(jade_like_directive_candidate(indent)).parse(input)?;

            let span = start_byte..remaining.byte_position();
            let mut document = document.borrow_mut();

            document.close_at(start_byte, Some(indent.len()));

            document.open_directives.push(OpenDirective {
                indent_len: indent.len(),
                opening_span: span.clone(),
            });

            document.children.push(SyntaxNode::Directive { span, candidate });

            Ok(((), remaining))
        });

        let text = parser(|input| {
            let start_byte = input.byte_position();
            let (content, remaining) = take_match(not(candidate_start()), 1..).parse(input)?;
            let span = start_byte..remaining.byte_position();

            document.borrow_mut().children.push(SyntaxNode::Text { span, content });

            Ok(((), remaining))
        });

        let nodes = repeat(
            choice(
                choice(classic, jade_like),
                text
            ),
            0..
        );

        let end = not(take(|_, _, _| true, 1..=1));
        let (_, remaining) = seq(nodes, end).parse(input)?;
        let document = document.get_mut();

        document.close_at(remaining.byte_position(), None);

        Ok((SyntaxTree { children: std::mem::take(&mut document.children) }, remaining))
    })
}

#[derive(Default)]
struct DocumentState<'a> {
    children: Vec<SyntaxNode<'a>>,
    open_directives: Vec<OpenDirective>,
}

struct OpenDirective {
    indent_len: usize,
    opening_span: Range<usize>,
}

impl DocumentState<'_> {
    fn close_at(&mut self, position: usize, indent: Option<usize>) {
        while self.open_directives.last().is_some_and(|directive| {
            indent.is_none_or(|indent| directive.indent_len >= indent)
        }) {
            let Some(directive) = self.open_directives.pop() else {
                break;
            };

            self.children.push(SyntaxNode::ImplicitEnd {
                span: position..position,
                opening_span: directive.opening_span,
            });
        }
    }
}

/// Empty braces are text; all other openings are candidates, even if unfinished.
fn candidate_start<'a>() -> impl Parser<'a, Output = ()> {
    choice(
        classic_candidate_start(),
        map(jade_like_line_start(), |_| ())
    )
}

fn classic_candidate_start<'a>() -> impl Parser<'a, Output = ()> {
    map(
        seq(classic_directive_start(), not(tag("}"))),
        |_| ()
    )
}

/// Reads indentation only at a physical line start, then checks the JadeLike marker.
fn jade_like_line_start<'a>() -> impl Parser<'a, Output = &'a str> {
    let indent = take(|ch, _, _| ch == ' ' || ch == '\t', 0..);

    map(
        seq(
            line_start(),
            seq(indent, lookup(jade_like_directive_start()))
        ),
        |((), (indent, ()))| indent,
    )
}
