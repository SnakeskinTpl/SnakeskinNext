use crate::parsec::*;

use super::*;

#[test]
fn alternative_location_covers_whole_choice_and_preserves_branch_starts() {
    let first = seq(tag("a"), tag("bc"));
    let second = seq(tag("ab"), tag("d"));
    let mut input = ParserInput::new("!abX");
    input.set_byte_position(1);

    let error = or(first, second).parse(input).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start_byte: 1, failure_byte: 3, found_char: Some('X') });

    let error = (&*error as &dyn std::error::Error).downcast_ref::<AlternativeError>().unwrap();

    assert_eq!(error.first.location().start_byte, 2);
    assert_eq!(error.second.location().start_byte, 3);
}

#[test]
fn first_success_wins_even_when_second_would_match_more() {
    let parser = or(tag("a"), tag("ab"));

    let (output, remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert_eq!(output, Either::Left("a"));
    assert_eq!(remaining.peek(), Some('b'));
}

#[test]
fn second_branch_starts_at_original_position_after_partial_match() {
    let parser = or(tag("яa"), tag("яb"));

    let (output, remaining) = parser.parse(ParserInput::new("яb!")).unwrap();

    assert_eq!(output, Either::Right("яb"));
    assert_eq!(remaining.byte_position(), 3);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn furthest_error_wins_regardless_of_branch_order() {
    for parser in [or(tag("template"), tag("var")), or(tag("var"), tag("template"))] {
        let error = parser.parse(ParserInput::new("templX")).unwrap_err();

        assert_eq!(error.byte_position(), 5);

        let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

        assert_eq!(error.expected_tag, "template");
    }
}

#[test]
fn equal_positions_preserve_both_errors() {
    let error = or(tag("if"), tag("include")).parse(ParserInput::new("ix")).unwrap_err();

    assert_eq!(error.byte_position(), 1);
    assert_eq!(error.location().found_char, Some('x'));

    let error = (&*error as &dyn std::error::Error).downcast_ref::<AlternativeError>().unwrap();

    assert_eq!(error.first.byte_position(), 1);
    assert_eq!(error.second.byte_position(), 1);
    assert!(error.first.to_string().contains("if"));
    assert!(error.second.to_string().contains("include"));
    assert!(error.to_string().contains(" or "));
}
