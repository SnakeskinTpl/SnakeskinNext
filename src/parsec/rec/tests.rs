use std::cell::Cell;

use crate::parsec::*;

use super::*;

// expression = "x" | "(" expression ")"
fn expression<'a>() -> Box<dyn Parser<'a, Output = &'a str>> {
    Box::new(choice(
        tag("x"),
        flat_map(seq(tag("("), seq(rec(expression), tag(")"))), unwrap_parentheses),
    ))
}

fn unwrap_parentheses<'a>(
    (_, (inner, _)): (&'a str, (&'a str, &'a str)),
    input: ParserInput<'a>,
) -> ParserResult<'a, &'a str> {
    Ok((inner, input))
}

#[test]
fn recursive_grammar_parses_nested_parentheses() {
    for source in ["x!", "(x)!", "(((x)))!"] {
        let (output, remaining) = expression().parse(ParserInput::new(source)).unwrap();

        assert_eq!(output, "x");
        assert_eq!(output.as_ptr(), source[source.find('x').unwrap()..].as_ptr());
        assert_eq!(remaining.byte_position(), source.len() - 1);
        assert_eq!(remaining.peek(), Some('!'));
    }
}

#[test]
fn recursive_grammar_reports_missing_closing_parenthesis() {
    let error = expression().parse(ParserInput::new("((x)")).unwrap_err();

    assert_eq!(error.byte_position(), 4);
    assert_eq!(error.location().found_char, None);

    let error = (&*error as &dyn std::error::Error).downcast_ref::<TagError>().unwrap();

    assert_eq!(error.expected_tag, ")");
}

#[test]
fn recursive_grammar_reports_invalid_inner_expression() {
    let error = expression().parse(ParserInput::new("(z)")).unwrap_err();

    assert_eq!(error.byte_position(), 1);
    assert_eq!(error.location().found_char, Some('z'));
}

#[test]
fn factory_is_deferred_and_called_for_each_parse_attempt() {
    let calls = Cell::new(0);
    let parser = rec(|| {
        calls.set(calls.get() + 1);
        tag("x")
    });

    assert_eq!(calls.get(), 0);

    let (output, _) = parser.parse(ParserInput::new("x")).unwrap();

    assert_eq!(output, "x");
    assert_eq!(calls.get(), 1);

    let error = parser.parse(ParserInput::new("z")).unwrap_err();

    assert_eq!(error.byte_position(), 0);
    assert_eq!(calls.get(), 2);
}

#[test]
fn deferred_parser_preserves_nonzero_input_position() {
    let mut input = ParserInput::new("я(x)!");
    input.set_byte_position(2);

    let (output, remaining) = rec(expression).parse(input).unwrap();

    assert_eq!(output, "x");
    assert_eq!(remaining.byte_position(), 5);
    assert_eq!(remaining.peek(), Some('!'));
}
