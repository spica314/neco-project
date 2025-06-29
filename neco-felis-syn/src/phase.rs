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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhaseParse();

impl Phase for PhaseParse {
    type FileExt = ();
    type ItemDefinitionExt = ();
    type ItemInductiveExt = ();
    type ItemInductiveBranchExt = ();
    type ItemTheoremExt = ();
    type TermApplyExt = ();
    type TermArrowDepExt = ();
    type TermArrowNodepExt = ();
    type TermMatchExt = ();
    type TermMatchBranchExt = ();
    type TermParenExt = ();
    type TermVariableExt = ();
}
