use std::marker::PhantomData;

use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    token::{Token, TokenKeyword, TokenSemicolon},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementBreak<D: Decoration> {
    pub keyword_break: TokenKeyword,
    pub semi: TokenSemicolon,
    pub ext: PhantomData<D>,
}

impl Parse for SynStatementBreak<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_break) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_break.keyword != "#break" {
            return Ok(None);
        }

        let Some(semi) = TokenSemicolon::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementBreak {
            keyword_break,
            semi,
            ext: PhantomData,
        }))
    }
}
