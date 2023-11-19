use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    token::{Token, TokenLParen, TokenRParen},
    SynTreeId,
};

use super::SynExpr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprParen<D: Decoration> {
    id: SynTreeId,
    pub lparen: TokenLParen,
    pub expr: Box<SynExpr<D>>,
    pub rparen: TokenRParen,
    ext: D::ExprParen,
}

impl<D: Decoration> SynExprParen<D> {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprParen<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(rparen) = TokenRParen::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynExprParen {
            id: Default::default(),
            lparen,
            expr: Box::new(expr),
            rparen,
            ext: (),
        }))
    }
}
