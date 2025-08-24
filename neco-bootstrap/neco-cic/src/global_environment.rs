use std::{collections::HashMap, rc::Rc};

use crate::{id::Id, inductive::InductiveEnvironment, term::Term};

/// Global environment containing definitions of constants and inductive types
#[derive(Debug, Clone)]
pub struct GlobalEnvironment {
    /// Constant definitions (axioms, definitions)
    pub constants: HashMap<Id, ConstantDefinition>,
    /// Inductive type definitions
    pub inductives: InductiveEnvironment,
}

/// Definition of a global constant
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantDefinition {
    /// Name of the constant
    pub name: Id,
    /// Type of the constant
    pub ty: Rc<Term>,
    /// Optional body (if this is a definition, not an axiom)
    pub body: Option<Rc<Term>>,
}

impl GlobalEnvironment {
    /// Creates a new empty global environment
    pub fn new() -> Self {
        GlobalEnvironment {
            constants: HashMap::new(),
            inductives: InductiveEnvironment::new(),
        }
    }

    /// Adds a constant definition
    pub fn add_constant(&mut self, def: ConstantDefinition) -> Result<(), String> {
        if self.constants.contains_key(&def.name) {
            return Err(format!("Constant {:?} already defined", def.name));
        }
        self.constants.insert(def.name, def);
        Ok(())
    }

    /// Gets a constant definition
    pub fn get_constant(&self, name: Id) -> Option<&ConstantDefinition> {
        self.constants.get(&name)
    }

    /// Adds an axiom (constant without body)
    pub fn add_axiom(&mut self, name: Id, ty: Rc<Term>) -> Result<(), String> {
        let def = ConstantDefinition {
            name,
            ty,
            body: None,
        };
        self.add_constant(def)
    }

    /// Adds a definition (constant with body)
    pub fn add_definition(&mut self, name: Id, ty: Rc<Term>, body: Rc<Term>) -> Result<(), String> {
        let def = ConstantDefinition {
            name,
            ty,
            body: Some(body),
        };
        self.add_constant(def)
    }
}

impl Default for GlobalEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstantDefinition {
    /// Creates a new axiom
    pub fn new_axiom(name: Id, ty: Rc<Term>) -> Self {
        ConstantDefinition {
            name,
            ty,
            body: None,
        }
    }

    /// Creates a new definition
    pub fn new_definition(name: Id, ty: Rc<Term>, body: Rc<Term>) -> Self {
        ConstantDefinition {
            name,
            ty,
            body: Some(body),
        }
    }

    /// Checks if this is an axiom (no body)
    pub fn is_axiom(&self) -> bool {
        self.body.is_none()
    }

    /// Checks if this is a definition (has body)
    pub fn is_definition(&self) -> bool {
        self.body.is_some()
    }
}
