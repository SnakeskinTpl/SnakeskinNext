use SnakeskinNext::parsec::input::ParserInput;
use SnakeskinNext::parsec::{Either, Parser, tag};

#[test]
fn qualified_macro_call_supports_multiple_branches() {
    let parser = SnakeskinNext::or!(tag("a"), tag("b"), tag("c"),);

    let (output, remaining) = parser.parse(ParserInput::new("b!")).unwrap();

    assert_eq!(output, Either::Left(Either::Right("b")));
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn single_branch_accepts_trailing_comma() {
    let parser = SnakeskinNext::or!(tag("a"),);

    let (output, remaining) = parser.parse(ParserInput::new("a")).unwrap();

    assert_eq!(output, "a");
    assert!(remaining.is_at_end());
}
