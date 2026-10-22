use crate::parsec::{ltrim, map, opt, seq, tag};

use super::*;

#[test]
fn input_end_matches_empty_input_and_unicode_eof_without_advancing() {
    for source in ["", "я🦀"] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(source.len());

        let ((), remaining) = input_end().parse(input).unwrap();

        assert_eq!(remaining.byte_position(), source.len());
        assert!(remaining.is_at_end());
    }
}

#[test]
fn input_end_reports_the_current_character_and_allows_optional_rollback() {
    let mut input = ParserInput::new("!я");
    input.set_byte_position(1);

    let error = input_end().parse(input.clone()).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation {
        start_byte: 1,
        failure_byte: 1,
        found_char: Some('я'),
    });

    assert!(error.is_recoverable());
    assert!((&*error as &dyn std::error::Error).is::<InputEndError>());

    let (output, remaining) = opt(input_end()).parse(input).unwrap();

    assert_eq!(output, None);
    assert_eq!(remaining.byte_position(), 1);
}

#[test]
fn trimmed_input_end_accepts_whitespace_but_rejects_trailing_tokens() {
    let grammar = seq(tag("value"), ltrim(input_end()));

    let ((output, ()), remaining) = grammar.parse(ParserInput::new("value \t\r\n")).unwrap();

    assert_eq!(output, "value");
    assert!(remaining.is_at_end());

    let error = grammar.parse(ParserInput::new("value extra")).unwrap_err();

    assert_eq!(error.byte_position(), 6);
    assert_eq!(error.location().found_char, Some('e'));
}

#[test]
fn input_end_honors_offset_slice_bounds() {
    let mut input = ParserInput::with_offset("я", 10);

    let error = input_end().parse(input.clone()).unwrap_err();

    assert_eq!(error.byte_position(), 10);

    input.next();

    let ((), remaining) = input_end().parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 12);
}

#[test]
fn succeeds_at_start_including_empty_input_without_advancing() {
    for source in ["", "я"] {
        let ((), remaining) = input_start().parse(ParserInput::new(source)).unwrap();

        assert_eq!(remaining.byte_position(), 0);
        assert_eq!(remaining.peek(), source.chars().next());
    }
}

#[test]
fn rejects_nonzero_position_including_end_of_input() {
    for source in ["яx", "я"] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(2);

        let error = input_start().parse(input).unwrap_err();

        assert_eq!(*error.location(), ErrorLocation {
            start_byte: 2,
            failure_byte: 2,
            found_char: source[2..].chars().next(),
        });
        assert!(error.is_recoverable());
    }
}

#[test]
fn optional_anchor_leaves_nonzero_position_unchanged() {
    let mut input = ParserInput::new("яx");
    input.set_byte_position(2);

    let (output, remaining) = opt(input_start()).parse(input).unwrap();

    assert_eq!(output, None);
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('x'));
}

#[test]
fn anchor_composes_with_consuming_parser() {
    let parser = map(seq(input_start(), tag("я")), |((), value)| value);

    let (output, remaining) = parser.parse(ParserInput::new("я!")).unwrap();

    assert_eq!(output, "я");
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('!'));
}
