use crate::{
    ParseError, Phase, PhaseParse, ProcTerm,
    token::{Token, TokenOperator},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcTermDereference<P: Phase> {
    pub term: Box<ProcTerm<P>>,
    pub dot_star: TokenOperator,
    pub ext: P::ProcTermDereferenceExt,
}

impl ProcTermDereference<PhaseParse> {
    // Helper method to try parsing dereference as a postfix operation
    pub fn try_parse_postfix(
        base_term: ProcTerm<PhaseParse>,
        tokens: &[Token],
        i: &mut usize,
    ) -> Result<Option<ProcTerm<PhaseParse>>, ParseError> {
        // Try to parse ".*" operator after base term
        if let Some(dot_star) = TokenOperator::parse_operator(tokens, i, ".*")? {
            let proc_term_dereference = ProcTermDereference {
                term: Box::new(base_term),
                dot_star,
                ext: (),
            };
            Ok(Some(ProcTerm::Dereference(proc_term_dereference)))
        } else {
            Ok(None)
        }
    }
}
