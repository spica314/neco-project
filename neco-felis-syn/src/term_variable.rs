use crate::{Parse, ParseError, token::TokenVariable};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermVariable {
    variable: TokenVariable,
}

impl Parse for TermVariable {
    fn parse(tokens: &[crate::token::Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(token_variable) = TokenVariable::parse(tokens, i)? {
            let term_variable = TermVariable {
                variable: token_variable,
            };

            return Ok(Some(term_variable));
        }

        Ok(None)
    }
}
