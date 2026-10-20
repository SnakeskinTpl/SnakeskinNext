use super::*;

#[test]
fn successful_match_preserves_original_position() {
    let mut input = ParserInput::new("!я🦀rest");
    input.set_byte_position(1);

    let ((), remaining) = lookup(tag("я🦀")).parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 1);
    assert_eq!(remaining.peek(), Some('я'));
}

#[test]
fn failure_preserves_inner_error_context() {
    let mut input = ParserInput::new("!яx");
    input.set_byte_position(1);

    let error = lookup(tag("яb")).parse(input).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start: 1, byte_position: 3, actual: Some('x') });

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

    assert_eq!(error.expected_tag, "яb");
    assert_eq!(error.expected, 'b');
}

#[test]
fn incomplete_match_reports_end_of_input() {
    let error = lookup(tag("ab")).parse(ParserInput::new("a")).unwrap_err();

    assert_eq!(error.byte_position(), 1);
    assert_eq!(error.location().actual, None);
}

#[test]
fn empty_success_is_allowed_at_end_of_input() {
    let ((), remaining) = lookup(succeed()).parse(ParserInput::new("")).unwrap();

    assert_eq!(remaining.byte_position(), 0);
    assert!(remaining.is_at_end());
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
fn successful_inner_parse_does_not_change_states() {
    let mut input = ParserInput::new("a");
    input.push_state(Context);
    let parser = lookup(flat_map(tag("a"), pop_context));

    let ((), remaining) = parser.parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 0);
    assert!(remaining.is_current_state(&Context));
}

#[test]
fn sequence_after_lookahead_reads_original_input() {
    let parser = seq(lookup(tag("ab")), tag("ab"));

    let (((), output), remaining) = parser.parse(ParserInput::new("ab")).unwrap();

    assert_eq!(output, "ab");
    assert!(remaining.is_at_end());
}
