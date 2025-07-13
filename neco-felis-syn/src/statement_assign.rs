use crate::{
    Parse, ParseError, Phase, PhaseParse, Term,
    token::{Token, TokenOperator, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatementAssign<P: Phase> {
    pub variable: TokenVariable,
    pub equals: TokenOperator,
    pub value: Box<Term<P>>,
    pub ext: P::StatementAssignExt,
}

impl Parse for StatementAssign<PhaseParse> {
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

        let statement_assign = StatementAssign {
            variable,
            equals,
            value: Box::new(value),
            ext: (),
        };

        *i = k;
        Ok(Some(statement_assign))
    }
}

impl<P: Phase> StatementAssign<P> {
    pub fn variable_name(&self) -> &str {
        self.variable.s()
    }
}
