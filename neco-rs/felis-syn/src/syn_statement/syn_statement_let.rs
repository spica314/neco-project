use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_expr::SynExpr,
    to_felis_string::ToFelisString,
    token::{Token, TokenEq, TokenIdent, TokenKeyword, TokenSemicolon},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementLet<D: Decoration> {
    pub keyword_let: TokenKeyword,
    pub name: TokenIdent,
    pub eq: TokenEq,
    pub expr: SynExpr<D>,
    pub semi: TokenSemicolon,
    pub ext: D::StatementLet,
}

impl Parse for SynStatementLet<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_let) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_let.keyword.as_str() != "#let" {
            return Ok(None);
        }

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(eq) = TokenEq::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(semi) = TokenSemicolon::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementLet {
            keyword_let,
            name,
            eq,
            expr,
            semi,
            ext: (),
        }))
    }
}

impl<D: Decoration> ToFelisString for SynStatementLet<D> {
    fn to_felis_string(&self) -> String {
        format!(
            "#let {} = {};",
            self.name.to_felis_string(),
            self.expr.to_felis_string()
        )
    }
}
