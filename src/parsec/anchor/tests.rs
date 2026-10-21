use crate::parsec::{map, opt, seq, tag};

use super::*;

#[test]
fn succeeds_at_start_including_empty_input_without_advancing() {
    for source in ["", "я"] {
        let ((), remaining) = start_of_input().parse(ParserInput::new(source)).unwrap();

        assert_eq!(remaining.byte_position(), 0);
        assert_eq!(remaining.peek(), source.chars().next());
    }
}

#[test]
fn rejects_nonzero_position_including_end_of_input() {
    for source in ["яx", "я"] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(2);

        let error = start_of_input().parse(input).unwrap_err();

        assert_eq!(*error.location(), ErrorLocation {
            start_byte: 2,
            failure_byte: 2,
            found_char: source[2..].chars().next(),
        });
        assert!(error.is_recoverable());
    }
}

#[test]
fn optional_anchor_leaves_nonzero_position_unchanged() {
    let mut input = ParserInput::new("яx");
    input.set_byte_position(2);

    let (output, remaining) = opt(start_of_input()).parse(input).unwrap();

    assert_eq!(output, None);
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('x'));
}

#[test]
fn anchor_composes_with_consuming_parser() {
    let parser = map(seq(start_of_input(), tag("я")), |((), value)| value);

    let (output, remaining) = parser.parse(ParserInput::new("я!")).unwrap();

    assert_eq!(output, "я");
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn matches_empty_input_and_first_line_without_consuming() {
    for source in ["", "я"] {
        let ((), remaining) = line_start().parse(ParserInput::new(source)).unwrap();

        assert_eq!(remaining.byte_position(), 0);
        assert_eq!(remaining.peek(), source.chars().next());
    }
}

#[test]
fn matches_after_lf_and_crlf_including_eof() {
    for (source, position) in [("я\nx", 3), ("я\r\nx", 4), ("я\n", 3)] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(position);

        let ((), remaining) = line_start().parse(input).unwrap();

        assert_eq!(remaining.byte_position(), position);
        assert_eq!(remaining.peek(), source[position..].chars().next());
    }
}

#[test]
fn rejects_middle_of_line_and_position_inside_crlf() {
    for (source, position) in [("яx", 2), ("я", 2), ("я\r\nx", 3)] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(position);

        let error = line_start().parse(input.clone()).unwrap_err();

        assert_eq!(error.byte_position(), position);
        assert!(error.is_recoverable());

        let (output, remaining) = opt(line_start()).parse(input).unwrap();

        assert_eq!(output, None);
        assert_eq!(remaining.byte_position(), position);
    }
}

#[test]
fn anchor_does_not_consume_newline_for_following_parser() {
    let ((), remaining) = seq(tag("я\n"), line_start())
        .parse(ParserInput::new("я\nx")).map(|((_, output), input)| (output, input)).unwrap();

    assert_eq!(remaining.byte_position(), 3);
    assert_eq!(remaining.peek(), Some('x'));
}
