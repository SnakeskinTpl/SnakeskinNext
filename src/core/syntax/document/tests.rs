use crate::parsec::*;

use super::*;

#[test]
fn explicit_markers_resolve_paired_and_unpaired_content_openings() {
    use crate::core::directives::{Directive, End, Shorthand};
    use crate::core::shorthands::ShorthandBinding;
    use super::super::directive_candidate::DirectiveMode;

    let mut context = DirectiveContext::default();
    let end = End::new();
    let slash = &end.definition().shorthands[0];
    context.shorthands.entry(slash.opening.into()).or_insert(ShorthandBinding {
        opening: slash.opening.into(),
        closing: None,
        target: "end",
        span: 0..0,
        opening_span: 0..0,
    });
    Shorthand::new().begin(
        &mut context,
        ParserInput::new("tag { opening: \"<\", closing: \">\" }")
    ).unwrap();

    for (source, name, body, mode) in [
        ("{/block}", "end", "block", DirectiveMode::Regular),
        ("#{/block}", "end", "block", DirectiveMode::Extended),
        ("{/}", "end", "", DirectiveMode::Regular),
        ("- /block", "end", "block", DirectiveMode::Regular),
        ("{< div}", "tag", "div", DirectiveMode::Regular),
        ("- < div", "tag", "div", DirectiveMode::Regular),
        ("# < div", "tag", "div", DirectiveMode::Extended),
        ("{<div}", "tag", "div", DirectiveMode::Regular),
    ] {
        let (syntax, remaining) = document(Some(&context)).parse(ParserInput::new(source)).unwrap();
        let Some(SyntaxNode::Directive { span, candidate }) = syntax.nodes.first() else {
            panic!("Expected a directive for {source:?}");
        };

        assert_eq!(candidate.name, Some(name), "{source:?}");
        assert_eq!(candidate.body, body, "{source:?}");
        assert_eq!(candidate.mode, mode, "{source:?}");

        let body_end = span.end - candidate.closing_len;

        assert_eq!(&source[body_end - body.len()..body_end], body);
        assert!(remaining.is_at_end());
    }
}

#[test]
fn registered_shorthands_distinguish_classic_and_jade_like_forms() {
    use crate::core::directives::{Directive, Shorthand};
    use crate::core::syntax::DirectiveForm;

    let mut context = DirectiveContext::default();
    Shorthand::new().begin(
        &mut context,
        ParserInput::new("tag { opening: \"<\", closing: \">\" }")
    ).unwrap();
    let source = "before <div>\n  < div\n    content";

    let (document, remaining) = document(Some(&context)).parse(ParserInput::new(source)).unwrap();
    let directives: Vec<_> = document.nodes.iter().filter_map(|node| {
        match node {
            SyntaxNode::Directive { span, candidate } => Some((span, candidate)),
            _ => None,
        }
    }).collect();

    assert_eq!(directives.len(), 2);
    assert_eq!(directives[0].1.form, DirectiveForm::Classic);
    assert_eq!(directives[1].1.form, DirectiveForm::JadeLike { indent: "  " });

    for (span, candidate) in directives {
        assert_eq!(candidate.name, Some("tag"));
        assert_eq!(candidate.body, "div");

        assert_eq!(
            &source[span.start + candidate.opening_len..span.end - candidate.closing_len],
            candidate.content
        );
    }

    assert!(remaining.is_at_end());

    let error = super::document(Some(&context))
        .parse(ParserInput::new("before <div"))
        .unwrap_err();

    assert!(!error.is_recoverable());
}

#[test]
fn shorthand_names_borrow_language_source_while_bodies_borrow_document_source() {
    use crate::core::directives::{Directive, Shorthand};

    let language_source = String::from("tag { opening: \"<\", closing: \">\" }");
    let source = String::from("<div class=\"foo\">\n< div class=\"bar\"");
    let mut context = DirectiveContext::default();
    Shorthand::new().begin(&mut context, ParserInput::new(&language_source)).unwrap();

    let (document, remaining) = document(Some(&context)).parse(ParserInput::new(&source)).unwrap();
    let candidates: Vec<_> = document.nodes.iter().filter_map(|node| {
        match node {
            SyntaxNode::Directive { candidate, .. } => Some(candidate),
            _ => None,
        }
    }).collect();

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].body, "div class=\"foo\"");
    assert_eq!(candidates[1].body, "div class=\"bar\"");

    for candidate in candidates {
        let name = candidate.name.unwrap();

        assert_eq!(name, "tag");
        assert_eq!(name.as_ptr(), language_source.as_ptr());
        assert_eq!(candidate.body, candidate.content);
        assert_eq!(candidate.body.as_ptr(), candidate.content.as_ptr());
    }

    assert!(remaining.is_at_end());
}

