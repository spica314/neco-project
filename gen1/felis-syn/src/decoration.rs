use std::fmt::Debug;
use std::hash::Hash;

pub trait Decoration: Debug {
    type Entrypoint: Debug + Clone + PartialEq + Eq + Hash;
    type ExprApp: Debug + Clone + PartialEq + Eq + Hash;
    type ExprIdentWithPath: Debug + Clone + PartialEq + Eq + Hash;
    type ExprNumber: Debug + Clone + PartialEq + Eq + Hash;
    type ExprMatch: Debug + Clone + PartialEq + Eq + Hash;
    type ExprParen: Debug + Clone + PartialEq + Eq + Hash;
    type ExprString: Debug + Clone + PartialEq + Eq + Hash;
    type Variant: Debug + Clone + PartialEq + Eq + Hash;
    type TypeDef: Debug + Clone + PartialEq + Eq + Hash;
    type TypeApp: Debug + Clone + PartialEq + Eq + Hash;
    type TypeAtom: Debug + Clone + PartialEq + Eq + Hash;
    type TypeMap: Debug + Clone + PartialEq + Eq + Hash;
    type TypeParen: Debug + Clone + PartialEq + Eq + Hash;
    type TypeDependentMap: Debug + Clone + PartialEq + Eq + Hash;
    type TypeUnit: Debug + Clone + PartialEq + Eq + Hash;
    type File: Debug + Clone + PartialEq + Eq + Hash;
    type FnDef: Debug + Clone + PartialEq + Eq + Hash;
    type ProcDef: Debug + Clone + PartialEq + Eq + Hash;
    type TypedArg: Debug + Clone + PartialEq + Eq + Hash;
    type ExprMatchPattern: Debug + Clone + PartialEq + Eq + Hash;
    type StatementLet: Debug + Clone + PartialEq + Eq + Hash;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UD;
impl Decoration for UD {
    type Entrypoint = ();
    type ExprApp = ();
    type ExprIdentWithPath = ();
    type ExprNumber = ();
    type ExprMatch = ();
    type ExprParen = ();
    type ExprString = ();
    type Variant = ();
    type TypeDef = ();
    type TypeApp = ();
    type TypeAtom = ();
    type TypeMap = ();
    type TypeParen = ();
    type TypeDependentMap = ();
    type TypeUnit = ();
    type File = ();
    type FnDef = ();
    type ProcDef = ();
    type TypedArg = ();
    type ExprMatchPattern = ();
    type StatementLet = ();
}
