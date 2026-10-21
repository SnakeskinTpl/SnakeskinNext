use crate::parsec::*;

use super::*;

#[test]
fn alternative_location_starts_at_choice_entry() {
    let first = seq(tag("a"), tag("bc"));
    let second = seq(tag("ab"), tag("d"));

    let error = choice(first, second).parse(ParserInput::new("abX")).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start_byte: 0, failure_byte: 2, found_char: Some('X') });

    let error = (&*error as &dyn std::error::Error).downcast_ref::<AlternativeError>().unwrap();

    assert_eq!(error.first.location().start_byte, 1);
    assert_eq!(error.second.location().start_byte, 2);
}

#[test]
fn first_success_wins_without_output_wrapper() {
    let (output, remaining) = choice(tag("a"), tag("ab"))
        .parse(ParserInput::new("ab"))
        .unwrap();

    assert_eq!(output, "a");
    assert_eq!(remaining.peek(), Some('b'));
}

#[test]
fn second_branch_retries_from_original_position() {
    let (output, remaining) = choice(tag("яa"), tag("яb"))
        .parse(ParserInput::new("яb!"))
        .unwrap();

    assert_eq!(output, "яb");
    assert_eq!(remaining.byte_position(), 3);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn furthest_error_wins_in_either_branch_order() {
    for parser in [choice(tag("template"), tag("var")), choice(tag("var"), tag("template"))] {
        let error = parser.parse(ParserInput::new("templX")).unwrap_err();

        assert_eq!(error.byte_position(), 5);
        assert!(error.to_string().contains("template"));
    }
}

#[test]
fn equal_positions_preserve_both_errors() {
    let error = choice(tag("if"), tag("include"))
        .parse(ParserInput::new("ix"))
        .unwrap_err();

    let error = (&*error as &dyn std::error::Error).downcast_ref::<AlternativeError>().unwrap();

    assert_eq!(error.first.byte_position(), 1);
    assert_eq!(error.second.byte_position(), 1);
    assert!(error.first.to_string().contains("if"));
    assert!(error.second.to_string().contains("include"));
}
