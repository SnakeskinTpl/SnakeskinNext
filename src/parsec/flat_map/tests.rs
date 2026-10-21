use std::cell::Cell;

use crate::parsec::*;

use super::*;

#[test]
fn callback_selects_next_parser_from_previous_output() {
    let parser = flat_map(choice(tag("a"), tag("b")), |output, input| {
        tag(output).parse(input)
    });

    let (output, remaining) = parser.parse(ParserInput::new("bb!")).unwrap();

    assert_eq!(output, "b");
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn callback_error_is_propagated() {
    let parser = flat_map(tag("a"), |_, input| tag("b").parse(input));

    let error = parser.parse(ParserInput::new("ax")).unwrap_err();

    assert_eq!(*error.location(), ErrorLocation { start_byte: 1, failure_byte: 1, found_char: Some('x') });

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

    assert_eq!(error.expected_tag, "b");
}

#[test]
fn inner_error_skips_callback() {
    let calls = Cell::new(0);
    let parser = flat_map(tag("ab"), |output, input| {
        calls.set(calls.get() + 1);
        Ok((output, input))
    });

    let error = parser.parse(ParserInput::new("ax")).unwrap_err();

    assert_eq!(calls.get(), 0);
    assert_eq!(error.byte_position(), 1);
}
