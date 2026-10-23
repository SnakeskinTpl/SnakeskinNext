//! Core directive implementations and body parsers.

use crate::core::shorthands::{ShorthandDefinition, ShorthandRegistry};
use crate::parsec::{ParseError, ParserInput};
use crate::core::syntax::expression::Expression;
use crate::core::types::TypeEnvironment;

mod err;

mod shorthand;
pub(crate) use shorthand::Shorthand;

mod end;
pub(crate) use end::End;

mod condition;
pub(crate) use condition::If;

mod variable;
pub(crate) use variable::{Let, Var};

pub(crate) trait Directive {
    fn definition(&self) -> &DirectiveDefinition<'_>;

    /// Parses the body and returns any action the processor must perform.
    fn begin<'a>(
        &self,
        context: &mut DirectiveContext<'a>,
        body: ParserInput<'a>,
    ) -> DirectiveBeginResult<'a>;

    /// Completes this occurrence; inline directives finish immediately after begin.
    fn end(&self, _context: &mut DirectiveContext<'_>) -> DirectiveResult {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct DirectiveDefinition<'a> {
    /// Full directive name, e.g. `shorthand`.
    pub name: &'a str,
    /// Alternative openings; explicit directive markers supply their own boundary.
    pub shorthands: Vec<ShorthandDefinition<'a>>,

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
    pub types: TypeEnvironment<'a>,
}

// Reuse located parser diagnostics until semantic diagnostics are defined.
pub(crate) type DirectiveResult = Result<(), Box<dyn ParseError>>;

#[derive(Debug)]
pub(crate) enum DirectiveAction<'a> {
    /// Continue normal inline or block processing.
    None,

    /// Close the current block, optionally checking its registered name.
    CloseBlock { name: Option<&'a str> },

    /// Preserve a checked condition; processing does not evaluate it.
    Conditional { condition: Expression<'a> },

    Declare { declaration: crate::core::ir::VariableDeclaration<'a> },
}

pub(crate) type DirectiveBeginResult<'a> = Result<DirectiveAction<'a>, Box<dyn ParseError>>;
