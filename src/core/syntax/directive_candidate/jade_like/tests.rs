use crate::parsec::*;

use super::*;

#[test]
fn explicit_continuation_waits_for_nested_frames_before_closing() {
    let content = "foo &\n [\n .\n \"}\"\n ]\n .";
    let source = format!("- {content}\nnext");

    let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(candidate.content, content);
    assert_eq!(remaining.peek(), Some('\n'));
}

#[test]
fn repeated_ampersand_does_not_open_another_continuation() {
    let content = "foo &\n &\n .";
    let source = format!("- {content}\nnext");

    let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(candidate.content, content);
    assert_eq!(remaining.peek(), Some('\n'));
}

#[test]
fn eof_reports_closing_character_for_innermost_frame() {
    for (content, expected) in [("foo &\n [\n", ']'), ("foo [\n \"\n", '"'), ("foo &\n []\n", '.')] {
        let source = format!("- {content}");

        let error = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap_err();
        let error = (&*error as &dyn std::error::Error).downcast_ref::<JadeLikeBodyError>().unwrap();

        assert_eq!(error.expected, Some(expected));
        assert_eq!(error.byte_position(), source.len());
    }
}

#[test]
fn balanced_delimiters_keep_multiline_arguments_together() {
    for content in ["foo {\n}\n", "foo [\n]\n", "foo (\n)\n", "foo {\n arr: [(1)]\n}\n"] {
        let source = format!("- {content}- next");

        let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

        assert_eq!(candidate.content, content.strip_suffix('\n').unwrap());
        assert_eq!(candidate.name, Some("foo"));
        assert_eq!(remaining.peek(), Some('\n'));
        assert_eq!(remaining.byte_position(), source.len() - "\n- next".len());
    }
}

#[test]
fn all_quote_kinds_support_multiline_and_ignore_brackets() {
    for quote in ['"', '\'', '`'] {
        let content = format!("foo {quote}\n text ] }} )\n {quote}inline\n  {quote}");
        let source = format!("- {content}\nnext");

        let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

        assert_eq!(candidate.content, content);
        assert_eq!(remaining.peek(), Some('\n'));
    }
}

#[test]
fn ordinary_quoted_strings_can_close_inline() {
    let source = r#"- foo ["}", '\'', `]`]"#;

    let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(source)).unwrap();

    assert_eq!(candidate.content, &source[2..]);
    assert!(remaining.is_at_end());
}

#[test]
fn explicit_continuation_ends_only_on_standalone_dot() {
    let content = "foo &\n  value.field\n  . extra\n  \".\"\n  .";
    let source = format!("- {content}\nnext");

    let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(candidate.content, content);
    assert_eq!(remaining.peek(), Some('\n'));
}

#[test]
fn embedded_ampersand_and_escaped_delimiters_do_not_continue_line() {
    for content in [r"foo \{ \[ \( ", "foo a & b"] {
        let source = format!("- {content}\nnext");

        let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

        assert_eq!(candidate.content, content);
        assert_eq!(remaining.peek(), Some('\n'));
    }
}

#[test]
fn multiline_crlf_and_borrowed_source_are_preserved() {
    let source = String::from("# foo {\r\n  я\r\n}\r\nnext");

    let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(candidate.mode, DirectiveMode::Extended);
    assert_eq!(candidate.body, "{\r\n  я\r\n}");
    assert_eq!(candidate.content.as_ptr(), source[2..].as_ptr());
    assert_eq!(remaining.peek(), Some('\r'));
}

#[test]
fn unfinished_delimiters_and_continuations_report_eof() {
    let cases: Vec<_> = [("foo {\n", '}'), ("foo [\n", ']'), ("foo (\n", ')'), ("foo &\n", '.'), ("foo \"\n", '"')]
        .into_iter().map(|(content, expected)| (format!("- {content}"), expected)).collect();
    let parser = jade_like_directive_candidate("", None);

    for (source, expected) in &cases {
        let error = parser.parse(ParserInput::new(source)).unwrap_err();

        assert_eq!(error.byte_position(), source.len());
        assert_eq!(error.location().found_char, None);

        let error = (&*error as &dyn std::error::Error).downcast_ref::<JadeLikeBodyError>().unwrap();

        assert_eq!(error.expected, Some(*expected));
    }

    // A failed parse must not leak its stack into the next attempt.
    let (candidate, _) = parser.parse(ParserInput::new("- foo []")).unwrap();

    assert_eq!(candidate.body, "[]");
}

