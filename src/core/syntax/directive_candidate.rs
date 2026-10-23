mod classic;
pub(crate) use classic::{classic_directive_candidate, classic_directive_start};

mod jade_like;
pub(crate) use jade_like::{jade_like_directive_candidate, jade_like_directive_start};

mod name;

/// A syntactic candidate; name and body remain tentative until directive resolution.
#[derive(Debug)]
pub(crate) struct DirectiveCandidate<'a> {
    /// Braced or indentation-based syntax.
    pub form: DirectiveForm<'a>,
    /// Uses the hash marker, e.g. `#{foo bar}` or `# foo bar`.
    pub extended: bool,
    /// Bytes before content, including indentation, marker and separator.
    pub opening_len: usize,
    /// Bytes after content occupied by the closing delimiter.
    pub closing_len: usize,
    /// Original content without opening or closing, e.g. `foo bar` in `{foo bar}`.
    pub content: &'a str,
    /// Tentative identifier, e.g. `foo` in `{foo bar}`; unresolved for `{1 + 2}`.
    pub name: Option<&'a str>,
    /// Content after the tentative name and separator, or all content if unnamed.
    pub body: &'a str,
}

#[derive(Debug, PartialEq)]
pub(crate) enum DirectiveForm<'a> {
    /// Explicit delimiters, e.g. `{foo bar}`.
    Classic,
    /// A line directive, e.g. `  - foo bar`.
    JadeLike {
        /// Original leading spaces and tabs, e.g. `  `.
        indent: &'a str,
    },
}
