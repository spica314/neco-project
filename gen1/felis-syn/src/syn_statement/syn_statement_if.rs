use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_expr::SynExpr,
    syn_proc::SynProcBlock,
    token::{Token, TokenKeyword},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynStatementIf<D: Decoration> {
    pub keyword_if: TokenKeyword,
    pub cond: SynExpr<D>,
    pub t_branch: SynProcBlock<D>,
    pub keyword_else: TokenKeyword,
    pub f_branch: SynProcBlock<D>,
}

impl Parse for SynStatementIf<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_if) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_if.keyword != "#if" {
            return Ok(None);
        }

        let Some(cond) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(t_branch) = SynProcBlock::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(keyword_else) = TokenKeyword::parse(tokens, &mut k)? else {
            return Err(());
        };
        if keyword_else.keyword != "#else" {
            return Err(());
        }

        let Some(f_branch) = SynProcBlock::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynStatementIf {
            keyword_if,
            cond,
            t_branch,
            keyword_else,
            f_branch,
        }))
    }
}