#[test]
fn document_accepts_a_language_context_independent_of_source_lifetime() {
    let context = DirectiveContext::default();
    let source = String::from("{foo bar}");

    let (document, remaining) = document(Some(&context))
        .parse(ParserInput::new(&source))
        .unwrap();

    assert_eq!(document.nodes.len(), 1);
    assert!(remaining.is_at_end());
}

#[test]
fn directive_delimiter_lengths_locate_content_in_original_source() {
    let cases = [
        ("{foo \u{0436}}", 1, 1),
        ("#{foo \u{0436}}", 2, 1),
        ("- foo \u{0436}", 2, 0),
        (" \t# \t foo \u{0436}", 6, 0),
        ("  - foo {\n\u{0436}\n}", 4, 0),
    ];

    for (source, opening_len, closing_len) in cases {
        let prefix = "previous content\n";
        let full_source = format!("{prefix}{source}");
        let mut input = ParserInput::new(&full_source);
        input.set_byte_position(prefix.len());

        let (document, remaining) = document(None).parse(input).unwrap();
        let Some(SyntaxNode::Directive { span, candidate }) = document.nodes.first() else {
            panic!("Expected a directive for {source:?}");
        };

        assert_eq!(candidate.opening_len, opening_len, "{source:?}");
        assert_eq!(candidate.closing_len, closing_len, "{source:?}");

        let content_span = span.start + candidate.opening_len..span.end - candidate.closing_len;

        assert_eq!(remaining.source_slice(content_span.clone()), candidate.content);
        assert_eq!(content_span.len(), candidate.content.len());

        let body_start = content_span.end - candidate.body.len();

        assert_eq!(remaining.source_slice(body_start..content_span.end), candidate.body);
        assert!(remaining.is_at_end());
    }
}

#[test]
fn classic_directive_after_blank_line_closes_jade_like_block() {
    let source = "- if foo > 2\n  ...\n\n{bla}";
    let classic_start = source.find("{bla}").unwrap();

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    assert_eq!(document.nodes.len(), 4);
    assert!(matches!(&document.nodes[1], SyntaxNode::Text { content, .. }
        if *content == "\n  ...\n\n"));
    assert!(matches!(&document.nodes[2], SyntaxNode::ImplicitEnd { span, opening_span }
        if *span == (classic_start..classic_start) && *opening_span == (0..12)));
    assert!(matches!(&document.nodes[3], SyntaxNode::Directive { span, .. }
        if *span == (classic_start..source.len())));
    assert!(remaining.is_at_end());
}

#[test]
fn classic_directive_at_line_start_uses_the_same_indentation_rule() {
    let source = "- outer\n  - inner\n  {foo}";
    let line_start = source.find("  {foo}").unwrap();
    let classic_start = source.find("{foo}").unwrap();

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    let ends: Vec<_> = document.nodes.iter().filter_map(|node| match node {
        SyntaxNode::ImplicitEnd { span, opening_span } => Some((span.clone(), opening_span.clone())),
        _ => None,
    }).collect();

    assert_eq!(ends, vec![
        (line_start..line_start, 8..17),
        (source.len()..source.len(), 0..7),
    ]);

    let end_index = document.nodes.iter().position(|node| {
        matches!(node, SyntaxNode::ImplicitEnd { span, .. } if span.start == line_start)
    }).unwrap();

    assert!(matches!(&document.nodes[end_index + 1], SyntaxNode::Text { span, content }
        if *span == (line_start..classic_start) && *content == "  "));
    assert!(matches!(&document.nodes[end_index + 2], SyntaxNode::Directive { span, .. }
        if *span == (classic_start..source.len())));
    assert!(remaining.is_at_end());
}

