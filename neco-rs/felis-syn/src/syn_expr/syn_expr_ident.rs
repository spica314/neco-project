use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    token::{Token, TokenIdent},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprIdent<D: Decoration> {
    pub ident: TokenIdent,
    pub ext: D::ExprIdent,
}

impl Parse for SynExprIdent<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprIdent { ident, ext: () }))
    }
}
