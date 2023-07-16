use crate::{
    parse::Parse,
    syn_expr::SynExpr,
    token::{Token, TokenEq, TokenIdent, TokenKeyword, TokenSemicolon},
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementLet {
    syn_tree_id: SynTreeId,
    pub keyword_let: TokenKeyword,
    pub name: TokenIdent,
    pub eq: TokenEq,
    pub expr: SynExpr,
    pub semi: TokenSemicolon,
}

impl SynStatementLet {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.syn_tree_id
    }
}

impl Parse for SynStatementLet {
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
            syn_tree_id: Default::default(),
            keyword_let,
            name,
            eq,
            expr,
            semi,
        }))
    }
}
