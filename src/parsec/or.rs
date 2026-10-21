use super::{Parser, ParserInput, ParserResult};

mod err;
pub use err::AlternativeError;
pub(super) use err::select_error;

#[cfg(test)]
mod tests;

/// Tries alternatives in order, nesting outputs as Either<Either<A, B>, C>.
#[macro_export]
macro_rules! or {
    ($parser:expr $(,)?) => {
        $parser
    };

    ($first:expr, $second:expr $(,)?) => {
        $crate::parsec::or($first, $second)
    };

    ($first:expr, $second:expr, $($rest:expr),+ $(,)?) => {
        $crate::or!(
            $crate::parsec::or($first, $second),
            $($rest),+
        )
    };
}

/// Tries the second parser from the original input if the first fails.
/// Returns the first success, not the longest match.
/// On failure, keeps the furthest error, or both errors at the same position.
/// Non-recoverable errors stop alternative selection immediately.
pub fn or<'a, A: Parser<'a>, B: Parser<'a>>(a: A, b: B) -> Or<A, B> {
    Or(a, b)
}

#[derive(Debug)]
pub struct Or<A, B>(A, B);

/// Identifies which alternative succeeded, allowing different output types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<'a, A: Parser<'a>, B: Parser<'a>> Parser<'a> for Or<A, B> {
    type Output = Either<A::Output, B::Output>;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let start = input.byte_position();
        match self.0.parse(input.clone()) {
            Ok((output, remaining)) => {
                let output = Either::Left(output);
                Ok((output, remaining))
            }

            Err(first) if !first.is_recoverable() => Err(first),
            Err(first) => {
                match self.1.parse(input) {
                    Ok((output, remaining)) => Ok((Either::Right(output), remaining)),
                    Err(second) => Err(select_error(start, first, second)),
                }
            },
        }
    }
}
