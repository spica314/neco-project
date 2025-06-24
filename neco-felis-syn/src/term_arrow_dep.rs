use crate::{
    Parse, ParseError, Term, TermVariable,
    token::{Token, TokenColon, TokenOperator, TokenParenL, TokenParenR},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermArrowDep {
    paren_l: TokenParenL,
    from: TermVariable,
    colon: TokenColon,
    from_ty: Box<Term>,
    paren_r: TokenParenR,
    arrow: TokenOperator,
    to: Box<Term>,
}

impl Parse for TermArrowDep {
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
        };

        *i = k;
        Ok(Some(term_arrow_dep))
    }
}
