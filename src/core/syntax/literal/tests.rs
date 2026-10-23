use crate::core::syntax::identifier;
use crate::parsec::{choice, value, ParserInput};

use super::*;

#[test]
fn string_literals_preserve_span_and_leave_the_following_expression_unread() {
    let source = " \tr'\u{0436}\\n' + tail";
    let input = ParserInput::with_offset(source, 11);

    let (value, remaining) = literal().parse(input).unwrap();
    let end = source.find(" +").unwrap();

    assert_eq!(value.span, 13..11 + end);
    assert_eq!(value.kind, LiteralKind::String("\u{0436}\\n".into()));

    assert_eq!(remaining.source_slice(remaining.byte_position()..11 + source.len()), " + tail");
}

#[test]
fn parses_boolean_keywords_and_preserves_trailing_input() {
    for (source, expected, end) in [
        ("true", true, 4),
        ("false", false, 5),
        ("true && flag", true, 4),
        ("false)", false, 5),
    ] {
        let (value, remaining) = literal().parse(ParserInput::new(source)).unwrap();

        assert_eq!(value, Literal { span: 0..end, kind: LiteralKind::Bool(expected) });

        assert_eq!(remaining.byte_position(), end);
        assert_eq!(remaining.source_slice(end..source.len()), &source[end..]);
    }
}

#[test]
fn rejects_identifier_prefixes_and_non_boolean_values() {
    for source in [
        "trueValue", "false_", "true1", "false\u{0436}",
        "True", "FALSE", "tru", "", "null",
    ] {
        let error = literal().parse(ParserInput::new(source)).unwrap_err();

        assert!(error.is_recoverable(), "{source:?}");
    }
}

#[test]
fn literal_mismatch_allows_fallback_to_an_identifier() {
    let parser = choice(
        value(literal(), "literal"),
        value(identifier(), "identifier")
    );

    let (kind, remaining) = parser.parse(ParserInput::new("trueValue")).unwrap();

    assert_eq!(kind, "identifier");
    assert!(remaining.is_at_end());
}

#[test]
fn spans_exclude_leading_whitespace_and_keep_enclosing_source_offsets() {
    let input = ParserInput::with_offset(" \tfalse\n", 13);

    let (value, remaining) = literal().parse(input).unwrap();

    assert_eq!(value, Literal { span: 15..20, kind: LiteralKind::Bool(false) });

    assert_eq!(remaining.source_slice(value.span), "false");
    assert_eq!(remaining.peek(), Some('\n'));
}

#[test]
fn invalid_keyword_suffix_reports_the_original_position() {
    let input = ParserInput::with_offset(" true_", 10);

    let error = literal().parse(input).unwrap_err();

    assert_eq!(error.location().failure_byte, 15);
    assert_eq!(error.location().found_char, Some('_'));
}
