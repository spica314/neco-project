use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenString},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprString<D: Decoration> {
    pub token_string: TokenString,
    pub ext: D::ExprString,
}

impl Parse for SynExprString<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let token_string = if let Some(token_string) = TokenString::parse(tokens, &mut k)? {
            token_string
        } else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprString {
            token_string,
            ext: (),
        }))
    }
}

impl<D: Decoration> ToFelisString for SynExprString<D> {
    fn to_felis_string(&self) -> String {
        self.token_string.to_felis_string()
    }
}
