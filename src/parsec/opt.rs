use super::{Parser, ParserInput, ParserResult};

#[cfg(test)]
mod tests;

/// Returns None on a recoverable failure, restoring the original cursor and states.
/// Non-recoverable errors are propagated unchanged.
/// A successful match returns Some, including when no input is consumed.
pub fn opt<'a, P: Parser<'a>>(parser: P) -> Opt<P> {
    Opt(parser)
}

#[derive(Debug)]
pub struct Opt<P>(P);

impl<'a, P: Parser<'a>> Parser<'a> for Opt<P> {
    type Output = Option<P::Output>;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        match self.0.parse(input.clone()) {
            Ok((output, remaining)) => Ok((Some(output), remaining)),
            Err(error) if error.is_recoverable() => Ok((None, input)),
            Err(error) => Err(error),
        }
    }
}
