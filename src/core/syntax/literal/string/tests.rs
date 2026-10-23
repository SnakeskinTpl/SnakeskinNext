use crate::parsec::prelude::*;
use crate::core::syntax::identifier;

use super::*;

fn complete(source: &str) -> Result<Cow<'_, str>, Box<dyn crate::parsec::ParseError>> {
    let ((value, ()), _) = seq(string(), ltrim(input_end())).parse(ParserInput::new(source))?;
    Ok(value)
}

#[test]
fn plain_strings_accept_both_quotes_and_borrow_source_text() {
    for source in ["\"hello\"tail", "'hello'tail", "\"\u{0436}\"tail", "''tail"] {
        let (value, remaining) = string().parse(ParserInput::new(source)).unwrap();
        let expected = &source[1..source.len() - 5];

        assert_eq!(value, expected);
        assert!(matches!(value, Cow::Borrowed(_)));
        assert_eq!(value.as_ptr(), expected.as_ptr());
        assert_eq!(remaining.source_slice(remaining.byte_position()..source.len()), "tail");
    }
}

#[test]
fn decodes_escapes_and_unicode_in_both_quote_forms() {
    let cases = [
        (r#""a\n\t\r\\\"\/\b\f""#, "a\n\t\r\\\"/\u{8}\u{c}"),
        (r"'it\'s'", "it's"),
        (r#""\u0041\uD83D\uDE00""#, "A\u{1f600}"),
    ];

    for (source, expected) in cases {
        let value = complete(source).unwrap();

        assert_eq!(value, expected);
        assert!(matches!(value, Cow::Owned(_)));
    }
}

#[test]
fn raw_strings_preserve_backslashes_and_do_not_escape_the_closing_quote() {
    for source in [r#"r"\n\u0041""#, r"r'\n\u0041'"] {
        let value = complete(source).unwrap();

        assert_eq!(value, r"\n\u0041");
        assert!(matches!(value, Cow::Borrowed(_)));
    }

    let (value, remaining) = string().parse(ParserInput::new("r'path\\'tail")).unwrap();

    assert_eq!(value, "path\\");
    assert_eq!(remaining.peek(), Some('t'));
}

#[test]
fn multiline_strings_preserve_line_breaks_and_relative_indentation() {
    for (source, expected) in [
        ("\"\n  foo\n  bla\n  baz\n\"", "foo\nbla\nbaz"),
        ("'\r\n\tfoo\r\n\t bla\r\n\t baz\r\n  '\t", "foo\r\n bla\r\n baz"),
        ("\"\n  Hello \"World\"\n\"", "Hello \"World\""),
        ("\"\n  foo\\n\n  bar\n\"", "foo\n\nbar"),
        ("r\"\n  foo\\n\n  bar\n\"", "foo\\n\nbar"),
        ("\"\n  foo\n    bar\n\"", "foo\n  bar"),
        ("\"\n  foo\n  \tbar\n\"", "foo\n\tbar"),
        ("\"\n  foo\n\tbar\n\"", "  foo\n\tbar"),
        ("\"\n  foo  \n  bar\n\"", "foo  \nbar"),
        ("\"\n  foo\n  \n  bar\n\"", "foo\n\nbar"),
        ("\"\n\n  foo\n\n\"", "\nfoo\n"),
        ("\"\n\"", ""),
    ] {
        let value = complete(source).unwrap();

        assert_eq!(value, expected, "{source:?}");
    }
}

#[test]
fn explicit_continuations_join_lines_but_raw_and_escaped_backslashes_do_not() {
    for (source, expected) in [
        ("\"\n  foo \\\n    bar\n\"", "foo bar"),
        ("\"\r\n  foo \\ \t\r\n\r\n    bar\r\n\"", "foo bar"),
        ("\"\n  foo\\\\\n  bar\n\"", "foo\\\nbar"),
        ("r\"\n  foo\\\n    bar\n\"", "foo\\\n  bar"),
        ("\"\n  foo\\\n\"", "foo"),
    ] {
        let value = complete(source).unwrap();

        assert_eq!(value, expected, "{source:?}");
    }
}

#[test]
fn malformed_strings_commit_after_the_opening_quote() {
    for source in [
        "\"unclosed", "'unclosed", "r\"unclosed", "\"line\nbreak\"",
        r#""\q""#, r#""\uD800""#, r#""\uDC00""#, r#""\u12""#,
        "\"\n  text\n\" trailing",
    ] {
        let error = string().parse(ParserInput::new(source)).unwrap_err();

        assert!(!error.is_recoverable(), "{source:?}");
    }
}

#[test]
fn escape_errors_keep_original_positions_after_multiline_normalization() {
    for source in ["'\u{0436}\\q'", "\"\n\tfoo\n\t \\q\n\""] {
        let input = ParserInput::with_offset(source, 23);

        let error = string().parse(input).unwrap_err();

        assert_eq!(error.location().failure_byte, 23 + source.find('\\').unwrap());
        assert_eq!(error.location().found_char, Some('\\'));
    }
}

#[test]
fn raw_prefix_is_not_reserved_without_a_quote() {
    let parser = choice(
        value(string(), "string"),
        value(identifier(), "identifier")
    );

    let (kind, remaining) = parser.parse(ParserInput::new("rawValue")).unwrap();

    assert_eq!(kind, "identifier");
    assert!(remaining.is_at_end());
}
