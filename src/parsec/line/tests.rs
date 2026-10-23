use crate::parsec::{opt, seq, tag, ParserInput};

use super::*;

#[test]
fn line_end_matches_before_line_breaks_and_at_eof_without_consuming() {
    for (source, position) in [("", 0), ("text", 4), ("\u{0436}\nnext", 2), ("\u{0436}\r\nnext", 2)] {
        let mut input = ParserInput::new(source);
        input.set_byte_position(position);

        let ((), remaining) = line_end().parse(input).unwrap();

        assert_eq!(remaining.byte_position(), position);
        assert_eq!(remaining.peek(), source[position..].chars().next());
    }
}

#[test]
fn line_end_rejects_ordinary_content_and_standalone_cr() {
    for source in ["text", " ", "\r", "\rtext"] {
        let error = line_end().parse(ParserInput::with_offset(source, 17)).unwrap_err();

        assert!(error.is_recoverable());
        assert_eq!(error.location().failure_byte, 17 + usize::from(source.starts_with('\r')));
    }
}

#[test]
fn line_end_leaves_crlf_for_the_line_break_parser() {
    let parser = seq(line_end(), line_break());

    let (((), ending), remaining) = parser.parse(ParserInput::with_offset("\r\nnext", 10)).unwrap();

    assert_eq!(ending, "\r\n");
    assert_eq!(remaining.byte_position(), 12);
    assert_eq!(remaining.peek(), Some('n'));
}

#[test]
fn line_end_matches_the_end_of_a_bounded_slice() {
    let source = "text trailing";
    let mut input = ParserInput::with_offset(&source[..4], 20);
    input.set_byte_position(24);

    let ((), remaining) = line_end().parse(input).unwrap();

    assert_eq!(remaining.byte_position(), 24);
    assert!(remaining.is_at_end());
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

#[test]
fn consumes_a_complete_line_ending_and_borrows_its_source() {
    for ending in ["\n", "\r\n"] {
        let source = format!("!{ending}rest");
        let mut input = ParserInput::new(&source);
        input.set_byte_position(1);

        let (output, remaining) = line_break().parse(input).unwrap();

        assert_eq!(output, ending);
        assert_eq!(output.as_ptr(), source[1..].as_ptr());
        assert_eq!(remaining.byte_position(), 1 + ending.len());
        assert_eq!(remaining.peek(), Some('r'));
    }
}

#[test]
fn rejects_incomplete_or_missing_line_ending_without_consuming_on_optional_match() {
    for source in ["", "\r", "\rx", "text"] {
        let (output, remaining) = opt(line_break()).parse(ParserInput::new(source)).unwrap();

        assert_eq!(output, None);
        assert_eq!(remaining.byte_position(), 0);
    }
}

#[test]
fn consecutive_line_endings_remain_separate() {
    let parser = seq(line_break(), seq(line_break(), tag("text")));

    let ((first, (second, text)), remaining) = parser.parse(ParserInput::new("\r\n\ntext")).unwrap();

    assert_eq!((first, second, text), ("\r\n", "\n", "text"));
    assert!(remaining.is_at_end());
}

#[test]
fn returns_borrowed_indentation_without_consuming_content() {
    for indent in ["", "  ", "\t", " \t "] {
        let source = format!("{indent}я");

        let (output, remaining) = line_indent().parse(ParserInput::new(&source)).unwrap();

        assert_eq!(output, indent);
        assert_eq!(output.as_ptr(), source.as_ptr());
        assert_eq!(remaining.byte_position(), indent.len());
        assert_eq!(remaining.peek(), Some('я'));
    }
}

#[test]
fn recognizes_indentation_after_lf_and_crlf() {
    for ending in ["\n", "\r\n"] {
        let source = format!("text{ending}\tvalue");
        let start_byte = "text".len() + ending.len();
        let mut input = ParserInput::new(&source);
        input.set_byte_position(start_byte);

        let (indent, remaining) = line_indent().parse(input).unwrap();

        assert_eq!(indent, "\t");
        assert_eq!(remaining.byte_position(), start_byte + 1);
        assert_eq!(remaining.peek(), Some('v'));
    }
}

#[test]
fn blank_lines_and_eof_are_recoverable_mismatches() {
    for source in ["", " \t", "\n", "\r\n", " \t\n", " \t\r\n"] {
        let error = line_indent().parse(ParserInput::new(source)).unwrap_err();

        assert!(error.is_recoverable());

        let (output, remaining) = opt(line_indent()).parse(ParserInput::new(source)).unwrap();

        assert_eq!(output, None);
        assert_eq!(remaining.byte_position(), 0);
    }
}

#[test]
fn rejects_indentation_in_the_middle_of_a_line() {
    let mut input = ParserInput::new("x  value");
    input.set_byte_position(1);

    let error = line_indent().parse(input).unwrap_err();

    assert!(error.is_recoverable());
    assert_eq!(error.byte_position(), 1);
}
