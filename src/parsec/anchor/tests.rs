use crate::parsec::{map, opt, seq, tag};

use super::*;

#[test]
fn succeeds_at_start_including_empty_input_without_advancing() {
    for source in ["", "я"] {
        let ((), remaining) = input_start().parse(ParserInput::new(source)).unwrap();

        assert_eq!(remaining.byte_position(), 0);
        assert_eq!(remaining.peek(), source.chars().next());
    }
}

#[test]
fn rejects_nonzero_position_including_end_of_input() {
    for source in ["яx", "я"] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(2);

        let error = input_start().parse(input).unwrap_err();

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

    let (output, remaining) = opt(input_start()).parse(input).unwrap();

    assert_eq!(output, None);
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('x'));
}

#[test]
fn anchor_composes_with_consuming_parser() {
    let parser = map(seq(input_start(), tag("я")), |((), value)| value);

    let (output, remaining) = parser.parse(ParserInput::new("я!")).unwrap();

    assert_eq!(output, "я");
    assert_eq!(remaining.byte_position(), 2);
    assert_eq!(remaining.peek(), Some('!'));
}
