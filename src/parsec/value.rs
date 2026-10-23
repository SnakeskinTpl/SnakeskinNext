use super::{map, Parser};

#[cfg(test)]
mod tests;

/// Replaces successful output with a clone of the supplied value; errors remain unchanged.
pub fn value<'a, P, O>(parser: P, value: O) -> impl Parser<'a, Output = O>
where
    P: Parser<'a>,
    O: Clone,
{
    map(parser, move |_| value.clone())
}
