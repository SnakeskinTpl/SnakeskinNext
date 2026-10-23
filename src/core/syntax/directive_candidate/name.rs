use crate::core::syntax::identifier;
use crate::core::shorthands::ShorthandRegistry;
use crate::parsec::prelude::*;

/// Parses an identifier followed by a required whitespace separator.
pub(crate) fn directive_name<'a>() -> impl Parser<'a, Output = &'a str> {
    map(
        seq!(
            identifier(),
            // Required separator: whitespace, e.g. a space in "foo value" or a newline.
            whitespace()
        ),
        |(name, _)| name
    )
}

/// Resolves an opening inside explicit directive markers, preserving the remaining body.
pub(crate) fn directive_content<'a>(
    shorthands: Option<&ShorthandRegistry<'a>>,
) -> impl Parser<'a, Output = (Option<&'a str>, &'a str)> {
    parser(move |input| {
        if let Some(binding) = shorthands.and_then(|registry| registry.find_opening(&input)) {
            let (_, input) = tag(binding.opening.as_ref()).parse(input)?;

            return map(
                ltrim(rest()),
                |body| (Some(binding.target), body)
            ).parse(input);
        }

        seq(
            opt(directive_name()),
            rest()
        ).parse(input)
    })
}
