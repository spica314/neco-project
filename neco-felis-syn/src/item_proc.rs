use crate::{
    ItemProcBlock, Parse, ParseError, Phase, PhaseParse, Term,
    token::{Token, TokenColon, TokenKeyword, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemProc<P: Phase> {
    pub ptx_modifier: Option<TokenKeyword>,
    pub keyword_proc: TokenKeyword,
    pub name: TokenVariable,
    pub colon: TokenColon,
    pub ty: Box<Term<P>>,
    pub proc_block: ItemProcBlock<P>,
    pub ext: P::ItemProcExt,
}

impl Parse for ItemProc<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Check for optional #ptx modifier
        let ptx_modifier = TokenKeyword::parse_keyword(tokens, &mut k, "ptx")?;

        let Some(keyword_proc) = TokenKeyword::parse_keyword(tokens, &mut k, "proc")? else {
            return Ok(None);
        };

        let Some(name) = TokenVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(ty) = Term::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(proc_block) = ItemProcBlock::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let item_proc = ItemProc {
            ptx_modifier,
            keyword_proc,
            name,
            colon,
            ty: Box::new(ty),
            proc_block,
            ext: (),
        };
        *i = k;
        Ok(Some(item_proc))
    }
}
