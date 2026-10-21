use crate::parsec::*;

use super::*;

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
        (false, "\ntext - inline "),
        (true, "#{baz}"),
        (false, "\n"),
        (true, "  # output &\n    continued text\n    ."),
        (false, "\nКонец {}"),
    ];

    let (mut tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    // These assertions cover source nodes; inferred ends have empty ranges.
    tree.children.retain(|node| !matches!(node, SyntaxNode::ImplicitEnd { .. }));

    assert_eq!(tree.children.len(), parts.len());
    assert!(remaining.is_at_end());

    let mut start_byte = 0;
    for (node, (is_directive, expected)) in tree.children.iter().zip(parts) {
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

    let (mut tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    // These assertions cover source nodes; inferred ends have empty ranges.
    tree.children.retain(|node| !matches!(node, SyntaxNode::ImplicitEnd { .. }));

    assert_eq!(tree.children.len(), 4);
    assert!(matches!(&tree.children[0], SyntaxNode::Directive { span, .. } if &source[span.clone()] == "- foo"));
    assert!(matches!(&tree.children[1], SyntaxNode::Text { content, .. } if *content == "\n"));
    assert!(matches!(&tree.children[2], SyntaxNode::Directive { span, .. } if &source[span.clone()] == " \t# bar value"));
    assert!(matches!(&tree.children[3], SyntaxNode::Text { content, .. } if *content == "\r\ntext"));
    assert!(remaining.is_at_end());
}

#[test]
fn inline_markers_remain_text() {
    let source = "text - foo # bar\n  ordinary text\n-no-separator";

    let (tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    assert_eq!(tree.children.len(), 1);
    assert!(matches!(&tree.children[0], SyntaxNode::Text { content, .. } if *content == source));
    assert!(remaining.is_at_end());
}

#[test]
fn braced_and_jade_like_nodes_preserve_contiguous_ranges() {
    let source = "{foo}\n\n  - bar [\n    {nested: \"}\"}\n  ]\ntext #{baz}";

    let (mut tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    // These assertions cover source nodes; inferred ends have empty ranges.
    tree.children.retain(|node| !matches!(node, SyntaxNode::ImplicitEnd { .. }));

    assert_eq!(tree.children.iter().filter(|node| matches!(node, SyntaxNode::Directive { .. })).count(), 3);

    let mut end = 0;
    for node in &tree.children {
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
        let error = document().parse(ParserInput::new(source)).unwrap_err();

        assert!(!error.is_recoverable());
    }
}

#[test]
fn jade_like_cannot_start_at_arbitrary_document_slice_position() {
    let source = "text - foo";
    let mut input = ParserInput::new(source);
    input.set_byte_position(5);

    let (tree, remaining) = document().parse(input).unwrap();

    assert!(matches!(&tree.children[0], SyntaxNode::Text { span, content } if *span == (5..source.len()) && *content == "- foo"));
    assert!(remaining.is_at_end());
}

#[test]
fn empty_document_has_no_nodes() {
    let (tree, remaining) = document().parse(ParserInput::new("")).unwrap();

    assert!(tree.children.is_empty());
    assert!(remaining.is_at_end());
}

#[test]
fn plain_text_and_empty_braces_form_one_borrowed_text_node() {
    let source = String::from("Привет {} #{}!");

    let (tree, remaining) = document().parse(ParserInput::new(&source)).unwrap();

    assert_eq!(tree.children.len(), 1);
    assert!(remaining.is_at_end());

    let SyntaxNode::Text { span, content } = &tree.children[0] else {
        panic!("Expected text");
    };

    assert_eq!(*span, 0..source.len());
    assert_eq!(*content, source);
    assert_eq!(content.as_ptr(), source.as_ptr());
}

#[test]
fn explicit_end_directives_remain_siblings_and_source_ranges_are_contiguous() {
    let source = "какой то текст {foo}\n{if a}\n 454\n{/if}\ndfdf";

    let (tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    assert_eq!(tree.children.len(), 7);
    assert!(remaining.is_at_end());

    let mut end = 0;
    for (index, node) in tree.children.iter().enumerate() {
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

    let (tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    assert_eq!(tree.children.len(), 2);
    assert!(matches!(&tree.children[0], SyntaxNode::Directive { span, .. } if *span == (0..5)));
    assert!(matches!(&tree.children[1], SyntaxNode::Directive { span, .. } if *span == (5..source.len())));
    assert!(remaining.is_at_end());
}

#[test]
fn unfinished_directive_is_not_swallowed_as_text_or_successful_prefix() {
    for source in ["text {", "text {foo", "text #{foo {bar}"] {
        let error = document().parse(ParserInput::new(source)).unwrap_err();

        assert_eq!(error.byte_position(), source.len());
        assert!(!error.is_recoverable());
    }
}

#[test]
fn node_ranges_use_original_input_position() {
    let mut input = ParserInput::new("!я{foo}");
    input.set_byte_position(1);

    let (tree, remaining) = document().parse(input).unwrap();

    assert!(matches!(&tree.children[0], SyntaxNode::Text { span, content } if *span == (1..3) && *content == "я"));
    assert!(matches!(&tree.children[1], SyntaxNode::Directive { span, .. } if *span == (3..8)));
    assert!(remaining.is_at_end());
}

#[test]
fn jade_like_ends_close_siblings_ancestors_and_eof_in_stack_order() {
    let source = "- foo\n  - bar\n  - baz\n- qux";

    let (tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    let directives: Vec<_> = tree.children.iter().filter_map(|node| match node {
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
    let parser = document();

    let (first, _) = parser.parse(ParserInput::new("- foo")).unwrap();
    let (second, _) = parser.parse(ParserInput::new("plain text")).unwrap();

    assert!(matches!(first.children.last(), Some(SyntaxNode::ImplicitEnd { .. })));
    assert_eq!(second.children.len(), 1);
    assert!(matches!(second.children[0], SyntaxNode::Text { .. }));
}

#[test]
fn multiline_body_does_not_close_its_directive_on_internal_dedent() {
    let source = "- outer\n  - inner [\n0\n]\n- next";
    let inner_start = source.find("  - inner").unwrap();
    let inner_end = source.find("]").unwrap() + 1;
    let next_start = source.find("- next").unwrap();

    let (tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    let ends: Vec<_> = tree.children.iter().filter_map(|node| match node {
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

    let (tree, remaining) = document().parse(ParserInput::new(source)).unwrap();

    let ends: Vec<_> = tree.children.iter().filter_map(|node| match node {
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
    let parser = document();

    let error = parser.parse(ParserInput::new("- outer\n  - broken [")).unwrap_err();

    assert!(!error.is_recoverable());

    let (tree, remaining) = parser.parse(ParserInput::new("{foo}")).unwrap();

    assert_eq!(tree.children.len(), 1);
    assert!(matches!(tree.children[0], SyntaxNode::Directive { .. }));
    assert!(remaining.is_at_end());
}
