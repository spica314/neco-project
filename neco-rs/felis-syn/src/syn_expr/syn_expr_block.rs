use crate::{
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenLBrace, TokenRBrace},
    SynTreeId,
};

use super::SynExpr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprBlock {
    id: SynTreeId,
    pub lbrace: TokenLBrace,
    pub expr: Box<SynExpr>,
    pub rbrace: TokenRBrace,
}

impl SynExprBlock {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprBlock {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynExprBlock {
            id: Default::default(),
            lbrace,
            expr: Box::new(expr),
            rbrace,
        }))
    }
}

impl ToFelisString for SynExprBlock {
    fn to_felis_string(&self) -> String {
        format!("{{ {} }}", self.expr.to_felis_string())
    }
}
