use super::*;

#[test]
fn successful_match_returns_some_and_remaining_input() {
    let source = String::from("я🦀!");

    let (output, remaining) = opt(tag("я🦀")).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(output, Some("я🦀"));
    assert_eq!(output.unwrap().as_ptr(), source.as_ptr());
    assert_eq!(remaining.byte_position(), 6);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn failed_partial_match_restores_original_position() {
    let mut input = ParserInput::new("!яx");
    input.set_byte_position(1);

    let (output, remaining) = opt(tag("яb")).parse(input).unwrap();

    assert_eq!(output, None);
    assert_eq!(remaining.byte_position(), 1);
    assert_eq!(remaining.peek(), Some('я'));
}

#[test]
fn incomplete_match_returns_none() {
    for source in ["", "a"] {
        let (output, remaining) = opt(tag("ab")).parse(ParserInput::new(source)).unwrap();

        assert_eq!(output, None);
        assert_eq!(remaining.byte_position(), 0);
    }
}

#[test]
fn empty_success_returns_some_without_advancing() {
    let (output, remaining) = opt(succeed()).parse(ParserInput::new("a")).unwrap();

    assert_eq!(output, Some(""));
    assert_eq!(remaining.byte_position(), 0);
}

#[derive(Clone, Debug, PartialEq)]
struct Context;

impl crate::parsec::input::ParserState for Context {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_box(&self) -> Box<dyn crate::parsec::input::ParserState> { Box::new(self.clone()) }
}

fn pop_context<'a>(output: &'a str, mut input: ParserInput<'a>) -> ParserResult<'a, &'a str> {
    input.pop_state();
    Ok((output, input))
}

#[test]
fn failed_match_restores_state_changes() {
    let mut input = ParserInput::new("ab");
    input.push_state(Context);
    let parser = opt(seq(flat_map(tag("a"), pop_context), tag("x")));

    let (output, remaining) = parser.parse(input).unwrap();

    assert_eq!(output, None);
    assert_eq!(remaining.byte_position(), 0);
    assert!(remaining.is_current_state(&Context));
}

#[test]
fn successful_match_preserves_state_changes() {
    let mut input = ParserInput::new("ab");
    input.push_state(Context);
    let parser = opt(seq(flat_map(tag("a"), pop_context), tag("b")));

    let (output, remaining) = parser.parse(input).unwrap();

    assert_eq!(output, Some(("a", "b")));
    assert!(remaining.peek_state().is_none());
    assert!(remaining.is_at_end());
}
