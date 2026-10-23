use crate::core::syntax::identifier;
use crate::core::syntax::json::{json, JsonKind};
use crate::core::syntax::literal::{literal, Literal, LiteralKind};

use super::*;

#[test]
fn null_leaves_the_following_separator_unconsumed() {
    for source in ["null", "null)", "null,", "null == value"] {
        let ((), remaining) = null().parse(ParserInput::new(source)).unwrap();

        assert_eq!(remaining.byte_position(), 4);
        assert_eq!(remaining.source_slice(4..source.len()), &source[4..]);
    }
}

#[test]
fn null_does_not_match_identifier_prefixes_or_other_spellings() {
    for source in ["nullable", "null_", "null1", "null\u{0436}", "Null", "NULL", "nul", ""] {
        let error = null().parse(ParserInput::new(source)).unwrap_err();

        assert!(error.is_recoverable(), "{source:?}");
    }

    let parser = choice(value(null(), "literal"), value(identifier(), "identifier"));

    let (kind, remaining) = parser.parse(ParserInput::new("nullable")).unwrap();

    assert_eq!(kind, "identifier");
    assert!(remaining.is_at_end());
}

#[test]
fn null_literal_preserves_absolute_span_and_excludes_whitespace() {
    let input = ParserInput::with_offset(" \tnull tail", 20);

    let (parsed, remaining) = literal().parse(input).unwrap();

    assert_eq!(parsed, Literal { span: 22..26, kind: LiteralKind::Null });

    assert_eq!(remaining.source_slice(parsed.span), "null");
    assert_eq!(remaining.peek(), Some(' '));
}

#[test]
fn configuration_uses_the_same_null_grammar_and_keyword_boundary() {
    let (parsed, remaining) = json().parse(ParserInput::new("null]")).unwrap();

    assert_eq!(parsed.kind, JsonKind::Null);
    assert_eq!(parsed.span, 0..4);
    assert_eq!(remaining.peek(), Some(']'));

    for source in ["nullable", "[nullValue]", "{value: null_}"] {
        assert!(json().parse(ParserInput::new(source)).is_err(), "{source:?}");
    }
}

#[test]
fn invalid_suffix_reports_the_original_position() {
    let input = ParserInput::with_offset("null_", 30);

    let error = null().parse(input).unwrap_err();

    assert_eq!(error.location().failure_byte, 34);
    assert_eq!(error.location().found_char, Some('_'));
}
