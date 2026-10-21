use crate::parsec::*;

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
