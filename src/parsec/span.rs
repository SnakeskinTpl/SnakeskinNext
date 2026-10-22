use std::ops::Range;

use super::{flat_map, parser, Parser};

#[cfg(test)]
mod tests;

/// Returns the consumed byte range and output, preserving remaining input and errors.
pub fn spanned<'a, P: Parser<'a>>(inner: P) -> impl Parser<'a, Output = (Range<usize>, P::Output)> {
    parser(move |input| {
        let start_byte = input.byte_position();
        let (output, remaining) = inner.parse(input)?;
        let span = start_byte..remaining.byte_position();

        Ok(((span, output), remaining))
    })
}

/// Returns the consumed source slice instead of the inner parser's output.
pub fn recognize<'a, P: Parser<'a>>(inner: P) -> impl Parser<'a, Output = &'a str> {
    flat_map(spanned(inner), |(span, _), input| {
        Ok((input.source_slice(span), input))
    })
}
