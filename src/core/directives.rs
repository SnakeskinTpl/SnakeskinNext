//! Core directive implementations and body parsers.

use std::borrow::Cow;
use std::collections::HashMap;

use crate::parsec::{ParseError, ParserInput};

mod shorthand;
pub(crate) use shorthand::{Shorthand, ShorthandBinding};

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

#[derive(Debug)]
pub(crate) struct DirectiveDefinition<'a> {
    pub name: &'a str,

    pub block: bool,
    pub requires_body: bool,
    pub allows_self_nesting: bool,

    pub groups: Vec<&'a str>,

    /// None allows any parent; an empty list allows none.
    pub allowed_parents: Option<Vec<DirectiveSelector<'a>>>,

    /// None allows any child; an empty list allows none.
    pub allowed_children: Option<Vec<DirectiveSelector<'a>>>,
    pub ancestors: AncestorRules<'a>,

    /// Constructs this directive may continue, such as else continuing if.
    pub continues: Vec<DirectiveSelector<'a>>,
}

#[derive(Debug)]
pub(crate) enum DirectiveSelector<'a> {
    Name(&'a str),
    Group(&'a str),
}

#[derive(Debug)]
pub(crate) struct AncestorRules<'a> {
    /// None imposes no ancestor requirement.
    pub allowed: Option<Vec<DirectiveSelector<'a>>>,
    pub forbidden: Vec<DirectiveSelector<'a>>,
}

/// Borrows declarations from language-module sources that outlive processing.
#[derive(Debug, Default)]
pub(crate) struct DirectiveContext<'a> {
    pub shorthands: HashMap<Cow<'a, str>, ShorthandBinding<'a>>,
}

// Reuse located parser diagnostics until semantic diagnostics are defined.
pub(crate) type DirectiveResult = Result<(), Box<dyn ParseError>>;
