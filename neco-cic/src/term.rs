use std::rc::Rc;

use crate::id::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sort {
    Set,
    Prop,
    Type(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermSort {
    pub sort: Sort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermVariable {
    pub id: Id,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermConstant {
    pub id: Id,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermProduct {
    pub var: Id,
    pub source: Rc<Term>,
    pub target: Rc<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermLambda {
    pub var: Id,
    pub source_ty: Rc<Term>,
    pub target: Rc<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermApplication {
    pub f: Rc<Term>,
    pub args: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermLetIn {
    pub var: Id,
    pub term: Rc<Term>,
    pub ty: Rc<Term>,
    pub body: Rc<Term>,
}

/// Constructor of an inductive type
/// Example: For nat, we have constructors O : nat and S : nat -> nat
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermConstructor {
    pub constructor_id: Id,
    pub inductive_id: Id,
    pub args: Vec<Term>,
}

/// Match analysis (pattern matching) on inductive types
/// Example: match n with | O => ... | S p => ...
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermMatch {
    pub scrutinee: Rc<Term>,
    pub return_type: Rc<Term>,
    pub branches: Vec<MatchBranch>,
}

/// A branch in a match expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchBranch {
    pub constructor_id: Id,
    pub bound_vars: Vec<Id>,
    pub body: Rc<Term>,
}

/// Fix point for recursive definitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermFix {
    pub fix_var: Id,
    pub body_type: Rc<Term>,
    pub body: Rc<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Sort(TermSort),
    Variable(TermVariable),
    Constant(TermConstant),
    Product(TermProduct),
    Lambda(TermLambda),
    Application(TermApplication),
    LetIn(TermLetIn),
    Constructor(TermConstructor),
    Match(TermMatch),
    Fix(TermFix),
}
