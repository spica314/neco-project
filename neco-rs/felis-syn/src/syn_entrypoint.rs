use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    token::{Token, TokenIdent, TokenKeyword},
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynEntrypoint<D: Decoration> {
    pub id: SynTreeId,
    pub token_entrypoint: TokenKeyword,
    pub ident: TokenIdent,
    pub ext: D::Entrypoint,
}

impl Parse for SynEntrypoint<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(token_entrypoint) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if token_entrypoint.keyword != "#entrypoint" {
            return Ok(None);
        };

        let ident = if let Some(ident) = TokenIdent::parse(tokens, &mut k)? {
            ident
        } else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynEntrypoint {
            id: SynTreeId::new(),
            token_entrypoint,
            ident,
            ext: (),
        }))
    }
}
