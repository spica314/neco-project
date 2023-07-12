use crate::{
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenString},
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprString {
    id: SynTreeId,
    pub token_string: TokenString,
}

impl SynExprString {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprString {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let token_string = if let Some(token_string) = TokenString::parse(tokens, &mut k)? {
            token_string
        } else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprString {
            id: SynTreeId::new(),
            token_string,
        }))
    }
}

impl ToFelisString for SynExprString {
    fn to_felis_string(&self) -> String {
        self.token_string.to_felis_string()
    }
}
