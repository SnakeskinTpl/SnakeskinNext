use super::*;

mod err;
pub use err::CutError;

#[cfg(test)]
mod tests;

/// Makes recoverable failures non-recoverable, preventing fallback in outer combinators.
pub fn cut<'a, P: Parser<'a>>(parser: P) -> Cut<P> {
    Cut(parser)
}

#[derive(Debug)]
pub struct Cut<P>(P);

impl<'a, P: Parser<'a>> Parser<'a> for Cut<P> {
    type Output = P::Output;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        self.0.parse(input).map_err(|cause| {
            if cause.is_recoverable() {
                CutError::new(cause) as Box<dyn ParseError>

            } else {
                cause
            }
        })
    }
}
