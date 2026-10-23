use std::collections::HashMap;

use crate::parsec::ParseError;

use super::directives::{Directive, DirectiveContext};
use super::ir::ProcessedDocument;

mod action;
mod register;
mod document;
mod blocks;
mod err;

#[cfg(test)]
mod tests;

struct Processor<'a> {
    directives: HashMap<&'a str, Box<dyn Directive + 'a>>,
    context: DirectiveContext<'a>,
}

impl<'a> Processor<'a> {
    fn new() -> Self {
        Self {
            directives: HashMap::new(),
            context: DirectiveContext::default(),
        }
    }
}

type ProcessResult<'a> = Result<ProcessedDocument<'a>, Box<dyn ParseError>>;
