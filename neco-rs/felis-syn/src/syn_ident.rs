use crate::{
    parse::Parse,
    token::{Token, TokenIdent},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynIdent {
    pub ident: TokenIdent,
    pub unique_name: Option<String>,
}

impl Parse for SynIdent {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynIdent {
            ident,
            unique_name: None,
        }))
    }
}
