use crate::parsec::*;

use super::*;

#[test]
fn successful_match_preserves_original_position() {
    let mut input = ParserInput::new("!я🦀rest");
    input.set_byte_position(1);

    let ((), remaining) = peek(tag("я🦀")).parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 1);
    assert_eq!(remaining.peek(), Some('я'));
}

#[test]
fn failure_preserves_inner_error_context() {
    let mut input = ParserInput::new("!яx");
    input.set_byte_position(1);

    let error = peek(tag("яb")).parse(input).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start_byte: 1, failure_byte: 3, found_char: Some('x') });

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

    assert_eq!(error.expected_tag, "яb");
    assert_eq!(error.expected, 'b');
}

#[test]
fn incomplete_match_reports_end_of_input() {
    let error = peek(tag("ab")).parse(ParserInput::new("a")).unwrap_err();

    assert_eq!(error.byte_position(), 1);
    assert_eq!(error.location().found_char, None);
}

#[test]
fn empty_success_is_allowed_at_end_of_input() {
    let ((), remaining) = peek(succeed()).parse(ParserInput::new("")).unwrap();

    assert_eq!(remaining.byte_position(), 0);
    assert!(remaining.is_at_end());
}

#[test]
fn sequence_after_peek_reads_original_input() {
    let parser = seq(peek(tag("ab")), tag("ab"));

    let (((), output), remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert_eq!(output, "ab");
    assert!(remaining.is_at_end());
}
