use super::*;

mod err;
pub use err::NotError;

#[cfg(test)]
mod tests;

/// Negative lookahead: succeeds only if the inner parser fails recoverably.
/// Non-recoverable errors are propagated unchanged.
/// Discards all cursor and state changes from the attempted parse.
pub fn not<'a, P: Parser<'a>>(parser: P) -> Not<P> {
    Not(parser)
}

#[derive(Debug)]
pub struct Not<P>(P);

impl<'a, P: Parser<'a>> Parser<'a> for Not<P> {
    type Output = ();

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        match self.0.parse(input.clone()) {
            Ok(_) => Err(NotError::new(ErrorLocation {
                start: input.byte_position(),
                byte_position: input.byte_position(),
                actual: input.peek(),
            })),
            Err(error) if error.is_recoverable() => Ok(((), input)),
            Err(error) => Err(error),
        }
    }
}
