use super::{Parser, ParserInput, ParserResult};

#[cfg(test)]
mod tests;

/// Sequences parsers, nesting outputs to the left: ((A, B), C).
#[macro_export]
macro_rules! seq {
    ($parser:expr $(,)?) => {
        $parser
    };

    ($first:expr, $second:expr $(,)?) => {
        $crate::parsec::seq($first, $second)
    };

    ($first:expr, $second:expr, $($rest:expr),+ $(,)?) => {
        $crate::seq!(
            $crate::parsec::seq($first, $second),
            $($rest),+
        )
    };
}

/// Runs the second parser on the input returned by the first.
pub fn seq<'a, A: Parser<'a>, B: Parser<'a>>(a: A, b: B) -> Seq<A, B> {
    Seq(a, b)
}

#[derive(Debug)]
pub struct Seq<A, B>(A, B);

impl<'a, A: Parser<'a>, B: Parser<'a>> Parser<'a> for Seq<A, B> {
    type Output = (A::Output, B::Output);

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let (output_a, remaining) = self.0.parse(input)?;
        let (output_b, remaining) = self.1.parse(remaining)?;
        Ok(((output_a, output_b), remaining))
    }
}
