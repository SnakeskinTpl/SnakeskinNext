use super::*;

#[cfg(test)]
mod tests;

/// Defers parser construction until parsing, allowing recursive grammars.
/// Calls the factory on every parse attempt; the parser is not cached.
/// Recursive parser types may require indirection through Box<dyn Parser>.
/// Left recursion is unsupported; nested calls must eventually consume input.
/// Nesting depth is limited by the call stack.
pub fn rec<'a, F, P>(f: F) -> Rec<F> where
    F: Fn() -> P,
    P: Parser<'a>,
{
    Rec(f)
}

pub struct Rec<F>(F);

impl<'a, F, P> Parser<'a> for Rec<F>
where
    F: Fn() -> P,
    P: Parser<'a>
{
    type Output = P::Output;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        self.0().parse(input)
    }
}
