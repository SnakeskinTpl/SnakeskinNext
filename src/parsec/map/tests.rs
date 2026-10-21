use std::cell::Cell;

use crate::parsec::*;

use super::*;

#[test]
fn transforms_output_and_preserves_remaining_input() {
    let parser = map(tag("я🦀"), |text| text.chars().count());

    let (output, remaining) = parser.parse(ParserInput::new("я🦀!")).unwrap();

    assert_eq!(output, 2);
    assert_eq!(remaining.byte_position(), 6);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn failure_skips_transform_and_preserves_inner_error() {
    let calls = Cell::new(0);
    let parser = map(tag("ab"), |_| calls.set(calls.get() + 1));

    let error = parser.parse(ParserInput::new("ax")).unwrap_err();

    assert_eq!(calls.get(), 0);
    assert_eq!(error.byte_position(), 1);
    assert!((&*error as &dyn std::error::Error).is::<TagError>());
}

#[test]
fn transform_can_return_borrowed_output_without_debug() {
    struct Value<'a>(&'a str);
    let source = String::from("abc!");
    let parser = map(tag("abc"), Value);

    let (output, remaining) = parser.parse(ParserInput::new(&source)).unwrap();

    assert_eq!(output.0, "abc");
    assert_eq!(output.0.as_ptr(), source.as_ptr());
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn identity_and_composition_preserve_results() {
    let source = "abc!";
    let identity = map(tag("abc"), |text| text);

    let (output, remaining) = identity.parse(ParserInput::new(source)).unwrap();

    assert_eq!(output, "abc");
    assert_eq!(remaining.byte_position(), 3);

    let nested = map(map(tag("abc"), |text| text.len()), |length| length + 1);
    let composed = map(tag("abc"), |text| text.len() + 1);

    let (nested_output, nested_remaining) = nested.parse(ParserInput::new(source)).unwrap();
    let (composed_output, composed_remaining) = composed.parse(ParserInput::new(source)).unwrap();

    assert_eq!(nested_output, composed_output);
    assert_eq!(nested_remaining.byte_position(), composed_remaining.byte_position());
}
