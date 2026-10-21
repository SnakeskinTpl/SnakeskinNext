use crate::parsec::*;

use super::*;

#[test]
fn directive_start_consumes_only_opening_marker() {
    for (source, expected_position) in [("{foo", 1), ("#{foo", 2), ("{}", 1)] {
        let (extended, remaining) = classic_directive_start().parse(ParserInput::new(source)).unwrap();

        assert_eq!(extended, source.starts_with('#'));
        assert_eq!(remaining.byte_position(), expected_position);
        assert_eq!(remaining.peek(), source[expected_position..].chars().next());
    }
}

#[test]
fn directive_start_requires_adjacent_hash_and_brace() {
    for source in ["", "text", "#", "# {"] {
        assert!(classic_directive_start().parse(ParserInput::new(source)).is_err());
    }
}

#[test]
fn directive_start_can_stop_text_scanning() {
    let parser = take_match(not(classic_directive_start()), 1..);

    let (output, remaining) = parser.parse(ParserInput::new("text #{foo}")).unwrap();

    assert_eq!(output, "text ");
    assert_eq!(remaining.byte_position(), 5);
    assert_eq!(remaining.peek(), Some('#'));
}

#[test]
fn names_cannot_start_with_a_digit() {
    for content in ["1 ", "1foo value", "١foo value"] {
        let source = format!("{{{content}}}");

        let (directive, _) = classic_directive_candidate().parse(ParserInput::new(&source)).unwrap();

        assert_eq!(directive.name, None);
        assert_eq!(directive.body, content);
    }

    for name in ["foo1", "_foo", "имя2"] {
        let source = format!("{{{name} value}}");

        let (directive, _) = classic_directive_candidate().parse(ParserInput::new(&source)).unwrap();

        assert_eq!(directive.name, Some(name));
        assert_eq!(directive.body, "value");
    }
}

#[test]
fn optional_hash_prefix_preserves_content_and_remaining_input() {
    for source in ["{foo bar}rest", "#{foo bar}rest"] {
        let (directive, remaining) = classic_directive_candidate().parse(ParserInput::new(source)).unwrap();

        assert_eq!(directive.name, Some("foo"));
        assert_eq!(directive.body, "bar");
        assert_eq!(directive.content, "foo bar");
        assert_eq!(directive.form, DirectiveForm::Classic);
        assert_eq!(directive.extended, source.starts_with('#'));
        assert_eq!(remaining.byte_position(), source.find('}').unwrap() + 1);
        assert_eq!(remaining.peek(), Some('r'));
    }
}

#[test]
fn named_directive_preserves_source_content_and_remaining_input() {
    let source = String::from("{foo bar}rest");

    let (directive, remaining) = classic_directive_candidate().parse(ParserInput::new(&source)).unwrap();

    assert_eq!(directive.name, Some("foo"));
    assert_eq!(directive.body, "bar");
    assert_eq!(directive.content, "foo bar");
    assert_eq!(directive.content.as_ptr(), source[1..].as_ptr());
    assert_eq!(remaining.byte_position(), 9);
    assert_eq!(remaining.peek(), Some('r'));
}

#[test]
fn whitespace_separates_name_and_preserves_body_end() {
    let (directive, _) = classic_directive_candidate()
        .parse(ParserInput::new("{foo_2\t\n\u{2003}value  }"))
        .unwrap();

    assert_eq!(directive.name, Some("foo_2"));
    assert_eq!(directive.body, "value  ");
    assert_eq!(directive.content, "foo_2\t\n\u{2003}value  ");
}

#[test]
fn shorthand_and_expression_content_remains_unresolved() {
    for content in ["+= value", "/foo", "user.name", "foo.bar baz", "foo", "? call()", " foo bar"] {
        let source = format!("{{{content}}}");

        let (directive, remaining) = classic_directive_candidate().parse(ParserInput::new(&source)).unwrap();

        assert_eq!(directive.name, None);
        assert_eq!(directive.body, content);
        assert_eq!(directive.content, content);
        assert!(remaining.is_at_end());
    }
}

#[test]
fn named_directive_can_have_empty_arguments() {
    let (directive, _) = classic_directive_candidate().parse(ParserInput::new("{foo }")).unwrap();

    assert_eq!(directive.name, Some("foo"));
    assert_eq!(directive.body, "");
}

#[test]
fn nested_and_escaped_braces_preserve_content() {
    let source = r"{foo {nested} \}value}rest";

    let (directive, remaining) = classic_directive_candidate().parse(ParserInput::new(source)).unwrap();

    assert_eq!(directive.name, Some("foo"));
    assert_eq!(directive.body, r"{nested} \}value");
    assert_eq!(remaining.peek(), Some('r'));
}

#[test]
fn empty_braces_are_not_a_directive() {
    let error = classic_directive_candidate().parse(ParserInput::new("{}")).unwrap_err();

    assert_eq!(error.byte_position(), 1);
}

#[test]
fn unclosed_directive_reports_end_of_input() {
    for source in ["{foo bar", "{foo {bar}"] {
        let error = classic_directive_candidate().parse(ParserInput::new(source)).unwrap_err();

        assert_eq!(error.byte_position(), source.len());
        assert_eq!(error.location().found_char, None);
    }
}
