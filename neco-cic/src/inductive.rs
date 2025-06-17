use std::{collections::HashMap, rc::Rc};

use crate::{
    id::{Id, IdGenerator},
    term::Term,
};

/// Definition of an inductive type
/// Example: Inductive nat : Set := O : nat | S : nat -> nat
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InductiveDefinition {
    /// Name of the inductive type
    pub name: Id,
    /// Parameters of the inductive type (for parametric types like List A)
    pub parameters: Vec<Parameter>,
    /// Sort that this inductive type lives in (Set, Prop, Type(i))
    pub sort: Rc<Term>,
    /// Constructors of this inductive type
    pub constructors: Vec<ConstructorDefinition>,
}

/// A parameter of an inductive type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Id,
    pub ty: Rc<Term>,
}

/// Definition of a constructor
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructorDefinition {
    /// Name of the constructor
    pub name: Id,
    /// Type of the constructor (in telescope form)
    /// Example: for S : nat -> nat, this would be [nat] -> nat
    pub ty: Rc<Term>,
    /// Number of arguments this constructor takes
    pub arity: usize,
}

/// Helper functions for working with inductive types
impl InductiveDefinition {
    /// Creates a new inductive definition
    pub fn new(
        name: Id,
        parameters: Vec<Parameter>,
        sort: Rc<Term>,
        constructors: Vec<ConstructorDefinition>,
    ) -> Self {
        InductiveDefinition {
            name,
            parameters,
            sort,
            constructors,
        }
    }

    /// Gets the full type of the inductive type including parameters
    /// Example: for List A, this returns A -> Set (if A : Set)
    pub fn get_type(&self) -> Rc<Term> {
        if self.parameters.is_empty() {
            self.sort.clone()
        } else {
            // Build a product type: Π(p1: T1). ... Π(pn: Tn). sort
            let mut result = self.sort.clone();
            for param in self.parameters.iter().rev() {
                result = Rc::new(Term::Product(crate::term::TermProduct {
                    var: param.name,
                    source: param.ty.clone(),
                    target: result,
                }));
            }
            result
        }
    }

    /// Finds a constructor by name
    pub fn find_constructor(&self, name: Id) -> Option<&ConstructorDefinition> {
        self.constructors.iter().find(|c| c.name == name)
    }

    /// Gets the number of constructors
    pub fn constructor_count(&self) -> usize {
        self.constructors.len()
    }
}

impl ConstructorDefinition {
    /// Creates a new constructor definition
    pub fn new(name: Id, ty: Rc<Term>, arity: usize) -> Self {
        ConstructorDefinition { name, ty, arity }
    }
}

/// Collection of inductive type definitions
#[derive(Debug, Clone, Default)]
pub struct InductiveEnvironment {
    /// Maps inductive type names to their definitions
    definitions: HashMap<Id, InductiveDefinition>,
    /// Maps constructor names to their inductive type
    constructor_to_inductive: HashMap<Id, Id>,
}

impl InductiveEnvironment {
    /// Creates a new empty inductive environment
    pub fn new() -> Self {
        InductiveEnvironment {
            definitions: HashMap::new(),
            constructor_to_inductive: HashMap::new(),
        }
    }

    /// Adds an inductive definition
    pub fn add_inductive(&mut self, def: InductiveDefinition) -> Result<(), String> {
        if self.definitions.contains_key(&def.name) {
            return Err(format!("Inductive type {:?} already defined", def.name));
        }

        // Register all constructors
        for constructor in &def.constructors {
            if self
                .constructor_to_inductive
                .contains_key(&constructor.name)
            {
                return Err(format!(
                    "Constructor {:?} already defined",
                    constructor.name
                ));
            }
            self.constructor_to_inductive
                .insert(constructor.name, def.name);
        }

        self.definitions.insert(def.name, def);
        Ok(())
    }

    /// Gets an inductive definition by name
    pub fn get_inductive(&self, name: Id) -> Option<&InductiveDefinition> {
        self.definitions.get(&name)
    }

    /// Gets the inductive type that a constructor belongs to
    pub fn get_inductive_for_constructor(&self, constructor: Id) -> Option<Id> {
        self.constructor_to_inductive.get(&constructor).copied()
    }

    /// Gets a constructor definition
    pub fn get_constructor(&self, constructor: Id) -> Option<&ConstructorDefinition> {
        let inductive_id = self.get_inductive_for_constructor(constructor)?;
        let inductive_def = self.get_inductive(inductive_id)?;
        inductive_def.find_constructor(constructor)
    }

    /// Lists all inductive types
    pub fn list_inductives(&self) -> impl Iterator<Item = &InductiveDefinition> {
        self.definitions.values()
    }
}

/// Helper functions for building common inductive types
impl InductiveEnvironment {
    /// Creates the standard Bool inductive type
    /// Inductive Bool : Set := True : Bool | False : Bool
    pub fn add_bool(&mut self, bool_id: Id, true_id: Id, false_id: Id) -> Result<(), String> {
        let set = Rc::new(Term::Sort(crate::term::TermSort {
            sort: crate::term::Sort::Set,
        }));
        let bool_type = Rc::new(Term::Constant(crate::term::TermConstant { id: bool_id }));

        let true_constructor = ConstructorDefinition::new(true_id, bool_type.clone(), 0);
        let false_constructor = ConstructorDefinition::new(false_id, bool_type, 0);

        let bool_def = InductiveDefinition::new(
            bool_id,
            vec![],
            set,
            vec![true_constructor, false_constructor],
        );

        self.add_inductive(bool_def)
    }

    /// Creates the standard Nat inductive type
    /// Inductive Nat : Set := O : Nat | S : Nat -> Nat
    pub fn add_nat(&mut self, nat_id: Id, zero_id: Id, succ_id: Id) -> Result<(), String> {
        let set = Rc::new(Term::Sort(crate::term::TermSort {
            sort: crate::term::Sort::Set,
        }));
        let nat_type = Rc::new(Term::Constant(crate::term::TermConstant { id: nat_id }));

        let zero_constructor = ConstructorDefinition::new(zero_id, nat_type.clone(), 0);

        // S : Nat -> Nat
        let mut id_gen = IdGenerator::new();
        let succ_type = Rc::new(Term::Product(crate::term::TermProduct {
            var: id_gen.generate_id(), // dummy variable
            source: nat_type.clone(),
            target: nat_type,
        }));
        let succ_constructor = ConstructorDefinition::new(succ_id, succ_type, 1);

        let nat_def = InductiveDefinition::new(
            nat_id,
            vec![],
            set,
            vec![zero_constructor, succ_constructor],
        );

        self.add_inductive(nat_def)
    }
}
