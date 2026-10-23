use std::cell::RefCell;
use std::ops::Range;

use crate::parsec::prelude::*;
use crate::parsec::{line_break, line_indent, line_start};
use crate::core::directives::DirectiveContext;
use crate::core::shorthands::ShorthandRegistry;

use super::directive_candidate::{
    classic_directive_candidate, classic_directive_start,
    jade_like_directive_candidate, jade_like_directive_start,
    DirectiveCandidate
};

#[cfg(test)]
mod tests;

/// Preliminary source structure; explicit end directives remain ordinary candidates.
#[derive(Debug)]
pub(crate) struct SyntaxDocument<'a> {
    pub nodes: Vec<SyntaxNode<'a>>,
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
/// Nonblank text lines also close directives according to their indentation.
pub(crate) fn document<'a>(
    context: Option<&DirectiveContext<'a>>,
) -> impl Parser<'a, Output = SyntaxDocument<'a>> {
    let shorthands = context.map(|context| &context.shorthands);

    state(|| RefCell::new(DocumentState::default()), move |document, input| {
        let classic = parser(|input| {
            let start_byte = input.byte_position();

            let parser = seq!(
                lookup(classic_candidate_start(shorthands)),
                cut(classic_directive_candidate(shorthands))
            );

            let (((), candidate), remaining) = parser.parse(input)?;
            let span = start_byte..remaining.byte_position();

            document.borrow_mut().nodes.push(SyntaxNode::Directive { span, candidate });

            Ok(((), remaining))
        });

        let jade_like = flat_map(jade_like_line_start(shorthands), |indent, input| {
            let start_byte = input.byte_position() - indent.len();
            let (candidate, remaining) = cut(jade_like_directive_candidate(indent, shorthands)).parse(input)?;

            let span = start_byte..remaining.byte_position();
            let mut document = document.borrow_mut();

            document.open_directives.push(OpenDirective {
                indent_len: indent.len(),
                opening_span: span.clone(),
            });

            document.nodes.push(SyntaxNode::Directive { span, candidate });

            Ok(((), remaining))
        });

        let text = parser(|input| {
            let start_byte = input.byte_position();

            let line_content = take_match(
                not(choice(
                    candidate_start(shorthands),
                    map(line_break(), |_| ())
                )),
                1..
            );

            let text_fragment = choice(
                map(seq(line_content, opt(line_break())), |_| ()),
                map(line_break(), |_| ())
            );

            let (_, remaining) = text_fragment.parse(input)?;

            let end_byte = remaining.byte_position();
            let mut document = document.borrow_mut();

            // Merge adjacent text fragments while preserving inferred ends between them.
            if let Some(SyntaxNode::Text { span, content }) = document.nodes.last_mut() {
                span.end = end_byte;
                *content = remaining.source_slice(span.clone());

            } else {
                let span = start_byte..end_byte;
                let content = remaining.source_slice(span.clone());

                document.nodes.push(SyntaxNode::Text { span, content });
            }

            Ok(((), remaining))
        });

        let node = choice!(classic, jade_like, text);

        let nodes = repeat(
            parser(|input| {
                let start_byte = input.byte_position();
                let (indent, _) = opt(line_indent()).parse(input.clone())?;

                if let Some(indent) = indent {
                    document.borrow_mut().close_at(start_byte, Some(indent.len()));
                }

                node.parse(input)
            }),

            0..
        );

        let end = input_end();
        let (_, remaining) = seq(nodes, end).parse(input)?;
        let document = document.get_mut();

        document.close_at(remaining.byte_position(), None);

        Ok((SyntaxDocument { nodes: std::mem::take(&mut document.nodes) }, remaining))
    })
}

#[derive(Default)]
struct DocumentState<'a> {
    nodes: Vec<SyntaxNode<'a>>,
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

            self.nodes.push(SyntaxNode::ImplicitEnd {
                span: position..position,
                opening_span: directive.opening_span,
            });
        }
    }
}

/// Empty braces are text; all other openings are candidates, even if unfinished.
fn candidate_start<'a>(
    shorthands: Option<&ShorthandRegistry<'_>>,
) -> impl Parser<'a, Output = ()> {
    choice(
        classic_candidate_start(shorthands),
        map(jade_like_line_start(shorthands), |_| ())
    )
}

fn classic_candidate_start<'a>(shorthands: Option<&ShorthandRegistry<'_>>) -> impl Parser<'a, Output = ()> {
    map(
        seq(classic_directive_start(shorthands), not(tag("}"))),
        |_| ()
    )
}

/// Reads indentation only at a physical line start, then checks the JadeLike marker.
fn jade_like_line_start<'a>(
    shorthands: Option<&ShorthandRegistry<'_>>,
) -> impl Parser<'a, Output = &'a str> {
    let indent = take(|ch, _, _| ch == ' ' || ch == '\t', 0..);

    map(
        seq!(
            line_start(),
            indent,
            lookup(jade_like_directive_start(shorthands))
        ),
        |(((), indent), ())| indent,
    )
}
