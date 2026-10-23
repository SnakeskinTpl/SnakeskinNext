use SnakeskinNext::parsec::prelude::*;

#[test]
fn prelude_provides_combinator_functions_and_macros() {
    let grammar = seq!(
        peek(or!(tag("a"), tag("b"))),
        choice!(tag("a"), tag("b")),
        ltrim1(map(tag("value"), str::len))
    );

    let ((((), marker), length), remaining) = grammar.parse(ParserInput::new("a value!")).unwrap();

    assert_eq!(marker, "a");
    assert_eq!(length, 5);
    assert_eq!(remaining.peek(), Some('!'));
}
