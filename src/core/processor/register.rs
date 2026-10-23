use std::borrow::Cow;
use std::collections::hash_map::Entry;

use crate::core::directives::Directive;
use crate::core::shorthands::ShorthandBinding;

use super::err::RegistrationError;
use super::Processor;

impl<'a> Processor<'a> {
    pub fn register_directive(
        &mut self,
        name: &'a str,
        directive: Box<dyn Directive + 'a>,
    ) -> Result<(), RegistrationError> {
        if self.directives.contains_key(name) {
            return Err(RegistrationError { name: name.into(), reason: "Directive is already registered" });
        }

        let shorthands = &directive.definition().shorthands;

        for (index, shorthand) in shorthands.iter().enumerate() {
            if shorthand.opening.is_empty() || shorthand.closing == Some("") {
                return Err(RegistrationError { name: name.into(), reason: "Empty shorthand delimiter" });
            }

            if self.context.shorthands.get(shorthand.opening).is_some() ||
               shorthands[..index].iter().any(|previous| previous.opening == shorthand.opening)
            {
                return Err(RegistrationError { name: shorthand.opening.into(), reason: "Shorthand is already registered" });
            }
        }

        let bindings: Vec<_> = shorthands.iter().map(|definition| ShorthandBinding {
            opening: Cow::Owned(definition.opening.into()),
            closing: definition.closing.map(|closing| Cow::Owned(closing.into())),
            target: name,
            span: 0..0,
            opening_span: 0..0,
        }).collect();

        match self.directives.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert(directive);

                for binding in bindings {
                    self.context.shorthands.entry(binding.opening.clone()).or_insert(binding);
                }

                Ok(())
            }

            Entry::Occupied(_) => Err(RegistrationError { name: name.into(), reason: "Directive is already registered" }),
        }
    }
}
