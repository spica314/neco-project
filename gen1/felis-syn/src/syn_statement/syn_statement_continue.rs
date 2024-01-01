use std::marker::PhantomData;

use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    token::{Token, TokenKeyword, TokenSemicolon},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementContinue<D: Decoration> {
    pub keyword_continue: TokenKeyword,
    pub semi: TokenSemicolon,
    pub ext: PhantomData<D>,
}

impl Parse for SynStatementContinue<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_continue) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_continue.keyword != "#continue" {
            return Ok(None);
        }

        let Some(semi) = TokenSemicolon::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementContinue {
            keyword_continue,
            semi,
            ext: PhantomData,
        }))
    }
}
