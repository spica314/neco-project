use crate::{
    Parse, ParseError, Phase, PhaseParse, ProcTerm, ProcTermFieldAccess, ProcTermNumber,
    ProcTermVariable,
    token::{Token, TokenOperator},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatementFieldAssign<P: Phase> {
    pub field_access: ProcTermFieldAccess<P>,
    pub equals: TokenOperator,
    pub value: Box<ProcTerm<P>>,
    pub ext: P::StatementFieldAssignExt,
}

impl Parse for StatementFieldAssign<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse field access (e.g., "points.x 0")
        let Some(field_access) = ProcTermFieldAccess::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Parse "=" operator
        let Some(equals) = TokenOperator::parse_operator(tokens, &mut k, "=")? else {
            return Ok(None);
        };

        // Parse value expression (simple terms only to avoid infinite recursion)
        let value = if let Some(number) = ProcTermNumber::parse(tokens, &mut k)? {
            ProcTerm::Number(number)
        } else if let Some(variable) = ProcTermVariable::parse(tokens, &mut k)? {
            ProcTerm::Variable(variable)
        } else {
            return Err(ParseError::Unknown("expected value expression after '='"));
        };

        let statement_field_assign = StatementFieldAssign {
            field_access,
            equals,
            value: Box::new(value),
            ext: (),
        };

        *i = k;
        Ok(Some(statement_field_assign))
    }
}
