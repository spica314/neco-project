use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    token::{Token, TokenIdent},
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprIdent<D: Decoration> {
    pub id: SynTreeId,
    pub ident: TokenIdent,
    pub ext: D::ExprIdent,
}

impl<D: Decoration> SynExprIdent<D> {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprIdent<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprIdent {
            id: Default::default(),
            ident,
            ext: (),
        }))
    }
}
