use super::{flat_map, Parser};

#[cfg(test)]
mod tests;

/// Transforms successful output without changing the remaining input.
pub fn map<'a, P, F, O>(parser: P, transform: F) -> impl Parser<'a, Output = O>
where
    P: Parser<'a>,
    F: Fn(P::Output) -> O,
{
    flat_map(parser, move |output, input| Ok((transform(output), input)))
}
