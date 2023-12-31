use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenNumber},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprNumber<D: Decoration> {
    pub number: TokenNumber,
    pub ext: D::ExprNumber,
}

impl<D: Decoration> ToFelisString for SynExprNumber<D> {
    fn to_felis_string(&self) -> String {
        self.number.as_str().to_string()
    }
}

impl Parse for SynExprNumber<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(number) = TokenNumber::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprNumber { number, ext: () }))
    }
}
