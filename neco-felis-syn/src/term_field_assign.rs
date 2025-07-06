use crate::{
    Parse, ParseError, Phase, PhaseParse, Term, TermFieldAccess,
    token::{Token, TokenOperator},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermFieldAssign<P: Phase> {
    pub field_access: TermFieldAccess<P>,
    pub equals: TokenOperator,
    pub value: Box<Term<P>>,
    pub ext: P::TermFieldAssignExt,
}

impl Parse for TermFieldAssign<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse field access (e.g., "points.x 0")
        let Some(field_access) = TermFieldAccess::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Parse "=" operator
        let Some(equals) = TokenOperator::parse_operator(tokens, &mut k, "=")? else {
            return Ok(None);
        };

        // Parse value expression (simple terms only to avoid infinite recursion)
        let value = if let Some(number) = crate::TermNumber::parse(tokens, &mut k)? {
            Term::Number(number)
        } else if let Some(variable) = crate::TermVariable::parse(tokens, &mut k)? {
            Term::Variable(variable)
        } else {
            return Err(ParseError::Unknown("expected value expression after '='"));
        };

        let term_field_assign = TermFieldAssign {
            field_access,
            equals,
            value: Box::new(value),
            ext: (),
        };

        *i = k;
        Ok(Some(term_field_assign))
    }
}
