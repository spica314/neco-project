use crate::{
    Parse, ParseError, Term, TermParen, TermVariable,
    token::{Token, TokenOperator},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermArrowNodep {
    from: Box<Term>,
    arrow: TokenOperator,
    to: Box<Term>,
}

impl TermArrowNodep {
    pub fn from(&self) -> &Term {
        &self.from
    }

    pub fn to(&self) -> &Term {
        &self.to
    }
}

impl Parse for TermArrowNodep {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        let Some(from) = TermForArrowNodepFrom::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(arrow) = TokenOperator::parse_operator(tokens, &mut k, "->")? else {
            // ?
            return Ok(None);
        };

        let Some(to) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("term_arrow_no_dep_1"));
        };

        let term_arrow_nodep = TermArrowNodep {
            from: Box::new(from.into()),
            arrow,
            to: Box::new(to),
        };

        *i = k;
        Ok(Some(term_arrow_nodep))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TermForArrowNodepFrom {
    Paren(TermParen),
    Variable(TermVariable),
}

impl From<TermForArrowNodepFrom> for Term {
    fn from(value: TermForArrowNodepFrom) -> Self {
        match value {
            TermForArrowNodepFrom::Paren(term_paren) => Term::Paren(term_paren),
            TermForArrowNodepFrom::Variable(term_variable) => Term::Variable(term_variable),
        }
    }
}

impl Parse for TermForArrowNodepFrom {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(term_paren) = TermParen::parse(tokens, i)? {
            return Ok(Some(TermForArrowNodepFrom::Paren(term_paren)));
        }

        if let Some(term_variable) = TermVariable::parse(tokens, i)? {
            return Ok(Some(TermForArrowNodepFrom::Variable(term_variable)));
        }

        Err(ParseError::Unknown("term_for_arrow_nodep_from_1"))
    }
}
