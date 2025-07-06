use crate::{
    Parse, ParseError, Phase, PhaseParse, Term,
    token::{Token, TokenKeyword, TokenOperator, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermLetMut<P: Phase> {
    pub let_keyword: TokenKeyword,
    pub mut_keyword: TokenKeyword,
    pub variable: TokenVariable,
    pub equals: TokenOperator,
    pub value: Box<Term<P>>,
    pub ext: P::TermLetMutExt,
}

impl Parse for TermLetMut<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse "let" keyword
        let Some(let_keyword) = TokenKeyword::parse_keyword(tokens, &mut k, "let")? else {
            return Ok(None);
        };

        // Parse "mut" keyword
        let Some(mut_keyword) = TokenKeyword::parse_keyword(tokens, &mut k, "mut")? else {
            return Ok(None);
        };

        // Parse variable name
        let Some(variable) = TokenVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Parse "=" operator
        let Some(equals) = TokenOperator::parse_operator(tokens, &mut k, "=")? else {
            return Ok(None);
        };

        // Parse value expression
        let Some(value) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected: value expression after '='"));
        };

        let term_let_mut = TermLetMut {
            let_keyword,
            mut_keyword,
            variable,
            equals,
            value: Box::new(value),
            ext: (),
        };

        *i = k;
        Ok(Some(term_let_mut))
    }
}

impl<P: Phase> TermLetMut<P> {
    pub fn variable_name(&self) -> &str {
        self.variable.s()
    }
}
