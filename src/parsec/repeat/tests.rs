use std::ops::Bound;

use crate::parsec::*;

use super::*;

#[test]
fn insufficient_matches_exposes_counts_and_cause() {
    let error = repeat(tag("ab"), 2..).parse(ParserInput::new("abax")).unwrap_err();
    let error = (&*error as &dyn std::error::Error).downcast_ref::<RepeatError>().unwrap();

    match error.kind() {
        RepeatErrorKind::InsufficientMatches { min, matched, cause } => {
            assert_eq!(*min, 2);
            assert_eq!(*matched, 1);
            assert_eq!(cause.byte_position(), 3);
        }
        other => panic!("Expected insufficient matches, got {other:?}"),
    }

    assert!(error.source().unwrap().is::<TagError>());
    assert!(error.to_string().contains("matched 1"));
}

#[test]
fn no_progress_has_distinct_kind_and_no_source() {
    let error = repeat(succeed(), ..).parse(ParserInput::new("x")).unwrap_err();
    let error = (&*error as &dyn std::error::Error).downcast_ref::<RepeatError>().unwrap();

    assert!(matches!(error.kind(), RepeatErrorKind::NoProgress));
    assert!(error.source().is_none());
    assert_eq!(error.byte_position(), 0);
}

#[test]
fn minimum_failure_preserves_furthest_position_and_inner_cause() {
    for (source, actual) in [("abcabX", Some('X')), ("abcab", None)] {
        let error = repeat(tag("abc"), 2..).parse(ParserInput::new(source)).unwrap_err();

        assert_eq!(*error.location(), ErrorLocation { start_byte: 0, failure_byte: 5, found_char: actual });

        let cause = error.source().unwrap().downcast_ref::<TagError>().unwrap();

        assert_eq!(cause.location().start_byte, 3);
        assert_eq!(cause.location().failure_byte, 5);
        assert_eq!(cause.expected_tag, "abc");
        assert_eq!(cause.expected, 'c');
        assert!(error.to_string().contains("abc"));
    }
}

#[test]
fn choice_compares_the_failed_iteration_position() {
    let parser = or(repeat(tag("abc"), 2..), tag("abcaY"));

    let error = parser.parse(ParserInput::new("abcabX")).unwrap_err();

    assert_eq!(error.byte_position(), 5);
    assert!(error.source().unwrap().is::<TagError>());
}

#[test]
fn met_minimum_restores_input_before_failed_iteration() {
    let (output, remaining) = repeat(tag("abc"), 1..)
        .parse(ParserInput::new("abcabX"))
        .unwrap();

    assert_eq!(output, ["abc"]);
    assert_eq!(remaining.byte_position(), 3);
    assert_eq!(remaining.peek(), Some('a'));
}

#[test]
fn count_bounds_limit_successful_iterations() {
    for (range, expected_count) in [(0..1, 0), (1..3, 2), (3..4, 3)] {
        let parser = repeat(tag("я"), range);

        let (output, remaining) = parser.parse(ParserInput::new("яяя!")).unwrap();

        assert_eq!(output, vec!["я"; expected_count]);
        assert_eq!(remaining.byte_position(), expected_count * 2);
    }
}

#[test]
fn inclusive_and_unbounded_ranges_are_supported() {
    let (output, remaining) = repeat(tag("a"), 2..=2).parse(ParserInput::new("aaa")).unwrap();

    assert_eq!(output, ["a", "a"]);
    assert_eq!(remaining.byte_position(), 2);

    let (output, remaining) = repeat(tag("a"), ..).parse(ParserInput::new("aaa!")).unwrap();

    assert_eq!(output, ["a", "a", "a"]);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn zero_maximum_does_not_run_inner_parser() {
    let (output, remaining) = repeat(succeed(), ..=0).parse(ParserInput::new("a")).unwrap();

    assert!(output.is_empty());
    assert_eq!(remaining.byte_position(), 0);
}

#[test]
fn failed_attempt_restores_cursor_after_partial_match() {
    let (output, remaining) = repeat(tag("ab"), ..).parse(ParserInput::new("abax")).unwrap();

    assert_eq!(output, ["ab"]);
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('a'));
}

#[test]
fn minimum_failure_reports_repetition_start_and_current_position() {
    let mut input = ParserInput::new("!яx");
    input.set_byte_position(1);

    let error = repeat(tag("я"), 2..).parse(input).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start_byte: 1, failure_byte: 3, found_char: Some('x') });
}

#[test]
fn empty_input_is_allowed_only_for_zero_minimum() {
    let (output, remaining) = repeat(tag("a"), ..).parse(ParserInput::new("")).unwrap();

    assert!(output.is_empty());
    assert!(remaining.is_at_end());

    let error = repeat(tag("a"), 1..).parse(ParserInput::new("")).unwrap_err();

    assert_eq!(error.location().found_char, None);
    assert_eq!(error.byte_position(), 0);
}

#[test]
#[should_panic(expected = "Invalid repetition range")]
fn exclusive_zero_upper_bound_is_rejected() {
    repeat(tag("a"), ..0);
}

#[test]
#[should_panic(expected = "Invalid repetition range")]
fn exclusive_maximum_lower_bound_is_rejected() {
    repeat(tag("a"), (Bound::Excluded(usize::MAX), Bound::Unbounded));
}

#[test]
#[should_panic(expected = "Invalid repetition range")]
fn empty_count_range_is_rejected() {
    repeat(tag("a"), 3..3);
}
