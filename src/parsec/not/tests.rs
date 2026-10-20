use super::*;

#[test]
fn failed_partial_match_does_not_consume_input() {
    let mut input = ParserInput::new("!яx");
    input.set_byte_position(1);

    let ((), remaining) = not(tag("яb")).parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 1);
    assert_eq!(remaining.peek(), Some('я'));
}

#[test]
fn forbidden_match_reports_start_of_check() {
    let mut input = ParserInput::new("!яx");
    input.set_byte_position(1);

    let error = not(tag("я")).parse(input).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start: 1, byte_position: 1, actual: Some('я') });
    assert!((&*error as &dyn std::error::Error).is::<NotError>());
}

#[test]
fn incomplete_match_succeeds_without_consuming_input() {
    for source in ["", "a"] {
        let ((), remaining) = not(tag("ab")).parse(ParserInput::new(source)).unwrap();

        assert_eq!(remaining.byte_position(), 0);
        assert_eq!(remaining.peek(), source.chars().next());
    }
}

#[test]
fn empty_success_is_a_forbidden_match_even_at_end() {
    let error = not(succeed()).parse(ParserInput::new("")).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start: 0, byte_position: 0, actual: None });
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
fn failed_inner_parse_does_not_change_states() {
    let mut input = ParserInput::new("ab");
    input.push_state(Context);
    let parser = not(seq(flat_map(tag("a"), pop_context), tag("x")));

    let ((), remaining) = parser.parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 0);
    assert!(remaining.is_current_state(&Context));
}

#[test]
fn sequence_after_negative_lookahead_reads_original_input() {
    let parser = seq(not(tag("ax")), tag("ab"));

    let (((), output), remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert_eq!(output, "ab");
    assert!(remaining.is_at_end());
}
