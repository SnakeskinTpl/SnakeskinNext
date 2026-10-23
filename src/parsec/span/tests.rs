use crate::parsec::prelude::*;
use crate::parsec::CutError;

use super::*;

#[test]
fn spanned_preserves_output_and_counts_original_utf8_byte_offsets() {
    let mut input = ParserInput::new("!я🦀?");
    input.set_byte_position(1);
    let inner = map(tag("я🦀"), |text| text.chars().count());

    let ((span, count), remaining) = spanned(inner).parse(input).unwrap();

    assert_eq!(span, 1..7);
    assert_eq!(count, 2);
    assert_eq!(remaining.byte_position(), 7);
    assert_eq!(remaining.peek(), Some('?'));
}

#[test]
fn recognize_returns_the_entire_consumed_source_without_copying() {
    let source = String::from("!a  b?");
    let mut input = ParserInput::new(&source);
    input.set_byte_position(1);
    let inner = value(seq(tag("a"), ltrim(tag("b"))), 42);

    let (text, remaining) = recognize(inner).parse(input).unwrap();

    assert_eq!(text, "a  b");
    assert_eq!(text.as_ptr(), source[1..].as_ptr());
    assert_eq!(remaining.peek(), Some('?'));
}

#[test]
fn whitespace_is_included_only_when_consumed_by_the_wrapped_parser() {
    let source = "  a";

    let ((including, _), _) = spanned(ltrim(tag("a"))).parse(ParserInput::new(source)).unwrap();
    let ((excluding, _), _) = ltrim(spanned(tag("a"))).parse(ParserInput::new(source)).unwrap();

    assert_eq!(including, 0..3);
    assert_eq!(excluding, 2..3);
}

#[test]
fn empty_success_has_an_empty_span_and_slice_at_the_current_position() {
    let mut input = ParserInput::new("!a");
    input.set_byte_position(1);

    let ((span, _), remaining) = spanned(succeed()).parse(input.clone()).unwrap();
    let (text, recognized_remaining) = recognize(succeed()).parse(input).unwrap();

    assert_eq!(span, 1..1);
    assert_eq!(text, "");
    assert_eq!(remaining.byte_position(), 1);
    assert_eq!(recognized_remaining.byte_position(), 1);
}

#[test]
fn wrappers_preserve_inner_error_location_and_commitment() {
    let mut input = ParserInput::new("!ax");
    input.set_byte_position(1);

    let spanned_error = spanned(cut(tag("ab"))).parse(input.clone()).unwrap_err();
    let recognized_error = recognize(cut(tag("ab"))).parse(input).unwrap_err();

    for error in [spanned_error, recognized_error] {
        assert!(!error.is_recoverable());
        assert_eq!(error.location().start_byte, 1);
        assert_eq!(error.location().failure_byte, 2);
        assert_eq!(error.location().found_char, Some('x'));
        assert!((&*error as &dyn std::error::Error).is::<CutError>());
    }
}
