use super::{value, Parser};

#[cfg(test)]
mod tests;

/// Discards successful output, preserving input consumption and errors.
pub fn void<'a, P: Parser<'a>>(parser: P) -> impl Parser<'a, Output = ()> {
    value(parser, ())
}
