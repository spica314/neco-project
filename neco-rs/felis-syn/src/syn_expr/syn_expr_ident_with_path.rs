use crate::{
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenColonColon, TokenIdent},
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprIdentWithPath {
    id: SynTreeId,
    pub path: Vec<(TokenIdent, TokenColonColon)>,
    pub ident: TokenIdent,
}

impl SynExprIdentWithPath {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl ToFelisString for SynExprIdentWithPath {
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

impl Parse for SynExprIdentWithPath {
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
            id: SynTreeId::new(),
            path,
            ident,
        }))
    }
}
