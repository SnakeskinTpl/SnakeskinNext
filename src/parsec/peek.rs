use super::{Parser, ParserInput, ParserResult};

#[cfg(test)]
mod tests;

/// Requires a successful match, discarding its output and input changes.
/// Propagates the inner parser's error unchanged on failure.
pub fn peek<'a, P: Parser<'a>>(parser: P) -> Peek<P> {
    Peek(parser)
}

#[derive(Debug)]
pub struct Peek<P>(P);

impl<'a, P: Parser<'a>> Parser<'a> for Peek<P> {
    type Output = ();

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        self.0.parse(input.clone())?;
        Ok(((), input))
    }
}