#[test]
fn text_dedent_closes_nested_directives_and_preserves_source_slices() {
    let source = "- outer\n  - inner\n    inside\n  sibling\noutside";
    let sibling_start = source.find("  sibling").unwrap();
    let outside_start = source.find("outside").unwrap();

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    let ends: Vec<_> = document.nodes.iter().filter_map(|node| match node {
        SyntaxNode::ImplicitEnd { span, opening_span } => Some((span.clone(), opening_span.clone())),
        _ => None,
    }).collect();

    assert_eq!(ends, vec![
        (sibling_start..sibling_start, 8..17),
        (outside_start..outside_start, 0..7),
    ]);

    let mut end = 0;
    for node in &document.nodes {
        let span = match node {
            SyntaxNode::Text { span, content } => {
                assert_eq!(*content, &source[span.clone()]);
                assert_eq!(content.as_ptr(), source[span.clone()].as_ptr());
                span
            }
            SyntaxNode::Directive { span, .. } | SyntaxNode::ImplicitEnd { span, .. } => span,
        };

        assert_eq!(span.start, end);
        end = span.end;
    }

    assert_eq!(end, source.len());
    assert!(remaining.is_at_end());
}

#[test]
fn blank_lines_and_deeper_tabbed_text_keep_directive_open() {
    let source = "- outer\r\n\r\n \t\r\n\tinside\r\n";

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    assert_eq!(document.nodes.len(), 3);
    assert!(matches!(&document.nodes[1], SyntaxNode::Text { content, .. }
        if *content == "\r\n\r\n \t\r\n\tinside\r\n"));
    assert!(matches!(&document.nodes[2], SyntaxNode::ImplicitEnd { span, .. }
        if *span == (source.len()..source.len())));
    assert!(remaining.is_at_end());
}

#[test]
fn inline_text_after_classic_directive_does_not_act_as_a_line_start() {
    let source = "- outer\n  {foo}inline\n  inside";

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    let ends: Vec<_> = document.nodes.iter().filter_map(|node| match node {
        SyntaxNode::ImplicitEnd { span, .. } => Some(span.clone()),
        _ => None,
    }).collect();

    assert_eq!(ends, vec![source.len()..source.len()]);
    assert!(remaining.is_at_end());
}

