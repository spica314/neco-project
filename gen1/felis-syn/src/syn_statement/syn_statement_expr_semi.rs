use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_expr::SynExpr,
    to_felis_string::ToFelisString,
    token::{Token, TokenSemicolon},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementExprSemi<D: Decoration> {
    pub expr: SynExpr<D>,
    pub semi: TokenSemicolon,
}

impl Parse for SynStatementExprSemi<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(semi) = TokenSemicolon::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynStatementExprSemi { expr, semi }))
    }
}

impl<D: Decoration> ToFelisString for SynStatementExprSemi<D> {
    fn to_felis_string(&self) -> String {
        let mut res = String::new();
        res.push_str(&self.expr.to_felis_string());
        res.push(';');
        res
    }
}
