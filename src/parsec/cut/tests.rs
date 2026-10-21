use crate::parsec::*;

use super::*;

#[test]
fn success_preserves_borrowed_output_and_remaining_input() {
    let source = String::from("я!");

    let (output, remaining) = cut(tag("я")).parse(ParserInput::new(&source)).unwrap();

    assert_eq!(output, "я");
    assert_eq!(output.as_ptr(), source.as_ptr());
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn failure_preserves_original_diagnostic_and_location() {
    let original = tag("ab").parse(ParserInput::new("ax")).unwrap_err();

    let error = cut(tag("ab")).parse(ParserInput::new("ax")).unwrap_err();

    assert!(!error.is_recoverable());
    assert_eq!(error.location(), original.location());
    assert_eq!(error.to_string(), original.to_string());
    assert!(error.source().unwrap().is::<TagError>());
}

#[test]
fn already_committed_errors_are_not_wrapped_again() {
    let error = cut(cut(tag("a"))).parse(ParserInput::new("x")).unwrap_err();

    assert!(error.source().unwrap().is::<TagError>());

    let error = cut(repeat(succeed(), ..)).parse(ParserInput::new("x")).unwrap_err();

    assert!((&*error as &dyn std::error::Error).is::<RepeatError>());
}

#[test]
fn opening_check_allows_fallback_but_committed_body_does_not() {
    let parser = choice(
        map(seq(lookup(tag("{")), cut(tag("{a}"))), |(_, value)| value),
        tag("text"),
    );

    let (output, _) = parser.parse(ParserInput::new("text")).unwrap();

    assert_eq!(output, "text");

    let error = parser.parse(ParserInput::new("{x}")).unwrap_err();

    assert!(!error.is_recoverable());
    assert_eq!(error.byte_position(), 1);
}

#[test]
fn optional_repeat_and_negative_lookahead_propagate_committed_failure() {
    let error = opt(cut(tag("a"))).parse(ParserInput::new("x")).unwrap_err();

    assert!(!error.is_recoverable());

    let error = repeat(cut(tag("a")), 0..).parse(ParserInput::new("ax")).unwrap_err();

    assert!(!error.is_recoverable());
    assert_eq!(error.byte_position(), 1);

    let error = not(cut(tag("a"))).parse(ParserInput::new("x")).unwrap_err();

    assert!(!error.is_recoverable());
}
