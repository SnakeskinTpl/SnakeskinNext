mod classic;
pub(crate) use classic::{classic_directive_candidate, classic_directive_start};

mod jade_like;
pub(crate) use jade_like::{jade_like_directive_candidate, jade_like_directive_start};

mod name;

/// A syntactic candidate; name and body remain tentative until directive resolution.
#[derive(Debug)]
pub(crate) struct DirectiveCandidate<'a> {
    pub form: DirectiveForm<'a>,
    pub extended: bool,
    pub name: Option<&'a str>,
    pub body: &'a str,
    /// Original directive content, before name or shorthand resolution.
    pub content: &'a str,
}

#[derive(Debug, PartialEq)]
pub(crate) enum DirectiveForm<'a> {
    Classic,
    JadeLike { indent: &'a str },
}
