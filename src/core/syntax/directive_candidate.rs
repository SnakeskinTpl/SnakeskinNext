mod classic;
pub(crate) use classic::{classic_directive_candidate, classic_directive_start};

mod jade_like;
pub(crate) use jade_like::{jade_like_directive_candidate, jade_like_directive_start};

mod name;

/// A syntactic candidate; name and body remain tentative until directive resolution.
#[derive(Debug)]
pub(crate) struct DirectiveCandidate<'a> {
    form: DirectiveForm<'a>,
    extended: bool,
    name: Option<&'a str>,
    body: &'a str,
    /// Original content between braces, before name or shorthand resolution.
    content: &'a str,
}

#[derive(Debug, PartialEq)]
pub(crate) enum DirectiveForm<'a> {
    Classic,
    JadeLike { indent: &'a str },
}
