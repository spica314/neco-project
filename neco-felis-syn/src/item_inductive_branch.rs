use crate::{
    Parse, ParseError, Term,
    token::{TokenColon, TokenComma, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemInductiveBranch {
    name: TokenVariable,
    colon: TokenColon,
    ty: Box<Term>,
    comma: TokenComma,
}

impl Parse for ItemInductiveBranch {
    fn parse(
        tokens: &[crate::token::Token],
        i: &mut usize,
    ) -> Result<Option<Self>, crate::ParseError> {
        let mut k = *i;

        let Some(name) = TokenVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("item_inductive_branch_1"));
        };

        let Some(ty) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("item_inductive_branch_2"));
        };

        let Some(comma) = TokenComma::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("item_inductive_branch_3"));
        };

        let item_inductive_branch = ItemInductiveBranch {
            name,
            colon,
            ty: Box::new(ty),
            comma,
        };

        *i = k;
        Ok(Some(item_inductive_branch))
    }
}
