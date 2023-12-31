use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_proc::SynProcBlock,
    token::{Token, TokenKeyword},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementLoop<D: Decoration> {
    pub keyword_loop: TokenKeyword,
    pub block: SynProcBlock<D>,
}

impl Parse for SynStatementLoop<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_loop) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_loop.keyword != "#loop" {
            return Ok(None);
        }

        let Some(block) = SynProcBlock::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementLoop {
            keyword_loop,
            block,
        }))
    }
}
