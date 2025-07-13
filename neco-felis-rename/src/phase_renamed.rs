use neco_felis_syn::Phase;

use crate::VariableId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhaseRenamed();

impl Phase for PhaseRenamed {
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
    type TermVariableExt = VariableId;
    type TermStringExt = ();
    type StatementsExt = ();
    type StatementsThenExt = ();
    type ItemProcBlockExt = ();
    type ItemProcExt = ();
    type TermUnitExt = ();
    type TermNumberExt = ();
    type TermLetExt = ();
    type TermLetMutExt = ();
    type TermAssignExt = ();
    type ItemArrayExt = ();
    type ItemStructExt = ();
    type TermFieldAccessExt = ();
    type TermConstructorCallExt = ();
    type TermFieldAssignExt = ();
    type TermIfExt = ();
    type StatementExt = ();
    type StatementAssignExt = ();
    type StatementFieldAssignExt = ();
    type StatementLetExt = ();
    type StatementLetMutExt = ();
}
