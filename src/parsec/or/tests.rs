use super::*;

#[test]
fn alternative_location_covers_whole_choice_and_preserves_branch_starts() {
    let first = seq(tag("a"), tag("bc"));
    let second = seq(tag("ab"), tag("d"));
    let mut input = ParserInput::new("!abX");
    input.set_byte_position(1);

    let error = or(first, second).parse(input).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start: 1, byte_position: 3, actual: Some('X') });

    let error = (&*error as &dyn std::error::Error).downcast_ref::<AlternativeError>().unwrap();

    assert_eq!(error.first.location().start, 2);
    assert_eq!(error.second.location().start, 3);
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
    assert_eq!(error.location().actual, Some('x'));

    let error = (&*error as &dyn std::error::Error).downcast_ref::<AlternativeError>().unwrap();

    assert_eq!(error.first.byte_position(), 1);
    assert_eq!(error.second.byte_position(), 1);
    assert!(error.first.to_string().contains("if"));
    assert!(error.second.to_string().contains("include"));
    assert!(error.to_string().contains(" or "));
}

#[derive(Clone, Debug, PartialEq)]
struct Context;

impl crate::parsec::input::ParserState for Context {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_box(&self) -> Box<dyn crate::parsec::input::ParserState> { Box::new(self.clone()) }
}

fn push_context<'a>(output: &'a str, mut input: ParserInput<'a>) -> ParserResult<'a, &'a str> {
    input.push_state(Context);
    Ok((output, input))
}

#[test]
fn failed_branch_state_changes_are_rolled_back() {
    let first = seq(flat_map(tag("a"), push_context), tag("x"));
    let parser = or(first, tag("ab"));

    let (output, remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert_eq!(output, Either::Right("ab"));
    assert!(remaining.peek_state().is_none());
}

#[test]
fn successful_branch_state_changes_are_preserved() {
    let parser = or(flat_map(tag("a"), push_context), tag("b"));

    let (output, remaining) = parser.parse(ParserInput::new("a")).unwrap();

    assert_eq!(output, Either::Left("a"));
    assert!(remaining.is_current_state(&Context));
}
