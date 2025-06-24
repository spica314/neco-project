use crate::{
    Parse, ParseError, TermApply, TermArrowDep, TermArrowNodep, TermMatch, TermParen, TermVariable,
    token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Paren(TermParen),
    ArrowNodep(TermArrowNodep),
    ArrowDep(TermArrowDep),
    Apply(TermApply),
    Variable(TermVariable),
    Match(TermMatch),
}

impl Parse for Term {
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

        Ok(None)
    }
}
