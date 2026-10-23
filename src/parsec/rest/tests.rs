use crate::parsec::{input_end, ltrim, seq, ParserInput};

use super::*;

#[test]
fn consumes_unicode_remainder_from_the_current_position_without_copying() {
    let source = "prefix \u{0436}\n\u{1f600}";
    let mut input = ParserInput::new(source);
    input.set_byte_position("prefix ".len());

    let (content, remaining) = rest().parse(input).unwrap();

    assert_eq!(content, "\u{0436}\n\u{1f600}");
    assert_eq!(content.as_ptr(), source["prefix ".len()..].as_ptr());
    assert_eq!(remaining.byte_position(), source.len());
    assert!(remaining.is_at_end());
}

#[test]
fn succeeds_on_empty_input_and_can_be_used_again_at_end() {
    let parser = rest();

    for source in ["", "text"] {
        let (_, remaining) = parser.parse(ParserInput::new(source)).unwrap();
        let (content, remaining) = parser.parse(remaining).unwrap();

        assert_eq!(content, "");
        assert_eq!(remaining.byte_position(), source.len());
        assert!(remaining.is_at_end());
    }
}

#[test]
fn respects_bounded_input_and_composes_with_whitespace_and_end_anchor() {
    let source = "before  body  after";
    let start = "before".len();
    let end = "before  body  ".len();
    let input = ParserInput::with_offset(&source[start..end], start);
    let parser = seq(ltrim(rest()), input_end());

    let ((content, ()), remaining) = parser.parse(input).unwrap();

    assert_eq!(content, "body  ");
    assert_eq!(content.as_ptr(), source["before  ".len()..].as_ptr());
    assert_eq!(remaining.byte_position(), end);
    assert!(remaining.is_at_end());
}
