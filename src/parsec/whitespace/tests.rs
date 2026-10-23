use crate::parsec::*;

use super::*;

#[test]
fn horizontal_whitespace_requires_a_space_or_tab_and_preserves_source_slice() {
    let source = "! \t  \r\ncontent";
    let mut input = ParserInput::new(source);
    input.set_byte_position(1);

    let (spaces, remaining) = hspace().parse(input).unwrap();

    assert_eq!(spaces, " \t  ");
    assert_eq!(spaces.as_ptr(), source[1..].as_ptr());
    assert_eq!(remaining.peek(), Some('\r'));
}

#[test]
fn horizontal_whitespace_does_not_consume_line_breaks_or_other_unicode_whitespace() {
    for source in ["", "text", "\n", "\r\n", "\u{00a0}", "\u{2003}"] {
        let error = hspace().parse(ParserInput::new(source)).unwrap_err();

        assert!(error.is_recoverable());
        assert_eq!(error.location().failure_byte, 0);
    }
}

#[test]
fn required_trim_skips_unicode_whitespace_and_preserves_inner_output() {
    let source = "! \t\r\n\u{2003}value ";
    let mut input = ParserInput::new(source);
    input.set_byte_position(1);

    let (output, remaining) = ltrim1(tag("value")).parse(input).unwrap();

    assert_eq!(output, "value");
    assert_eq!(output.as_ptr(), source[source.find("value").unwrap()..].as_ptr());
    assert_eq!(remaining.peek(), Some(' '));
}

#[test]
fn required_trim_does_not_call_inner_parser_without_a_separator() {
    for source in ["", "value"] {
        let called = std::cell::Cell::new(false);
        let inner = parser(|input| {
            called.set(true);
            Ok(((), input))
        });

        let error = ltrim1(inner).parse(ParserInput::new(source)).unwrap_err();

        assert!(error.is_recoverable());
        assert_eq!(error.byte_position(), 0);
        assert!(!called.get());
    }
}

#[test]
fn required_trim_allows_empty_inner_success_and_preserves_inner_errors() {
    let (output, remaining) = ltrim1(succeed()).parse(ParserInput::new(" \n")).unwrap();

    assert_eq!(output, "");
    assert!(remaining.is_at_end());

    let error = ltrim1(tag("value")).parse(ParserInput::new(" \tx")).unwrap_err();

    assert_eq!(error.byte_position(), 2);
    assert_eq!(error.location().found_char, Some('x'));
    assert!((&*error as &dyn std::error::Error).is::<TagError>());
}

#[test]
fn skips_leading_whitespace_and_preserves_trailing_whitespace() {
    let source = " \t\r\n\u{a0}\u{2003}я \n";
    let start_byte = source.find('я').unwrap();

    let (output, remaining) = ltrim(tag("я"))
        .parse(ParserInput::new(source))
        .unwrap();

    assert_eq!(output, "я");
    assert_eq!(output.as_ptr(), source[start_byte..].as_ptr());
    assert_eq!(remaining.byte_position(), start_byte + "я".len());
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
