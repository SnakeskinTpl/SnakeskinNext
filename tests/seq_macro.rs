use SnakeskinNext::parsec::{Parser, tag};
use SnakeskinNext::parsec::input::ParserInput;

#[test]
fn qualified_macro_call_nests_outputs_and_accepts_trailing_comma() {
    let parser = SnakeskinNext::seq!(tag("a"), tag("b"), tag("c"), tag("d"),);

    let (output, remaining) = parser.parse(ParserInput::new("abcd!")).unwrap();

    assert_eq!(output, ((("a", "b"), "c"), "d"));
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn single_parser_accepts_trailing_comma() {
    let parser = SnakeskinNext::seq!(tag("a"),);

    let (output, remaining) = parser.parse(ParserInput::new("a")).unwrap();

    assert_eq!(output, "a");
    assert!(remaining.is_at_end());
}
