use super::{take, Parser};

#[cfg(test)]
mod tests;

/// Consumes all remaining input and returns its original source slice, including empty input.
pub fn rest<'a>() -> impl Parser<'a, Output = &'a str> {
    take(|_, _, _| true, 0..)
}
