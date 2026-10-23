use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Range;

use crate::parsec::{tag, Parser, ParserInput};

#[derive(Debug, Default)]
pub(crate) struct ShorthandRegistry<'a> {
    bindings: HashMap<Cow<'a, str>, ShorthandBinding<'a>>,
}

impl<'a> ShorthandRegistry<'a> {
    pub fn entry(&mut self, opening: Cow<'a, str>) -> Entry<'_, Cow<'a, str>, ShorthandBinding<'a>> {
        self.bindings.entry(opening)
    }

    pub fn get(&self, opening: &str) -> Option<&ShorthandBinding<'a>> {
        self.bindings.get(opening)
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Finds the longest opening at the cursor without consuming input.
    pub fn find_opening(&self, input: &ParserInput<'_>) -> Option<&ShorthandBinding<'a>> {
        self.bindings.values()
            .filter(|binding| !binding.opening.is_empty())
            .filter(|binding| tag(binding.opening.as_ref()).parse(input.clone()).is_ok())
            .max_by_key(|binding| binding.opening.len())
    }

    /// Finds a paired opening without a following JadeLike separator.
    pub fn find_classic_opening(&self, input: &ParserInput<'_>) -> Option<&ShorthandBinding<'a>> {
        self.bindings.values()
            .filter(|binding| !binding.opening.is_empty() && binding.closing.as_deref().is_some_and(|closing| !closing.is_empty()))

            .filter(|binding| {
                tag(binding.opening.as_ref()).parse(input.clone())
                    .is_ok_and(|(_, remaining)| !matches!(remaining.peek(), Some(' ' | '\t')))
            })

            .max_by_key(|binding| binding.opening.len())
    }
}

#[derive(Debug)]
pub(crate) struct ShorthandBinding<'a> {
    pub opening: Cow<'a, str>,
    pub closing: Option<Cow<'a, str>>,
    pub target: &'a str,
    pub span: Range<usize>,
    pub opening_span: Range<usize>,
}

