use super::{flat_map, take, Parser};

#[cfg(test)]
mod tests;

/// Consumes one or more Unicode whitespace characters, including line breaks.
pub fn whitespace<'a>() -> impl Parser<'a, Output = &'a str> {
    take(|ch, _, _| ch.is_whitespace(), 1..)
}

/// Consumes one or more ASCII spaces or tabs, leaving line breaks untouched.
pub fn horizontal_whitespace<'a>() -> impl Parser<'a, Output = &'a str> {
    take(|ch, _, _| ch == ' ' || ch == '\t', 1..)
}

/// Skips leading Unicode whitespace, including line breaks, then runs the parser.
/// Leaves trailing whitespace untouched and calls the parser even at end of input.
pub fn ltrim<'a, P: Parser<'a>>(inner: P) -> impl Parser<'a, Output = P::Output> {
    flat_map(
        take(|ch, _, _| ch.is_whitespace(), 0..),
        move |_, input| inner.parse(input)
    )
}

/// Requires leading Unicode whitespace, then returns only the inner parser's output.
pub fn ltrim1<'a, P: Parser<'a>>(inner: P) -> impl Parser<'a, Output = P::Output> {
    flat_map(whitespace(), move |_, input| inner.parse(input))
}

