use std::cell::{Cell, RefCell};
use std::ops::Bound;

use crate::parsec::*;

use super::*;

#[test]
fn error_exposes_location_and_match_count() {
    for (source, actual) in [("!я?", Some('?')), ("!я", None)] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(1);

        let error = take(|ch, _, _| ch.is_alphabetic(), 2..).parse(input).unwrap_err();

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
        let parser = take(|_, _, _| true, range);

        let (output, remaining) = parser.parse(ParserInput::new("abcd")).unwrap();

        assert_eq!(output, expected);
        assert_eq!(remaining.byte_position(), expected.len());
    }
}

#[test]
fn inclusive_and_unbounded_ranges_are_supported() {
    let (output, _) = take(|_, _, _| true, 3..=3).parse(ParserInput::new("abcd")).unwrap();

    assert_eq!(output, "abc");

    let (output, _) = take(|_, _, _| true, ..).parse(ParserInput::new("abcd")).unwrap();

    assert_eq!(output, "abcd");

    let (output, remaining) = take(|_, _, _| true, ..=0).parse(ParserInput::new("a")).unwrap();

    assert_eq!(output, "");
    assert_eq!(remaining.byte_position(), 0);
}

#[test]
fn unicode_output_borrows_source_at_current_position() {
    let source = String::from("!я🦀x");
    let mut input = ParserInput::new(&source);
    input.set_byte_position(1);

    let (output, remaining) = take(|_, _, _| true, 2..=2).parse(input).unwrap();

    assert_eq!(output, "я🦀");
    assert_eq!(output.as_ptr(), source[1..].as_ptr());
    assert_eq!(remaining.byte_position(), 7);
    assert_eq!(remaining.peek(), Some('x'));
}

#[test]
fn rejected_character_is_not_consumed() {
    let (output, remaining) = take(|ch, _, _| ch.is_alphabetic(), 1..)
        .parse(ParserInput::new("abc!"))
        .unwrap();

    assert_eq!(output, "abc");
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn insufficient_match_reports_byte_position() {
    for source in ["я!", "я"] {
        let error = take(|ch, _, _| ch.is_alphabetic(), 2..)
            .parse(ParserInput::new(source))
            .unwrap_err();

        assert_eq!(error.byte_position(), 2);
    }
}

#[test]
fn empty_input_succeeds_only_when_zero_matches_are_allowed() {
    let (output, remaining) = take(|_, _, _| true, ..).parse(ParserInput::new("")).unwrap();

    assert_eq!(output, "");
    assert!(remaining.is_at_end());

    let error = take(|_, _, _| true, 1..).parse(ParserInput::new("")).unwrap_err();

    assert_eq!(error.byte_position(), 0);
}

#[test]
fn odd_backslash_runs_escape_delimiters_and_even_runs_do_not() {
    for (source, expected) in [
        (r#"a\"b"rest"#, r#"a\"b"#),
        (r#"a\\\"b"rest"#, r#"a\\\"b"#),
        (r#"a\\"rest"#, r#"a\\"#),
    ] {
        let parser = take(|ch, escaped, _| ch != '"' || escaped, ..);

        let (output, remaining) = parser.parse(ParserInput::new(source)).unwrap();

        assert_eq!(output, expected);
        assert_eq!(remaining.peek(), Some('"'));
    }
}

#[test]
fn escape_tracking_starts_at_current_position() {
    let mut input = ParserInput::new("\\x");
    input.set_byte_position(1);

    let (output, remaining) = take(|_, escaped, _| !escaped, ..).parse(input).unwrap();

    assert_eq!(output, "x");
    assert!(remaining.is_at_end());
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn exclusive_zero_upper_bound_is_rejected() {
    take(|_, _, _| true, ..0);
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn exclusive_maximum_lower_bound_is_rejected() {
    take(|_, _, _| true, (Bound::Excluded(usize::MAX), Bound::Unbounded));
}

#[test]
#[should_panic(expected = "Invalid character count range")]
fn empty_count_range_is_rejected() {
    take(|_, _, _| true, 3..3);
}

#[test]
fn predicate_sees_current_character_escaping_and_original_byte_positions() {
    let seen = RefCell::new(Vec::new());
    let mut input = ParserInput::new("!я\\x\\\\!");
    input.set_byte_position(1);
    let parser = try_take(|ch, escaped, input| {
        assert_eq!(input.peek(), Some(ch));
        seen.borrow_mut().push((ch, escaped, input.byte_position()));
        Ok(ch != '!')
    }, 0..);

    let (output, remaining) = parser.parse(input).unwrap();

    assert_eq!(output, "я\\x\\\\");
    assert_eq!(remaining.peek(), Some('!'));
    assert_eq!(*seen.borrow(), [
        ('я', false, 1), ('\\', false, 3), ('x', true, 4),
        ('\\', false, 5), ('\\', true, 6), ('!', false, 7),
    ]);
}

#[test]
fn ordinary_take_receives_input_context_too() {
    let parser = take(|_, _, input| input.byte_position() < 3, 0..);

    let (output, remaining) = parser.parse(ParserInput::new("яab")).unwrap();

    assert_eq!(output, "яa");
    assert_eq!(remaining.byte_position(), 3);
}

#[test]
fn predicate_errors_propagate_without_wrapping_or_changing_recoverability() {
    for fatal in [false, true] {
        let parser = try_take(|ch, _, input| {
            if ch == '!' {
                let location = ErrorLocation {
                    start_byte: input.byte_position(),
                    failure_byte: input.byte_position(),
                    found_char: Some(ch),
                };
                return Err(if fatal {
                    RepeatError::no_progress(location) as Box<dyn ParseError>
                } else {
                    TagError::new(location, "x", 'x') as Box<dyn ParseError>
                });
            }
            Ok(true)
        }, 5..);

        let error = parser.parse(ParserInput::new("я!")).unwrap_err();

        assert_eq!(error.byte_position(), 2);
        assert_eq!(error.is_recoverable(), !fatal);
        if fatal {
            assert!((&*error as &dyn std::error::Error).is::<RepeatError>());
        } else {
            assert!((&*error as &dyn std::error::Error).is::<TagError>());
        }
    }
}

#[test]
fn false_and_eof_before_minimum_produce_take_error() {
    for source in ["я!", "я"] {
        let error = try_take(|ch, _, _| Ok(ch != '!'), 2..)
            .parse(ParserInput::new(source)).unwrap_err();

        assert_eq!(error.byte_position(), 2);

        let error = (&*error as &dyn std::error::Error).downcast_ref::<TakeError>().unwrap();

        assert_eq!(error.min, 2);
        assert_eq!(error.matched, 1);
    }
}

#[test]
fn limits_count_characters_and_output_borrows_source() {
    let source = String::from("я🦀!");

    let (output, remaining) = try_take(|_, _, _| Ok(true), 2..=2)
        .parse(ParserInput::new(&source)).unwrap();

    assert_eq!(output, "я🦀");
    assert_eq!(output.as_ptr(), source.as_ptr());
    assert_eq!(remaining.byte_position(), 6);
}

#[test]
fn eof_and_zero_limit_do_not_call_predicate() {
    let calls = Cell::new(0);
    let check = |_, _, _: &ParserInput<'_>| {
        calls.set(calls.get() + 1);
        Ok(true)
    };

    try_take(check, 0..).parse(ParserInput::new("")).unwrap();
    try_take(check, ..=0).parse(ParserInput::new("x")).unwrap();

    assert_eq!(calls.get(), 0);
}
