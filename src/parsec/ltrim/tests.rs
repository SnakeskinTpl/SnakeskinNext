use crate::parsec::*;

use super::*;

#[test]
fn skips_leading_whitespace_and_preserves_trailing_whitespace() {
    let source = " \t\r\n\u{a0}\u{2003}я \n";
    let start = source.find('я').unwrap();

    let (output, remaining) = ltrim(tag("я"))
        .parse(ParserInput::new(source))
        .unwrap();

    assert_eq!(output, "я");
    assert_eq!(output.as_ptr(), source[start..].as_ptr());
    assert_eq!(remaining.byte_position(), start + "я".len());
    assert_eq!(remaining.peek(), Some(' '));
}

#[test]
fn no_leading_whitespace_is_required() {
    let (output, remaining) = ltrim(tag("a"))
        .parse(ParserInput::new("a!"))
        .unwrap();

    assert_eq!(output, "a");
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn starts_skipping_at_current_position() {
    let mut input = ParserInput::new("! \tя!");
    input.set_byte_position(1);

    let (output, remaining) = ltrim(tag("я")).parse(input).unwrap();

    assert_eq!(output, "я");
    assert_eq!(remaining.byte_position(), 5);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn inner_parser_can_succeed_at_end_of_input() {
    for source in ["", " \t\n"] {
        let (output, remaining) = ltrim(succeed())
            .parse(ParserInput::new(source))
            .unwrap();

        assert_eq!(output, "");
        assert_eq!(remaining.byte_position(), source.len());
        assert!(remaining.is_at_end());

        let (output, remaining) = ltrim(opt(tag("a")))
            .parse(ParserInput::new(source))
            .unwrap();

        assert_eq!(output, None);
        assert!(remaining.is_at_end());
    }
}

#[test]
fn failure_preserves_inner_error_after_skipped_whitespace() {
    for source in [" \t", " \tx"] {
        let error = ltrim(tag("a"))
            .parse(ParserInput::new(source))
            .unwrap_err();

        assert_eq!(*error.location(), ErrorLocation {
            start_byte: 2,
            failure_byte: 2,
            found_char: source[2..].chars().next(),
        });

        let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

        assert_eq!(error.expected_tag, "a");
    }
}
