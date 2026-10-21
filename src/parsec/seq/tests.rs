use std::cell::Cell;

use crate::parsec::*;

use super::*;

#[test]
fn sequence_preserves_output_order_and_remaining_input() {
    let parser = seq(tag("я"), tag("🦀"));

    let ((first, second), remaining) = parser.parse(ParserInput::new("я🦀!")).unwrap();

    assert_eq!((first, second), ("я", "🦀"));
    assert_eq!(remaining.byte_position(), 6);
    assert_eq!(remaining.peek(), Some('!'));
}

struct CountCalls<'a>(&'a Cell<usize>);

impl<'a> Parser<'a> for CountCalls<'_> {
    type Output = ();

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        self.0.set(self.0.get() + 1);
        Ok(((), input))
    }
}

#[test]
fn first_error_prevents_second_parser_from_running() {
    let calls = Cell::new(0);
    let parser = seq(tag("a"), CountCalls(&calls));

    let error = parser.parse(ParserInput::new("x")).unwrap_err();

    assert_eq!(error.byte_position(), 0);
    assert_eq!(calls.get(), 0);
}

#[test]
fn second_error_keeps_its_source_position_and_expectation() {
    let parser = seq(tag("я"), tag("ab"));

    let error = parser.parse(ParserInput::new("яax")).unwrap_err();

    assert_eq!(error.byte_position(), 3);

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

    assert_eq!(error.location().start_byte, 2);
    assert_eq!(error.expected_tag, "ab");
    assert_eq!(error.location().found_char, Some('x'));
}

#[derive(Debug, Clone, PartialEq)]
struct Context;

impl crate::parsec::input::ParserState for Context {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn crate::parsec::input::ParserState> {
        Box::new(self.clone())
    }
}

fn push_context<'a>(output: &'a str, mut input: ParserInput<'a>) -> ParserResult<'a, &'a str> {
    input.push_state(Context);
    Ok((output, input))
}

fn check_context<'a>(output: &'a str, input: ParserInput<'a>) -> ParserResult<'a, &'a str> {
    assert!(input.is_current_state(&Context));
    Ok((output, input))
}

#[test]
fn state_changes_from_first_parser_reach_second_parser() {
    let parser = seq(flat_map(tag("a"), push_context), flat_map(tag("b"), check_context));

    let (_, remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert!(remaining.is_current_state(&Context));
    assert!(remaining.is_at_end());
}
