use std::collections::HashMap;

use crate::parsec::ParseError;

use super::directives::{Directive, DirectiveContext};
use super::ir::IrDocument;

mod action;
mod register;
mod process;
mod builder;
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

type ProcessResult<'a> = Result<IrDocument<'a>, Box<dyn ParseError>>;
