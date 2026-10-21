use std::ops::Bound;

use crate::parsec::*;

use super::*;

#[test]
fn error_exposes_location_and_match_count() {
    for (source, actual) in [("!я?", Some('?')), ("!я", None)] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(1);

        let error = take(|ch, _| ch.is_alphabetic(), 2..).parse(input).unwrap_err();

        assert_eq!(*error.location(), ErrorLocation { start_byte: 1, failure_byte: 3, found_char: actual });
        assert_eq!(error.byte_position(), error.location().failure_byte);

        let error = (&*error as &dyn std::error::Error).downcast_ref::<TakeError>().unwrap();

        assert_eq!(error.min, 2);
        assert_eq!(error.matched, 1);
        assert!(error.to_string().contains("matched 1"));
    }
}

#[test]
fn bounds_limit_character_count() {
    for (range, expected) in [(0..1, ""), (1..3, "ab"), (3..4, "abc")] {
        let parser = take(|_, _| true, range);

        let (output, remaining) = parser.parse(ParserInput::new("abcd")).unwrap();

        assert_eq!(output, expected);
        assert_eq!(remaining.byte_position(), expected.len());
    }
}

#[test]
fn inclusive_and_unbounded_ranges_are_supported() {
    let (output, _) = take(|_, _| true, 3..=3).parse(ParserInput::new("abcd")).unwrap();

    assert_eq!(output, "abc");

    let (output, _) = take(|_, _| true, ..).parse(ParserInput::new("abcd")).unwrap();

    assert_eq!(output, "abcd");

    let (output, remaining) = take(|_, _| true, ..=0).parse(ParserInput::new("a")).unwrap();

    assert_eq!(output, "");
    assert_eq!(remaining.byte_position(), 0);
}

#[test]
fn unicode_output_borrows_source_at_current_position() {
    let source = String::from("!я🦀x");
    let mut input = ParserInput::new(&source);
    input.set_byte_position(1);

    let (output, remaining) = take(|_, _| true, 2..=2).parse(input).unwrap();

    assert_eq!(output, "я🦀");
    assert_eq!(output.as_ptr(), source[1..].as_ptr());
    assert_eq!(remaining.byte_position(), 7);
    assert_eq!(remaining.peek(), Some('x'));
}

#[test]
fn rejected_character_is_not_consumed() {
    let (output, remaining) = take(|ch, _| ch.is_alphabetic(), 1..)
        .parse(ParserInput::new("abc!"))
        .unwrap();

    assert_eq!(output, "abc");
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn insufficient_match_reports_byte_position() {
    for source in ["я!", "я"] {
        let error = take(|ch, _| ch.is_alphabetic(), 2..)
            .parse(ParserInput::new(source))
            .unwrap_err();

        assert_eq!(error.byte_position(), 2);
    }
}

#[test]
fn empty_input_succeeds_only_when_zero_matches_are_allowed() {
    let (output, remaining) = take(|_, _| true, ..).parse(ParserInput::new("")).unwrap();

    assert_eq!(output, "");
    assert!(remaining.is_at_end());

    let error = take(|_, _| true, 1..).parse(ParserInput::new("")).unwrap_err();

    assert_eq!(error.byte_position(), 0);
}

#[test]
fn odd_backslash_runs_escape_delimiters_and_even_runs_do_not() {
    for (source, expected) in [
        (r#"a\"b"rest"#, r#"a\"b"#),
        (r#"a\\\"b"rest"#, r#"a\\\"b"#),
        (r#"a\\"rest"#, r#"a\\"#),
    ] {
        let parser = take(|ch, escaped| ch != '"' || escaped, ..);

        let (output, remaining) = parser.parse(ParserInput::new(source)).unwrap();

        assert_eq!(output, expected);
        assert_eq!(remaining.peek(), Some('"'));
    }
}

#[test]
fn escape_tracking_starts_at_current_position() {
    let mut input = ParserInput::new("\\x");
    input.set_byte_position(1);

    let (output, remaining) = take(|_, escaped| !escaped, ..).parse(input).unwrap();

    assert_eq!(output, "x");
    assert!(remaining.is_at_end());
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn exclusive_zero_upper_bound_is_rejected() {
    take(|_, _| true, ..0);
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn exclusive_maximum_lower_bound_is_rejected() {
    take(|_, _| true, (Bound::Excluded(usize::MAX), Bound::Unbounded));
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn empty_count_range_is_rejected() {
    take(|_, _| true, 3..3);
}
