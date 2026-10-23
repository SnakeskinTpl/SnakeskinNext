use crate::parsec::prelude::*;
use crate::parsec::ErrorLocation;

mod err;
use err::NumberLiteralError;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NumberLiteral<'a> {
    /// Exact decimal spelling without the type suffix; no floating-point conversion.
    pub content: &'a str,
    /// Syntactic form, independent of the contextual numeric type.
    pub form: NumberForm,
    /// Explicit type hint; absent suffixes are resolved from context and defaults.
    pub suffix: Option<NumberSuffix<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NumberForm {
    Integer,
    /// Fractional or exponential decimal notation, e.g. `1.5` or `1e3`.
    Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NumberSuffix<'a> {
    Int { bits: Option<&'a str> },
    UInt { bits: Option<&'a str> },
    Float { bits: Option<&'a str> },
}

/// Parses an unsigned decimal spelling; signs belong to the expression grammar.
/// Suffix widths and value ranges are validated after contextual type resolution.
pub(crate) fn number<'a>() -> impl Parser<'a, Output = NumberLiteral<'a>> {
    flat_map(spanned(digits()), |(integer_span, _), input| {
        // A single dot begins a fraction; `1..2` leaves the range operator untouched.
        let fraction = seq(
            seq(tag("."), not(tag("."))),
            cut(digits())
        );

        // Once e/E is present, an optional sign must be followed by exponent digits.
        let exponent = seq!(
            choice(tag("e"), tag("E")),
            opt(choice(tag("+"), tag("-"))),
            cut(digits())
        );

        let grammar = seq!(
            spanned(seq(opt(fraction), opt(exponent))),
            opt(spanned(suffix())),
            not(take(|ch, _, _| ch.is_alphanumeric() || ch == '_', 1..=1))
        );

        flat_map(cut(grammar), move |(((magnitude_span, (fraction, exponent)), suffix), ()), input| {
            let content = input.source_slice(integer_span.start..magnitude_span.end);

            let form = if fraction.is_some() || exponent.is_some() {
                NumberForm::Decimal

            } else {
                NumberForm::Integer
            };

            // Integer suffixes cannot describe fractional or exponential notation.
            if let Some((span, NumberSuffix::Int { .. } | NumberSuffix::UInt { .. })) = &suffix {
                if form == NumberForm::Decimal {
                    return Err(NumberLiteralError::new(ErrorLocation {
                        start_byte: integer_span.start,
                        failure_byte: span.start,
                        found_char: input.source_slice(span.clone()).chars().next(),
                    }));
                }
            }

            let suffix = suffix.map(|(_, suffix)| suffix);
            Ok((NumberLiteral { content, form, suffix }, input))
        }).parse(input)
    })
}

/// E.g. `u` uses the default unsigned width, while `i16` explicitly requests 16 bits.
fn suffix<'a>() -> impl Parser<'a, Output = NumberSuffix<'a>> {
    map(
        seq(
            choice!(tag("i"), tag("u"), tag("f")),
            opt(take(|ch, _, _| ch.is_ascii_digit(), 1..))
        ),
        |(kind, bits)| match kind {
            "i" => NumberSuffix::Int { bits },
            "u" => NumberSuffix::UInt { bits },
            _ => NumberSuffix::Float { bits },
        }
    )
}

/// Reads decimal digits with optional underscore separators, e.g. `100_333_454`.
fn digits<'a>() -> impl Parser<'a, Output = &'a str> {
    let group = || take(|ch, _, _| ch.is_ascii_digit(), 1..);

    recognize(seq(
        group(),
        repeat(seq(tag("_"), cut(group())), 0..)
    ))
}
