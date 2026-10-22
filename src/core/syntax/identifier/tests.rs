use crate::parsec::ParserInput;

use super::*;

#[test]
fn identifiers_borrow_source_and_leave_the_separator_unconsumed() {
    for name in ["tag_2", "_", "_name", "тег2"] {
        let source = format!("!{name} value");
        let mut input = ParserInput::new(&source);
        input.set_byte_position(1);

        let (output, remaining) = identifier().parse(input).unwrap();

        assert_eq!(output, name);
        assert_eq!(output.as_ptr(), source[1..].as_ptr());
        assert_eq!(remaining.byte_position(), 1 + name.len());
        assert_eq!(remaining.peek(), Some(' '));
    }
}

#[test]
fn accepts_eof_and_stops_before_punctuation() {
    let (name, remaining) = identifier().parse(ParserInput::new("tag")).unwrap();

    assert_eq!(name, "tag");
    assert!(remaining.is_at_end());

    let (name, remaining) = identifier().parse(ParserInput::new("tag-name")).unwrap();

    assert_eq!(name, "tag");
    assert_eq!(remaining.peek(), Some('-'));
}

#[test]
fn rejects_invalid_first_characters_and_empty_input() {
    for source in ["", "2tag", " tag", "-tag"] {
        let error = identifier().parse(ParserInput::new(source)).unwrap_err();

        assert_eq!(error.byte_position(), 0);
        assert!(error.is_recoverable());
    }
}
