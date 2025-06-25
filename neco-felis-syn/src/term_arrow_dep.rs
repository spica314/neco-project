use crate::{
    Parse, ParseError, Phase, PhaseParse, Term, TermVariable,
    token::{Token, TokenColon, TokenOperator, TokenParenL, TokenParenR},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermArrowDep<P: Phase> {
    paren_l: TokenParenL,
    from: TermVariable<P>,
    colon: TokenColon,
    from_ty: Box<Term<P>>,
    paren_r: TokenParenR,
    arrow: TokenOperator,
    to: Box<Term<P>>,
    #[allow(dead_code)]
    ext: P::TermArrowDepExt,
}

impl<P: Phase> TermArrowDep<P> {
    pub fn from(&self) -> &TermVariable<P> {
        &self.from
    }

    pub fn from_ty(&self) -> &Term<P> {
        &self.from_ty
    }

    pub fn to(&self) -> &Term<P> {
        &self.to
    }
}

impl Parse for TermArrowDep<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        let Some(paren_l) = TokenParenL::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(from) = TermVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(from_ty) = Term::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(paren_r) = TokenParenR::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(arrow) = TokenOperator::parse_operator(tokens, &mut k, "->")? else {
            return Ok(None);
        };

        let Some(to) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("term_arrow_dep_1"));
        };

        let term_arrow_dep = TermArrowDep {
            paren_l,
            from,
            colon,
            from_ty: Box::new(from_ty),
            paren_r,
            arrow,
            to: Box::new(to),
            ext: (),
        };

        *i = k;
        Ok(Some(term_arrow_dep))
    }
}
