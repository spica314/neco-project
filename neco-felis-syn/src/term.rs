use crate::{
    Parse, ParseError, Phase, PhaseParse, TermApply, TermArrowDep, TermArrowNodep, TermMatch,
    TermParen, TermVariable, token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term<P: Phase> {
    Paren(TermParen<P>),
    ArrowNodep(TermArrowNodep<P>),
    ArrowDep(TermArrowDep<P>),
    Apply(TermApply<P>),
    Variable(TermVariable<P>),
    Match(TermMatch<P>),
}

impl Parse for Term<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(term_match) = TermMatch::parse(tokens, i)? {
            return Ok(Some(Term::Match(term_match)));
        }

        if let Some(term_arrow_dep) = TermArrowDep::parse(tokens, i)? {
            return Ok(Some(Term::ArrowDep(term_arrow_dep)));
        }

        if let Some(term_arrow_nodep) = TermArrowNodep::parse(tokens, i)? {
            return Ok(Some(Term::ArrowNodep(term_arrow_nodep)));
        }

        if let Some(term_apply) = TermApply::parse(tokens, i)? {
            return Ok(Some(Term::Apply(term_apply)));
        }

        if let Some(term_variable) = TermVariable::parse(tokens, i)? {
            return Ok(Some(Term::Variable(term_variable)));
        }

        if let Some(term_paren) = TermParen::parse(tokens, i)? {
            return Ok(Some(Term::Paren(term_paren)));
        }

        Ok(None)
    }
}
