use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_statement::SynStatement,
    syn_type::SynType,
    token::{Token, TokenColon, TokenIdent, TokenKeyword, TokenLBrace, TokenRBrace},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynProcDef<D: Decoration> {
    pub keyword_proc: TokenKeyword,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType<D>,
    pub proc_block: SynProcBlock<D>,
    pub ext: D::ProcDef,
}

impl Parse for SynProcDef<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_proc) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_proc.keyword != "#proc" {
            return Ok(None);
        };

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(proc_block) = SynProcBlock::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynProcDef {
            keyword_proc,
            name,
            colon,
            ty,
            proc_block,
            ext: (),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynProcBlock<D: Decoration> {
    pub lbrace: TokenLBrace,
    pub statements: Vec<SynStatement<D>>,
    pub rbrace: TokenRBrace,
}

impl Parse for SynProcBlock<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let mut statements = vec![];
        while let Some(statement) = SynStatement::parse(tokens, &mut k)? {
            statements.push(statement);
        }

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynProcBlock {
            lbrace,
            statements,
            rbrace,
        }))
    }
}
