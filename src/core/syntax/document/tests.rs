use super::*;

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
