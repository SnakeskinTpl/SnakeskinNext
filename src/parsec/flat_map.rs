use super::{Parser, ParserInput, ParserResult};

#[cfg(test)]
mod tests;

/// Passes the parsed output and remaining input to the next parsing step.
/// The callback may change input and states or return an error.
pub fn flat_map<'a, P, F, O>(parser: P, next: F) -> FlatMap<P, F>
where
    P: Parser<'a>,
    F: Fn(P::Output, ParserInput<'a>) -> ParserResult<'a, O>,
{
    FlatMap(parser, next)
}

#[derive(Debug)]
pub struct FlatMap<P, F>(P, F);

impl<'a, P, F, O> Parser<'a> for FlatMap<P, F>
where
    P: Parser<'a>,
    F: Fn(P::Output, ParserInput<'a>) -> ParserResult<'a, O>,
{
    type Output = O;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let (output, remaining) = self.0.parse(input)?;

        (self.1)(output, remaining)
    }
}
