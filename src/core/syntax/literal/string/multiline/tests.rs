use super::*;

#[test]
fn physical_line_keeps_indentation_line_ending_and_original_byte_range() {
    let source = " \t\u{0436}\r\nnext";
    let input = ParserInput::with_offset(source, 20);

    let (line, remaining) = physical_line().parse(input).unwrap();

    assert_eq!(line.indent, " \t");
    assert_eq!(line.text, " \t\u{0436}");
    assert_eq!(line.line_break, "\r\n");
    assert_eq!(line.span, 20..24);
    assert_eq!(line.text.as_ptr(), source.as_ptr());

    assert_eq!(remaining.byte_position(), 26);
    assert_eq!(remaining.peek(), Some('n'));
}

#[test]
fn framing_lines_are_discarded_but_blank_content_lines_remain() {
    let source = " \t\r\n  foo\n\t \n  ";

    let (parsed, remaining) = lines().parse(ParserInput::new(source)).unwrap();

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].text, "  foo");
    assert_eq!(parsed[0].line_break, "\n");
    assert_eq!(parsed[1].text, "\t ");
    assert_eq!(parsed[1].indent, "\t ");

    assert!(remaining.is_at_end());
}

#[test]
fn line_grammar_requires_opening_newline_and_only_indentation_before_closing_quote() {
    for source in ["foo\n", "\n  foo\nnot indentation", "\n  foo"] {
        assert!(lines().parse(ParserInput::new(source)).is_err(), "{source:?}");
    }

    for source in ["\n", " \t\r\n  "] {
        let (parsed, remaining) = lines().parse(ParserInput::new(source)).unwrap();

        assert!(parsed.is_empty());
        assert!(remaining.is_at_end());
    }
}

#[test]
fn common_indent_ignores_blank_lines_and_preserves_additional_indentation() {
    let source = "\n    foo\n \t\n      bar\n    ";
    let (parsed, _) = lines().parse(ParserInput::new(source)).unwrap();

    let indent = common_indent(&parsed);
    let rendered = multiline_value(source, 0, false).unwrap();

    assert_eq!(indent, "    ");
    assert_eq!(rendered, "foo\n\n  bar");
}

#[test]
fn common_indent_compares_characters_instead_of_visual_width_or_length() {
    for (source, expected_indent, expected_value) in [
        ("\n  foo\n\tbar\n", "", "  foo\n\tbar"),
        ("\n \tfoo\n  bar\n", " ", "\tfoo\n bar"),
        ("\n\t foo\n\t\tbar\n", "\t", " foo\n\tbar"),
        ("\nfoo\n  bar\n", "", "foo\n  bar"),
        ("\n \t\n  \n", "", "\n"),
    ] {
        let (parsed, _) = lines().parse(ParserInput::new(source)).unwrap();

        assert_eq!(common_indent(&parsed), expected_indent, "{source:?}");

        let value = multiline_value(source, 0, false).unwrap();

        assert_eq!(value, expected_value, "{source:?}");
    }
}

#[test]
fn line_join_accepts_only_one_backslash_and_optional_horizontal_whitespace() {
    for source in ["\\", "\\ \t"] {
        let ((), remaining) = line_join().parse(ParserInput::new(source)).unwrap();

        assert!(remaining.is_at_end());
    }

    for source in ["", "\\\\", "\\text", "\\ \n", "\\\u{2003}"] {
        assert!(line_join().parse(ParserInput::new(source)).is_err(), "{source:?}");
    }
}

#[test]
fn line_content_removes_only_an_unescaped_trailing_line_join() {
    for (source, expected, joins) in [
        ("foo", "foo", false),
        ("foo \\ \t", "foo ", true),
        (r"foo\\", r"foo\\", false),
        (r"foo\\\", r"foo\\", true),
        (r"foo\n", r"foo\n", false),
        (r"foo\ bar", r"foo\ bar", false),
    ] {
        let ((text, joins_next_line), remaining) = line_content().parse(ParserInput::new(source)).unwrap();

        assert_eq!(text, expected, "{source:?}");
        assert_eq!(joins_next_line, joins, "{source:?}");
        assert!(remaining.is_at_end());
    }
}

#[test]
fn line_join_skips_blank_lines_and_extra_indentation_without_losing_preceding_spaces() {
    let content = "\r\n  Hello \\ \t\r\n  \r\n\t\r\n    world!\r\n  ";

    let value = multiline_value(content, 0, false).unwrap();

    assert_eq!(value, "Hello world!");
}

#[test]
fn raw_mode_preserves_backslashes_while_applying_the_same_layout_rules() {
    let content = "\n  foo\\\n    bar\\n\n  ";

    let value = multiline_value(content, 0, true).unwrap();

    assert_eq!(value, "foo\\\n  bar\\n");
}

#[test]
fn internal_line_endings_and_trailing_spaces_are_preserved() {
    let content = "\n  foo  \r\n    bar\n\n  ";

    let value = multiline_value(content, 0, false).unwrap();

    assert_eq!(value, "foo  \r\n  bar\n");
}

#[test]
fn decode_failure_after_line_join_points_to_original_source() {
    let content = "\n  \u{0436} \\\n\n      \\q\n  ";
    let offset = 37;
    let invalid_escape = content.rfind("\\q").unwrap();

    let error = multiline_value(content, offset, false).unwrap_err();

    assert_eq!(error.location().failure_byte, offset + invalid_escape);
    assert_eq!(error.location().found_char, Some('\\'));
    assert!(!error.is_recoverable());
}
