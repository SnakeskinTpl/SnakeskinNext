use SnakeskinNext::parsec::input::ParserInput;
use SnakeskinNext::parsec::{Parser, tag};

#[test]
fn qualified_macro_returns_common_type_for_multiple_branches() {
    let parser = SnakeskinNext::choice!(tag("a"), tag("b"), tag("c"),);

    let (output, remaining) = parser.parse(ParserInput::new("c!")).unwrap();

    assert_eq!(output, "c");
    assert_eq!(remaining.peek(), Some('!'));
}

#[test]
fn single_branch_accepts_trailing_comma() {
    let parser = SnakeskinNext::choice!(tag("a"),);

    let (output, remaining) = parser.parse(ParserInput::new("a")).unwrap();

    assert_eq!(output, "a");
    assert!(remaining.is_at_end());
}
