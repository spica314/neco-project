use std::{collections::HashMap, rc::Rc};

use crate::{id::Id, term::Term};

/// Local context (typing context) that maps variable identifiers to their types.
/// In CIC, this represents Γ in the judgment Γ ⊢ t : T
#[derive(Debug, Clone)]
pub struct LocalContext {
    /// Maps variable IDs to their types
    bindings: HashMap<Id, Rc<Term>>,
    /// Ordered list of bindings for proper scoping
    /// This is important for dependent types where later bindings can refer to earlier ones
    order: Vec<Id>,
}

impl LocalContext {
    /// Creates an empty context
    pub fn new() -> Self {
        LocalContext {
            bindings: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Adds a new binding to the context
    /// Returns an error if the variable is already bound
    pub fn extend(&mut self, var: Id, ty: Rc<Term>) -> Result<(), String> {
        if self.bindings.contains_key(&var) {
            return Err(format!("Variable {var:?} is already bound in context"));
        }
        self.bindings.insert(var, ty);
        self.order.push(var);
        Ok(())
    }

    /// Creates a new context with an additional binding
    /// This is useful for immutable context extension
    pub fn with(&self, var: Id, ty: Rc<Term>) -> Self {
        let mut new_ctx = self.clone();
        let _ = new_ctx.extend(var, ty); // Ignore error for convenience in type checking
        new_ctx
    }

    /// Looks up the type of a variable
    pub fn lookup(&self, var: Id) -> Option<Rc<Term>> {
        self.bindings.get(&var).cloned()
    }

    /// Checks if a variable is bound in the context
    pub fn contains(&self, var: Id) -> bool {
        self.bindings.contains_key(&var)
    }

    /// Returns the number of bindings
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Checks if the context is empty
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Removes a binding from the context
    /// This is useful when exiting a scope
    pub fn remove(&mut self, var: Id) -> Option<Rc<Term>> {
        self.order.retain(|&v| v != var);
        self.bindings.remove(&var)
    }

    /// Returns an iterator over the bindings in order
    pub fn iter(&self) -> impl Iterator<Item = (Id, &Rc<Term>)> {
        self.order
            .iter()
            .filter_map(move |&id| self.bindings.get(&id).map(|ty| (id, ty)))
    }
}

impl Default for LocalContext {
    fn default() -> Self {
        Self::new()
    }
}
