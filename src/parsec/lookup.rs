use super::*;

#[cfg(test)]
mod tests;

/// Requires a successful match, discarding its output and input changes.
/// Propagates the inner parser's error unchanged on failure.
pub fn lookup<'a, P: Parser<'a>>(parser: P) -> Lookup<P> {
    Lookup(parser)
}

#[derive(Debug)]
pub struct Lookup<P>(P);

impl<'a, P: Parser<'a>> Parser<'a> for Lookup<P> {
    type Output = ();

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        self.0.parse(input.clone())?;
        Ok(((), input))
    }
}
