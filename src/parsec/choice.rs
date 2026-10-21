use super::{Parser, ParserInput, ParserResult};
use super::or::select_error;

#[cfg(test)]
mod tests;

/// Tries alternatives in order, returning their common output type.
#[macro_export]
macro_rules! choice {
    ($parser:expr $(,)?) => {
        $parser
    };

    ($first:expr, $second:expr $(,)?) => {
        $crate::parsec::choice($first, $second)
    };

    ($first:expr, $second:expr, $($rest:expr),+ $(,)?) => {
        $crate::choice!(
            $crate::parsec::choice($first, $second),
            $($rest),+
        )
    };
}

/// Uses the same backtracking and error selection as `or`, without Either wrappers.
/// Non-recoverable errors stop alternative selection immediately.
pub fn choice<'a, A, B, O>(a: A, b: B) -> Choice<A, B>
where
    A: Parser<'a, Output = O>,
    B: Parser<'a, Output = O>,

{
    Choice(a, b)
}

#[derive(Debug)]
pub struct Choice<A, B>(A, B);

impl<'a, A, B, O> Parser<'a> for Choice<A, B>
where
    A: Parser<'a, Output = O>,
    B: Parser<'a, Output = O>,

{
    type Output = O;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let start_byte = input.byte_position();
        match self.0.parse(input.clone()) {
            Ok(result) => Ok(result),
            Err(first) if !first.is_recoverable() => Err(first),
            Err(first) => self.1.parse(input).map_err(|second| select_error(start_byte, first, second)),
        }
    }
}
