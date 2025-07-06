use crate::{
    Parse, ParseError, Phase, PhaseParse, Term,
    token::{Token, TokenOperator, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermFieldAccess<P: Phase> {
    pub object: TokenVariable,
    pub dot: TokenOperator,
    pub field: TokenVariable,
    pub index: Option<Box<Term<P>>>,
    pub ext: P::TermFieldAccessExt,
}

impl<P: Phase> TermFieldAccess<P> {
    pub fn object_name(&self) -> &str {
        self.object.s()
    }

    pub fn field_name(&self) -> &str {
        self.field.s()
    }
}

impl Parse for TermFieldAccess<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse object (variable)
        let Some(object) = TokenVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Parse "." operator
        let Some(dot) = TokenOperator::parse_operator(tokens, &mut k, ".")? else {
            return Ok(None);
        };

        // Parse field name
        let Some(field) = TokenVariable::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected field name after '.'"));
        };

        // Try to parse an optional index (e.g., "0" in "points.x 0")
        // Only support simple terms like numbers to avoid infinite recursion
        let index = if let Some(number) = crate::TermNumber::parse(tokens, &mut k)? {
            Some(Box::new(Term::Number(number)))
        } else {
            crate::TermVariable::parse(tokens, &mut k)?
                .map(|variable| Box::new(Term::Variable(variable)))
        };

        let term_field_access = TermFieldAccess {
            object,
            dot,
            field,
            index,
            ext: (),
        };

        *i = k;
        Ok(Some(term_field_access))
    }
}
