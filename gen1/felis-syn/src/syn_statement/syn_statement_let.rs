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
    pub keyword_mut: Option<TokenKeyword>,
    pub name: TokenIdent,
    pub initial: Option<SynStatementLetInitial<D>>,
    pub semi: TokenSemicolon,
    pub ext: D::StatementLet,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementLetInitial<D: Decoration> {
    pub eq: TokenEq,
    pub expr: SynExpr<D>,
}

impl Parse for SynStatementLetInitial<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(eq) = TokenEq::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementLetInitial { eq, expr }))
    }
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

        let keyword_mut = TokenKeyword::parse(tokens, &mut k)?;
        let keyword_mut =
            if keyword_mut.is_some() && keyword_mut.as_ref().unwrap().keyword.as_str() == "#mut" {
                keyword_mut
            } else {
                None
            };

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let initial = SynStatementLetInitial::parse(tokens, &mut k)?;

        let Some(semi) = TokenSemicolon::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementLet {
            keyword_let,
            keyword_mut,
            name,
            initial,
            semi,
            ext: (),
        }))
    }
}

impl<D: Decoration> ToFelisString for SynStatementLet<D> {
    fn to_felis_string(&self) -> String {
        if let Some(initial) = &self.initial {
            format!(
                "#let {}{} = {};",
                if self.keyword_mut.is_some() {
                    "#mut "
                } else {
                    ""
                },
                self.name.to_felis_string(),
                initial.expr.to_felis_string()
            )
        } else {
            format!(
                "#let {}{};",
                if self.keyword_mut.is_some() {
                    "#mut "
                } else {
                    ""
                },
                self.name.to_felis_string()
            )
        }
    }
}
