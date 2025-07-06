use crate::{
    Parse, ParseError, Phase, PhaseParse, Term,
    token::{Token, TokenOperator, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermAssign<P: Phase> {
    pub variable: TokenVariable,
    pub equals: TokenOperator,
    pub value: Box<Term<P>>,
    pub ext: P::TermAssignExt,
}

impl Parse for TermAssign<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

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

        let term_assign = TermAssign {
            variable,
            equals,
            value: Box::new(value),
            ext: (),
        };

        *i = k;
        Ok(Some(term_assign))
    }
}

impl<P: Phase> TermAssign<P> {
    pub fn variable_name(&self) -> &str {
        self.variable.s()
    }
}
