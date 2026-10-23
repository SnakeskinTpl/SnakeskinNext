//! Core directive implementations and body parsers.

use crate::core::shorthands::ShorthandRegistry;
use crate::parsec::{ParseError, ParserInput};

mod shorthand;
pub(crate) use shorthand::Shorthand;

pub(crate) trait Directive {
    fn definition(&self) -> &DirectiveDefinition<'_>;

    /// Parses the directive body and begins processing this occurrence.
    fn begin<'a>(
        &self,
        context: &mut DirectiveContext<'a>,
        body: ParserInput<'a>,
    ) -> DirectiveResult;

    /// Completes this occurrence; inline directives finish immediately after begin.
    fn end(&self, _context: &mut DirectiveContext<'_>) -> DirectiveResult {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct DirectiveDefinition<'a> {
    /// Full directive name, e.g. `shorthand`.
    pub name: &'a str,

    /// Opens a block requiring an explicit or inferred end.
    pub block: bool,
    /// Requires non-whitespace content after the name.
    pub requires_body: bool,
    /// Allows another occurrence of this directive inside its own block.
    pub allows_self_nesting: bool,

    /// Named groups used by placement selectors.
    pub groups: Vec<&'a str>,

    /// Allowed immediate parents; None allows any, an empty list allows none.
    pub allowed_parents: Option<Vec<DirectiveSelector<'a>>>,

    /// Allowed immediate children; None allows any, an empty list allows none.
    pub allowed_children: Option<Vec<DirectiveSelector<'a>>>,
    /// Placement constraints checked against the ancestor chain.
    pub ancestors: AncestorRules<'a>,

    /// Constructs this directive may continue, such as else continuing if.
    pub continues: Vec<DirectiveSelector<'a>>,
}

#[derive(Debug)]
pub(crate) enum DirectiveSelector<'a> {
    Name(&'a str),
    Group(&'a str),
}

#[derive(Debug, Default)]
pub(crate) struct AncestorRules<'a> {
    /// None imposes no ancestor requirement.
    pub allowed: Option<Vec<DirectiveSelector<'a>>>,
    pub forbidden: Vec<DirectiveSelector<'a>>,
}

/// Borrows declarations from language-module sources that outlive processing.
#[derive(Debug, Default)]
pub(crate) struct DirectiveContext<'a> {
    pub shorthands: ShorthandRegistry<'a>,
}

// Reuse located parser diagnostics until semantic diagnostics are defined.
pub(crate) type DirectiveResult = Result<(), Box<dyn ParseError>>;