#[test]
fn mixed_document_preserves_node_boundaries_and_multiline_bodies() {
    let source = r##"Привет {foo}
  - bar [
    {nested: "}"}
  ]
text - inline #{baz}
  # output &
    continued text
    .
Конец {}"##;

    let parts = [
        (false, "Привет "),
        (true, "{foo}"),
        (false, "\n"),
        (true, "  - bar [\n    {nested: \"}\"}\n  ]"),
        (false, "\n"),
        (false, "text - inline "),
        (true, "#{baz}"),
        (false, "\n"),
        (true, "  # output &\n    continued text\n    ."),
        (false, "\n"),
        (false, "Конец {}"),
    ];

    let (mut document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    // These assertions cover source nodes; inferred ends have empty ranges.
    document.nodes.retain(|node| !matches!(node, SyntaxNode::ImplicitEnd { .. }));

    assert_eq!(document.nodes.len(), parts.len());
    assert!(remaining.is_at_end());

    let mut start_byte = 0;
    for (node, (is_directive, expected)) in document.nodes.iter().zip(parts) {
        let span = match node {
            SyntaxNode::Text { span, content } => {
                assert!(!is_directive);
                assert_eq!(*content, expected);
                assert_eq!(content.as_ptr(), source[span.clone()].as_ptr());
                span
            }

            SyntaxNode::Directive { span, .. } => {
                assert!(is_directive);
                span
            }
            SyntaxNode::ImplicitEnd { .. } => panic!("Expected a source node"),
        };

        assert_eq!(*span, start_byte..start_byte + expected.len());
        assert_eq!(&source[span.clone()], expected);
        start_byte = span.end;
    }

    assert_eq!(start_byte, source.len());
}

#[test]
fn jade_like_directives_are_recognized_on_first_and_indented_lines() {
    let source = "- foo\n \t# bar value\r\ntext";

    let (mut document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    // These assertions cover source nodes; inferred ends have empty ranges.
    document.nodes.retain(|node| !matches!(node, SyntaxNode::ImplicitEnd { .. }));

    assert_eq!(document.nodes.len(), 5);
    assert!(matches!(&document.nodes[0], SyntaxNode::Directive { span, .. } if &source[span.clone()] == "- foo"));
    assert!(matches!(&document.nodes[1], SyntaxNode::Text { content, .. } if *content == "\n"));
    assert!(matches!(&document.nodes[2], SyntaxNode::Directive { span, .. } if &source[span.clone()] == " \t# bar value"));
    assert!(matches!(&document.nodes[3], SyntaxNode::Text { content, .. } if *content == "\r\n"));
    assert!(matches!(&document.nodes[4], SyntaxNode::Text { content, .. } if *content == "text"));
    assert!(remaining.is_at_end());
}

#[test]
fn inline_markers_remain_text() {
    let source = "text - foo # bar\n  ordinary text\n-no-separator";

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    assert_eq!(document.nodes.len(), 1);
    assert!(matches!(&document.nodes[0], SyntaxNode::Text { content, .. } if *content == source));
    assert!(remaining.is_at_end());
}

#[test]
fn braced_and_jade_like_nodes_preserve_contiguous_ranges() {
    let source = "{foo}\n\n  - bar [\n    {nested: \"}\"}\n  ]\ntext #{baz}";

    let (mut document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    // These assertions cover source nodes; inferred ends have empty ranges.
    document.nodes.retain(|node| !matches!(node, SyntaxNode::ImplicitEnd { .. }));

    assert_eq!(document.nodes.iter().filter(|node| matches!(node, SyntaxNode::Directive { .. })).count(), 3);

    let mut end = 0;
    for node in &document.nodes {
        let span = match node {
            SyntaxNode::Text { span, content } => {
                assert_eq!(*content, &source[span.clone()]);
                span
            }
            SyntaxNode::Directive { span, .. } => span,
            SyntaxNode::ImplicitEnd { .. } => panic!("Expected a source node"),
        };

        assert_eq!(span.start, end);
        end = span.end;
    }

    assert_eq!(end, source.len());
    assert!(remaining.is_at_end());
}

#[test]
fn malformed_jade_like_body_is_not_swallowed_as_text() {
    for source in ["text\n- foo [", "- foo ([)]", "# foo &\n", "- \ntext"] {
        let error = document(None).parse(ParserInput::new(source)).unwrap_err();

        assert!(!error.is_recoverable());
    }
}

#[test]
fn jade_like_cannot_start_at_arbitrary_document_slice_position() {
    let source = "text - foo";
    let mut input = ParserInput::new(source);
    input.set_byte_position(5);

    let (document, remaining) = document(None).parse(input).unwrap();

    assert!(matches!(&document.nodes[0], SyntaxNode::Text { span, content } if *span == (5..source.len()) && *content == "- foo"));
    assert!(remaining.is_at_end());
}

#[test]
fn empty_document_has_no_nodes() {
    let (document, remaining) = document(None).parse(ParserInput::new("")).unwrap();

    assert!(document.nodes.is_empty());
    assert!(remaining.is_at_end());
}

#[test]
fn plain_text_and_empty_braces_form_one_borrowed_text_node() {
    let source = String::from("Привет {} #{}!");

    let (document, remaining) = document(None).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(document.nodes.len(), 1);
    assert!(remaining.is_at_end());

    let SyntaxNode::Text { span, content } = &document.nodes[0] else {
        panic!("Expected text");
    };

    assert_eq!(*span, 0..source.len());
    assert_eq!(*content, source);
    assert_eq!(content.as_ptr(), source.as_ptr());
}

#[test]
fn explicit_end_directives_remain_siblings_and_source_ranges_are_contiguous() {
    let source = "какой то текст {foo}\n{if a}\n 454\n{/if}\ndfdf";

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    assert_eq!(document.nodes.len(), 7);
    assert!(remaining.is_at_end());

    let mut end = 0;
    for (index, node) in document.nodes.iter().enumerate() {
        let span = match node {
            SyntaxNode::Text { span, content } => {
                assert_eq!(index % 2, 0);
                assert_eq!(*content, &source[span.clone()]);
                span
            }
            SyntaxNode::Directive { span, .. } => {
                assert_eq!(index % 2, 1);
                span
            }
            SyntaxNode::ImplicitEnd { .. } => panic!("Expected a source node"),
        };

        assert_eq!(span.start, end);
        assert!(span.end > span.start);
        end = span.end;
    }

    assert_eq!(end, source.len());
}

#[test]
fn adjacent_classic_and_extended_directives_need_no_text_nodes() {
    let source = "{foo}#{bar {nested}}";

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    assert_eq!(document.nodes.len(), 2);
    assert!(matches!(&document.nodes[0], SyntaxNode::Directive { span, .. } if *span == (0..5)));
    assert!(matches!(&document.nodes[1], SyntaxNode::Directive { span, .. } if *span == (5..source.len())));
    assert!(remaining.is_at_end());
}

#[test]
fn unfinished_directive_is_not_swallowed_as_text_or_successful_prefix() {
    for source in ["text {", "text {foo", "text #{foo {bar}"] {
        let error = document(None).parse(ParserInput::new(source)).unwrap_err();

        assert_eq!(error.byte_position(), source.len());
        assert!(!error.is_recoverable());
    }
}

#[test]
fn node_ranges_use_original_input_position() {
    let mut input = ParserInput::new("!я{foo}");
    input.set_byte_position(1);

    let (document, remaining) = document(None).parse(input).unwrap();

    assert!(matches!(&document.nodes[0], SyntaxNode::Text { span, content } if *span == (1..3) && *content == "я"));
    assert!(matches!(&document.nodes[1], SyntaxNode::Directive { span, .. } if *span == (3..8)));
    assert!(remaining.is_at_end());
}

#[test]
fn jade_like_ends_close_siblings_ancestors_and_eof_in_stack_order() {
    let source = "- foo\n  - bar\n  - baz\n- qux";

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    let directives: Vec<_> = document.nodes.iter().filter_map(|node| match node {
        SyntaxNode::Directive { span, .. } => Some((false, span.clone(), span.clone())),
        SyntaxNode::ImplicitEnd { span, opening_span } => Some((true, span.clone(), opening_span.clone())),
        SyntaxNode::Text { .. } => None,
    }).collect();

    assert_eq!(directives, vec![
        (false, 0..5, 0..5),
        (false, 6..13, 6..13),
        (true, 14..14, 6..13),
        (false, 14..21, 14..21),
        (true, 22..22, 14..21),
        (true, 22..22, 0..5),
        (false, 22..27, 22..27),
        (true, 27..27, 22..27),
    ]);

    assert!(remaining.is_at_end());
}

#[test]
fn document_parser_creates_a_fresh_indentation_stack_for_each_parse() {
    let parser = document(None);

    let (first, _) = parser.parse(ParserInput::new("- foo")).unwrap();
    let (second, _) = parser.parse(ParserInput::new("plain text")).unwrap();

    assert!(matches!(first.nodes.last(), Some(SyntaxNode::ImplicitEnd { .. })));
    assert_eq!(second.nodes.len(), 1);
    assert!(matches!(second.nodes[0], SyntaxNode::Text { .. }));
}

#[test]
fn multiline_body_does_not_close_its_directive_on_internal_dedent() {
    let source = "- outer\n  - inner [\n0\n]\n- next";
    let inner_start = source.find("  - inner").unwrap();
    let inner_end = source.find("]").unwrap() + 1;
    let next_start = source.find("- next").unwrap();

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    let ends: Vec<_> = document.nodes.iter().filter_map(|node| match node {
        SyntaxNode::ImplicitEnd { span, opening_span } => Some((span.clone(), opening_span.clone())),
        _ => None,
    }).collect();

    assert_eq!(ends, vec![
        (next_start..next_start, inner_start..inner_end),
        (next_start..next_start, 0.."- outer".len()),
        (source.len()..source.len(), next_start..source.len()),
    ]);

    assert!(remaining.is_at_end());
}

#[test]
fn nested_directives_close_at_eof_after_trailing_crlf() {
    let source = "- outer\r\n\t# inner\r\n";
    let inner_start = source.find('\t').unwrap();
    let inner_end = source.len() - 2;

    let (document, remaining) = document(None).parse(ParserInput::new(source)).unwrap();

    let ends: Vec<_> = document.nodes.iter().filter_map(|node| match node {
        SyntaxNode::ImplicitEnd { span, opening_span } => Some((span.clone(), opening_span.clone())),
        _ => None,
    }).collect();

    assert_eq!(ends, vec![
        (source.len()..source.len(), inner_start..inner_end),
        (source.len()..source.len(), 0.."- outer".len()),
    ]);

    assert!(remaining.is_at_end());
}

#[test]
fn failed_document_parse_does_not_leave_open_directives_for_next_attempt() {
    let parser = document(None);

    let error = parser.parse(ParserInput::new("- outer\n  - broken [")).unwrap_err();

    assert!(!error.is_recoverable());

    let (document, remaining) = parser.parse(ParserInput::new("{foo}")).unwrap();

    assert_eq!(document.nodes.len(), 1);
    assert!(matches!(document.nodes[0], SyntaxNode::Directive { .. }));
    assert!(remaining.is_at_end());
}
