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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        global_environment::GlobalEnvironment,
        id::Id,
        local_context::LocalContext,
        reduction::reduce_step,
        term::{MatchBranch, Sort, Term, TermConstant, TermMatch, TermSort, TermVariable},
        typechecker::infer_type,
    };

    use super::{ConstructorDefinition, InductiveDefinition, InductiveEnvironment, Parameter};

    #[test]
    fn test_bool_inductive_definition() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives.add_bool(bool_id, true_id, false_id).unwrap();

        let bool_def = env.inductives.get_inductive(bool_id).unwrap();
        assert_eq!(bool_def.name, bool_id);
        assert_eq!(bool_def.constructors.len(), 2);
    }

    #[test]
    fn test_bool_constructor_lookup() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives.add_bool(bool_id, true_id, false_id).unwrap();

        let true_constr = env.inductives.get_constructor(true_id).unwrap();
        assert_eq!(true_constr.name, true_id);
        assert_eq!(true_constr.arity, 0);
    }

    #[test]
    fn test_nat_inductive_definition() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let nat_def = env.inductives.get_inductive(nat_id).unwrap();
        assert_eq!(nat_def.name, nat_id);
        assert_eq!(nat_def.constructors.len(), 2);
    }

    #[test]
    fn test_nat_zero_constructor() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let zero_constr = env.inductives.get_constructor(zero_id).unwrap();
        assert_eq!(zero_constr.name, zero_id);
        assert_eq!(zero_constr.arity, 0);
    }

    #[test]
    fn test_nat_succ_constructor() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let succ_constr = env.inductives.get_constructor(succ_id).unwrap();
        assert_eq!(succ_constr.name, succ_id);
        assert_eq!(succ_constr.arity, 1);
    }

    #[test]
    fn test_bool_true_constructor_typing() {
        let mut env = GlobalEnvironment::new();
        let ctx = LocalContext::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives.add_bool(bool_id, true_id, false_id).unwrap();

        let true_constr = Term::Constant(TermConstant { id: true_id });

        let ty = infer_type(&ctx, &env, &true_constr).unwrap();
        assert_eq!(*ty, Term::Constant(TermConstant { id: bool_id }));
    }

    #[test]
    fn test_bool_false_constructor_typing() {
        let mut env = GlobalEnvironment::new();
        let ctx = LocalContext::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives.add_bool(bool_id, true_id, false_id).unwrap();

        let false_constr = Term::Constant(TermConstant { id: false_id });

        let ty = infer_type(&ctx, &env, &false_constr).unwrap();
        assert_eq!(*ty, Term::Constant(TermConstant { id: bool_id }));
    }

    #[test]
    fn test_case_reduction_true_branch() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives.add_bool(bool_id, true_id, false_id).unwrap();

        let true_constr = Term::Constant(TermConstant { id: true_id });
        let false_constr = Term::Constant(TermConstant { id: false_id });

        let case_expr = Term::Match(TermMatch {
            scrutinee: Rc::new(true_constr.clone()),
            return_type: Rc::new(Term::Constant(TermConstant { id: bool_id })),
            branches: vec![
                MatchBranch {
                    constructor_id: true_id,
                    bound_vars: vec![],
                    body: Rc::new(true_constr.clone()),
                },
                MatchBranch {
                    constructor_id: false_id,
                    bound_vars: vec![],
                    body: Rc::new(false_constr),
                },
            ],
        });

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, true_constr);
    }

    #[test]
    fn test_case_reduction_false_branch() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives.add_bool(bool_id, true_id, false_id).unwrap();

        let true_constr = Term::Constant(TermConstant { id: true_id });
        let false_constr = Term::Constant(TermConstant { id: false_id });

        let case_expr = Term::Match(TermMatch {
            scrutinee: Rc::new(false_constr.clone()),
            return_type: Rc::new(Term::Constant(TermConstant { id: bool_id })),
            branches: vec![
                MatchBranch {
                    constructor_id: true_id,
                    bound_vars: vec![],
                    body: Rc::new(true_constr),
                },
                MatchBranch {
                    constructor_id: false_id,
                    bound_vars: vec![],
                    body: Rc::new(false_constr.clone()),
                },
            ],
        });

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, false_constr);
    }

    #[test]
    fn test_nat_case_zero_branch() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let zero = Term::Constant(TermConstant { id: zero_id });

        let case_expr = Term::Match(TermMatch {
            scrutinee: Rc::new(zero.clone()),
            return_type: Rc::new(Term::Constant(TermConstant { id: nat_id })),
            branches: vec![
                MatchBranch {
                    constructor_id: zero_id,
                    bound_vars: vec![],
                    body: Rc::new(zero.clone()),
                },
                MatchBranch {
                    constructor_id: succ_id,
                    bound_vars: vec![Id::new()],
                    body: Rc::new(zero.clone()),
                },
            ],
        });

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, zero);
    }

    #[test]
    fn test_nat_case_succ_branch_with_substitution() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let n = Id::new();

        let zero = Term::Constant(TermConstant { id: zero_id });

        let one = Term::Application(crate::term::TermApplication {
            f: Rc::new(Term::Constant(TermConstant { id: succ_id })),
            args: vec![zero.clone()],
        });

        let two = Term::Application(crate::term::TermApplication {
            f: Rc::new(Term::Constant(TermConstant { id: succ_id })),
            args: vec![one.clone()],
        });

        let case_expr = Term::Match(TermMatch {
            scrutinee: Rc::new(two),
            return_type: Rc::new(Term::Constant(TermConstant { id: nat_id })),
            branches: vec![
                MatchBranch {
                    constructor_id: zero_id,
                    bound_vars: vec![],
                    body: Rc::new(zero),
                },
                MatchBranch {
                    constructor_id: succ_id,
                    bound_vars: vec![n],
                    body: Rc::new(Term::Variable(TermVariable { id: n })),
                },
            ],
        });

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, one);
    }

    #[test]
    fn test_list_inductive_definition() {
        let mut inductive_env = InductiveEnvironment::new();

        let list_id = Id::new();
        let nil_id = Id::new();
        let cons_id = Id::new();
        let a_param_id = Id::new();

        let type0 = Rc::new(Term::Sort(TermSort {
            sort: Sort::Type(0),
        }));
        let var_a = Rc::new(Term::Variable(TermVariable { id: a_param_id }));
        let list_type = Rc::new(Term::Constant(TermConstant { id: list_id }));

        let nil_type = list_type.clone();
        let nil_constr = ConstructorDefinition::new(nil_id, nil_type, 0);

        let cons_type = Rc::new(Term::Product(crate::term::TermProduct {
            var: Id::new(),
            source: var_a.clone(),
            target: Rc::new(Term::Product(crate::term::TermProduct {
                var: Id::new(),
                source: list_type.clone(),
                target: list_type,
            })),
        }));
        let cons_constr = ConstructorDefinition::new(cons_id, cons_type, 2);

        let list_def = InductiveDefinition::new(
            list_id,
            vec![Parameter {
                name: a_param_id,
                ty: type0,
            }],
            Rc::new(Term::Sort(TermSort {
                sort: Sort::Type(0),
            })),
            vec![nil_constr, cons_constr],
        );

        inductive_env.add_inductive(list_def).unwrap();

        let list_def = inductive_env.get_inductive(list_id).unwrap();
        assert_eq!(list_def.name, list_id);
        assert_eq!(list_def.parameters.len(), 1);
    }

    #[test]
    fn test_list_constructor_count() {
        let mut inductive_env = InductiveEnvironment::new();

        let list_id = Id::new();
        let nil_id = Id::new();
        let cons_id = Id::new();
        let a_param_id = Id::new();

        let type0 = Rc::new(Term::Sort(TermSort {
            sort: Sort::Type(0),
        }));
        let var_a = Rc::new(Term::Variable(TermVariable { id: a_param_id }));
        let list_type = Rc::new(Term::Constant(TermConstant { id: list_id }));

        let nil_type = list_type.clone();
        let nil_constr = ConstructorDefinition::new(nil_id, nil_type, 0);

        let cons_type = Rc::new(Term::Product(crate::term::TermProduct {
            var: Id::new(),
            source: var_a.clone(),
            target: Rc::new(Term::Product(crate::term::TermProduct {
                var: Id::new(),
                source: list_type.clone(),
                target: list_type,
            })),
        }));
        let cons_constr = ConstructorDefinition::new(cons_id, cons_type, 2);

        let list_def = InductiveDefinition::new(
            list_id,
            vec![Parameter {
                name: a_param_id,
                ty: type0,
            }],
            Rc::new(Term::Sort(TermSort {
                sort: Sort::Type(0),
            })),
            vec![nil_constr, cons_constr],
        );

        inductive_env.add_inductive(list_def).unwrap();

        let list_def = inductive_env.get_inductive(list_id).unwrap();
        assert_eq!(list_def.constructors.len(), 2);
    }
}
