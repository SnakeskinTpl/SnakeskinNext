use std::ops::Bound;

use super::*;

#[test]
fn not_stops_before_delimiter_and_preserves_source_slice() {
    let source = String::from("!я🦀{rest");
    let mut input = ParserInput::new(&source);
    input.set_byte_position(1);

    let (output, remaining) = take_match(not(tag("{")), 1..).parse(input).unwrap();

    assert_eq!(output, "я🦀");
    assert_eq!(output.as_ptr(), source[1..].as_ptr());
    assert_eq!(remaining.byte_position(), 7);
    assert_eq!(remaining.peek(), Some('{'));
}

#[test]
fn consuming_check_only_advances_scanner_one_character() {
    let (output, remaining) = take_match(tag("aa"), ..).parse(ParserInput::new("aaa!")).unwrap();

    assert_eq!(output, "aa");
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('a'));
}

#[test]
fn zero_width_success_is_safe_and_limits_count_characters() {
    let (output, remaining) = take_match(succeed(), 2..=2).parse(ParserInput::new("я🦀!")).unwrap();

    assert_eq!(output, "я🦀");
    assert_eq!(remaining.peek(), Some('!'));

    let (output, remaining) = take_match(succeed(), ..2).parse(ParserInput::new("abc")).unwrap();

    assert_eq!(output, "a");
    assert_eq!(remaining.byte_position(), 1);
}

#[test]
fn eof_without_delimiter_is_a_valid_stop() {
    let (output, remaining) = take_match(not(tag("{")), ..).parse(ParserInput::new("text")).unwrap();

    assert_eq!(output, "text");
    assert!(remaining.is_at_end());
}

#[test]
fn zero_maximum_does_not_check_parser() {
    let (output, remaining) = take_match(repeat(succeed(), ..), ..=0)
        .parse(ParserInput::new("x")).unwrap();

    assert_eq!(output, "");
    assert_eq!(remaining.byte_position(), 0);
}

#[test]
fn insufficient_checks_preserve_cause_and_counts() {
    let error = take_match(tag("ab"), 2..).parse(ParserInput::new("abX")).unwrap_err();

    assert_eq!(error.byte_position(), 1);
    assert!(error.is_recoverable());

    let cause = error.source().unwrap().downcast_ref::<TagError>().unwrap();

    assert_eq!(cause.byte_position(), 1);

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TakeMatchError>().unwrap();

    assert_eq!(error.min, 2);
    assert_eq!(error.matched, 1);
}

#[test]
fn insufficient_input_at_eof_has_no_check_cause() {
    let error = take_match(succeed(), 2..).parse(ParserInput::new("я")).unwrap_err();

    assert_eq!(error.byte_position(), 2);
    assert_eq!(error.location().actual, None);
    assert!(error.source().is_none());
}

#[test]
fn non_recoverable_error_is_not_a_stop_condition() {
    let error = take_match(repeat(succeed(), ..), ..).parse(ParserInput::new("x")).unwrap_err();

    assert!(!error.is_recoverable());
    assert!((&*error as &dyn std::error::Error).is::<RepeatError>());
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn exclusive_zero_upper_bound_is_rejected() {
    take_match(succeed(), ..0);
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn exclusive_maximum_lower_bound_is_rejected() {
    take_match(succeed(), (Bound::Excluded(usize::MAX), Bound::Unbounded));
}
