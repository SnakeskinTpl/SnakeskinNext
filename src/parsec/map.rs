use super::*;

#[cfg(test)]
mod tests;

/// Transforms successful output without changing the remaining input or states.
pub fn map<'a, P, F, O>(parser: P, transform: F) -> Map<P, F>
where
    P: Parser<'a>,
    F: Fn(P::Output) -> O,
{
    Map(parser, transform)
}

#[derive(Debug)]
pub struct Map<P, F>(P, F);

impl<'a, P, F, O> Parser<'a> for Map<P, F>
where
    P: Parser<'a>,
    F: Fn(P::Output) -> O,
{
    type Output = O;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let (output, remaining) = self.0.parse(input)?;
        Ok(((self.1)(output), remaining))
    }
}
