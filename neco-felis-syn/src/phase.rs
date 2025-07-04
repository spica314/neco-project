pub trait Phase {
    type FileExt: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash;
    type ItemDefinitionExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type ItemInductiveExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type ItemInductiveBranchExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type ItemTheoremExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type ItemEntrypointExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type ItemBuiltinExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type TermApplyExt: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash;
    type TermArrowDepExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type TermArrowNodepExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type TermMatchExt: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash;
    type TermMatchBranchExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type TermParenExt: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash;
    type TermVariableExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type TermStringExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type StatementsExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type StatementsThenExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type ItemProcBlockExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type ItemProcExt: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash;
    type TermUnitExt: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash;
    type TermNumberExt: std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash;
    type TermLetExt: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhaseParse();

impl Phase for PhaseParse {
    type FileExt = ();
    type ItemDefinitionExt = ();
    type ItemInductiveExt = ();
    type ItemInductiveBranchExt = ();
    type ItemTheoremExt = ();
    type ItemEntrypointExt = ();
    type ItemBuiltinExt = ();
    type TermApplyExt = ();
    type TermArrowDepExt = ();
    type TermArrowNodepExt = ();
    type TermMatchExt = ();
    type TermMatchBranchExt = ();
    type TermParenExt = ();
    type TermVariableExt = ();
    type TermStringExt = ();
    type StatementsExt = ();
    type StatementsThenExt = ();
    type ItemProcBlockExt = ();
    type ItemProcExt = ();
    type TermUnitExt = ();
    type TermNumberExt = ();
    type TermLetExt = ();
}
