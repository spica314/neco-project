use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenColonColon, TokenIdent},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprIdentWithPath<D: Decoration> {
    pub path: Vec<(TokenIdent, TokenColonColon)>,
    pub ident: TokenIdent,
    pub ext: D::ExprIdentWithPath,
}

impl<D: Decoration> ToFelisString for SynExprIdentWithPath<D> {
    fn to_felis_string(&self) -> String {
        let mut res = String::new();
        for t in &self.path {
            res.push_str(t.0.as_str());
            res.push_str("::");
        }
        res.push_str(self.ident.as_str());
        res
    }
}

impl Parse for SynExprIdentWithPath<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut path = vec![];
        let Some(mut ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        while let Some(colon_colon) = TokenColonColon::parse(tokens, &mut k)? {
            path.push((ident.clone(), colon_colon));
            let Some(ident2) = TokenIdent::parse(tokens, &mut k)? else {
                return Err(());
            };
            ident = ident2;
        }

        *i = k;
        Ok(Some(SynExprIdentWithPath {
            path,
            ident,
            ext: (),
        }))
    }
}
