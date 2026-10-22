use super::{choice, map, Parser};

mod err;
pub use err::AlternativeError;
pub(crate) use err::select_error;

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
pub fn or<'a, A: Parser<'a>, B: Parser<'a>>(a: A, b: B) -> impl Parser<'a, Output = Either<A::Output, B::Output>> {
    choice(
        map(a, Either::Left),
        map(b, Either::Right)
    )
}

/// Identifies which alternative succeeded, allowing different output types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}
