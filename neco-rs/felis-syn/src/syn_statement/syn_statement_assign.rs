use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_expr::SynExpr,
    token::{Token, TokenEq, TokenSemicolon},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementAssign<D: Decoration> {
    pub lhs: SynExpr<D>,
    pub eq: TokenEq,
    pub rhs: SynExpr<D>,
    pub semi: TokenSemicolon,
}

impl Parse for SynStatementAssign<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lhs) = SynExpr::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(eq) = TokenEq::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(rhs) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(semi) = TokenSemicolon::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementAssign { lhs, eq, rhs, semi }))
    }
}