#[test]
fn mismatched_closer_reports_original_byte_position() {
    let source = "- foo ([)]";

    let error = jade_like_directive_candidate("", None).parse(ParserInput::new(source)).unwrap_err();

    assert_eq!(error.byte_position(), source.find(')').unwrap());
    assert_eq!(error.location().found_char, Some(')'));

    let error = (&*error as &dyn std::error::Error).downcast_ref::<JadeLikeBodyError>().unwrap();

    assert_eq!(error.expected, Some(']'));
}

#[test]
fn jade_like_preserves_indentation_and_extended_mode() {
    for (marker, mode) in [("-", DirectiveMode::Regular), ("#", DirectiveMode::Extended)] {
        let source = format!(" \t{marker} foo value  \nnext");

        let parser = flat_map(take(|ch, _, _| ch == ' ' || ch == '\t', 0..), |indent, input| {
            jade_like_directive_candidate(indent, None).parse(input)
        });

        let (candidate, remaining) = parser.parse(ParserInput::new(&source)).unwrap();

        assert_eq!(candidate.form, DirectiveForm::JadeLike { indent: " \t" });
        assert_eq!(candidate.mode, mode);
        assert_eq!(candidate.name, Some("foo"));
        assert_eq!(candidate.body, "value  ");
        assert_eq!(candidate.content, "foo value  ");
        assert_eq!(remaining.peek(), Some('\n'));
    }
}

#[test]
fn jade_like_accepts_line_start_after_newline_and_preserves_crlf() {
    let source = "text\n  - foo\r\nnext";
    let mut input = ParserInput::new(source);
    input.set_byte_position(7);

    let (candidate, remaining) = jade_like_directive_candidate(&source[5..7], None).parse(input).unwrap();

    assert_eq!(candidate.form, DirectiveForm::JadeLike { indent: "  " });
    assert_eq!(candidate.name, None);
    assert_eq!(candidate.body, "foo");
    assert_eq!(remaining.peek(), Some('\r'));
}

#[test]
fn jade_like_requires_marker_separator_and_content() {
    for source in ["#{foo}", "-foo", "- \nnext", "#\nnext"] {
        assert!(jade_like_directive_candidate("", None).parse(ParserInput::new(source)).is_err());
    }

    let mut input = ParserInput::new("text - foo");
    input.set_byte_position(5);

    // Line position is the caller's responsibility; parsing starts at the marker.
    let (candidate, remaining) = jade_like_directive_candidate("", None).parse(input).unwrap();

    assert_eq!(candidate.content, "foo");
    assert!(remaining.is_at_end());
}

#[test]
fn jade_like_content_borrows_source_and_keeps_shorthands_unresolved() {
    let source = String::from("- += foo");

    let (candidate, remaining) = jade_like_directive_candidate("", None).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(candidate.name, None);
    assert_eq!(candidate.body, "+= foo");
    assert_eq!(candidate.content.as_ptr(), source[2..].as_ptr());
    assert!(remaining.is_at_end());
}

#[test]
fn start_parses_only_marker_and_separator() {
    for (source, mode) in [("- foo", DirectiveMode::Regular), ("#\tfoo", DirectiveMode::Extended)] {
        let (actual, remaining) = jade_like_directive_start(None).parse(ParserInput::new(source)).unwrap();

        assert_eq!(actual, mode);
        assert_eq!(remaining.peek(), Some('f'));
    }

    assert!(jade_like_directive_start(None).parse(ParserInput::new("  - foo")).is_err());
}
#[test]
fn registered_openings_require_a_horizontal_separator_and_choose_longest_match() {
    use crate::core::directives::{Directive, DirectiveContext, Shorthand};

    let mut context = DirectiveContext::default();
    let shorthand = Shorthand::new();

    for declaration in [
        "tag { opening: \"<\", closing: \">\" }",
        "tag { opening: \"<<\" }",
        "tag { opening: \"\u{2192}\" }",
    ] {
        shorthand.begin(&mut context, ParserInput::new(declaration)).unwrap();
    }

    let parser = jade_like_directive_start(Some(&context.shorthands));

    for source in ["< div", "<<\tdiv", "\u{2192} div"] {
        let (mode, remaining) = parser.parse(ParserInput::new(source)).unwrap();

        assert_eq!(mode, DirectiveMode::Regular);
        assert_eq!(remaining.source_slice(remaining.byte_position()..source.len()), "div");
    }

    for source in ["<div>", "<<div", "<\ndiv", "<", "? div"] {
        assert!(parser.parse(ParserInput::new(source)).is_err(), "{source:?}");
    }

    let (mode, _) = parser.parse(ParserInput::new("# foo")).unwrap();

    assert_eq!(mode, DirectiveMode::Extended);
    assert!(jade_like_directive_start(None).parse(ParserInput::new("< div")).is_err());
}
