use super::{Parser, ParserInput, ParserResult};

/// Wraps a parsing function without adding state or changing its result.
pub fn parser<'a, F, O>(parse: F) -> ParserFn<F>
where
    F: Fn(ParserInput<'a>) -> ParserResult<'a, O>,
{
    ParserFn(parse)
}

pub struct ParserFn<F>(F);

impl<'a, F, O> Parser<'a> for ParserFn<F>
where
    F: Fn(ParserInput<'a>) -> ParserResult<'a, O>,
{
    type Output = O;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        self.0(input)
    }
}
