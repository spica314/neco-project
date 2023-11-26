use std::fmt::Debug;
use std::hash::Hash;

pub trait Decoration {
    type Entrypoint: Debug + Clone + PartialEq + Eq + Hash;
    type ExprApp: Debug + Clone + PartialEq + Eq + Hash;
    type ExprBlock: Debug + Clone + PartialEq + Eq + Hash;
    type ExprIdentWithPath: Debug + Clone + PartialEq + Eq + Hash;
    type ExprIdent: Debug + Clone + PartialEq + Eq + Hash;
    type ExprMatch: Debug + Clone + PartialEq + Eq + Hash;
    type ExprParen: Debug + Clone + PartialEq + Eq + Hash;
    type ExprString: Debug + Clone + PartialEq + Eq + Hash;
    type FormulaForall: Debug + Clone + PartialEq + Eq + Hash;
    type FormulaImplies: Debug + Clone + PartialEq + Eq + Hash;
    type FormulaAtom: Debug + Clone + PartialEq + Eq + Hash;
    type FormulaApp: Debug + Clone + PartialEq + Eq + Hash;
    type FormulaParen: Debug + Clone + PartialEq + Eq + Hash;
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
    type TheoremDef: Debug + Clone + PartialEq + Eq + Hash;
    type TypedArg: Debug + Clone + PartialEq + Eq + Hash;
    type ExprMatchPattern: Debug + Clone + PartialEq + Eq + Hash;
    type StatementLet: Debug + Clone + PartialEq + Eq + Hash;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UD;
impl Decoration for UD {
    type Entrypoint = ();
    type ExprApp = ();
    type ExprBlock = ();
    type ExprIdentWithPath = ();
    type ExprIdent = ();
    type ExprMatch = ();
    type ExprParen = ();
    type ExprString = ();
    type FormulaForall = ();
    type FormulaImplies = ();
    type FormulaAtom = ();
    type FormulaApp = ();
    type FormulaParen = ();
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
    type TheoremDef = ();
    type TypedArg = ();
    type ExprMatchPattern = ();
    type StatementLet = ();
}
