#[cfg(test)]
mod tests;

use super::*;

/// Skips leading Unicode whitespace, including line breaks, then runs the parser.
/// Leaves trailing whitespace untouched and calls the parser even at end of input.
pub fn ltrim<'a, P: Parser<'a>>(parser: P) -> LTrim<P> {
    LTrim(parser)
}

#[derive(Debug)]
pub struct LTrim<P>(P);

impl<'a, P: Parser<'a>> Parser<'a> for LTrim<P> {
    type Output = P::Output;

    fn parse(&self, mut input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        while input.peek().is_some_and(char::is_whitespace) {
            input.next();
        }

        self.0.parse(input)
    }
}
