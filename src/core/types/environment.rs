use std::collections::HashMap;

use super::Type;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct Binding {
    pub ty: Type,

    // TODO: Enforce binding mutability when assignment expressions are implemented.
    pub mutable: bool,
}

/// Inner scopes shadow outer bindings; duplicate names in one scope are rejected.
#[derive(Debug, Clone)]
pub(crate) struct TypeEnvironment<'a> {
    scopes: Vec<HashMap<&'a str, Binding>>,
}

impl<'a> TypeEnvironment<'a> {
    pub fn new() -> Self { Self::default() }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.binding(name).map(|binding| &binding.ty)
    }

    pub fn binding(&self, name: &str) -> Option<&Binding> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    /// Seeds the current scope; directive declarations use declare to reject duplicates.
    pub fn insert(&mut self, name: &'a str, ty: Type) -> Option<Type> {
        self.scopes.last_mut().and_then(|scope| {
            scope.insert(name, Binding { ty, mutable: false }).map(|binding| binding.ty)
        })
    }

    /// Returns false if this scope already contains the name.
    pub fn declare(&mut self, name: &'a str, ty: Type, mutable: bool) -> bool {
        use std::collections::hash_map::Entry;

        if let Some(scope) = self.scopes.last_mut() {
            if let Entry::Vacant(entry) = scope.entry(name) {
                entry.insert(Binding { ty, mutable });
                return true;
            }
        }

        false
    }

    pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 { self.scopes.pop(); }
    }

    pub fn len(&self) -> usize {
        let mut visible = HashMap::new();
        for scope in &self.scopes { visible.extend(scope.iter()); }
        visible.len()
    }
}

impl Default for TypeEnvironment<'_> {
    fn default() -> Self { Self { scopes: vec![HashMap::new()] } }
}

impl<'a, const N: usize> From<[(&'a str, Type); N]> for TypeEnvironment<'a> {
    fn from(bindings: [(&'a str, Type); N]) -> Self {
        let mut environment = Self::new();
        for (name, ty) in bindings { environment.insert(name, ty); }
        environment
    }
}
