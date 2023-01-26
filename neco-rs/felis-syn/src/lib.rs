use token::*;

pub mod token;

#[derive(Debug, Clone)]
pub struct SynTypeDef {
    pub keyword_type: TokenKeyword,
    pub name: TokenIdent,
    pub args: Vec<SynTypedArg>,
    pub colon: TokenColon,
    pub ty_ty: Box<SynType>,
    pub lbrace: TokenLBrace,
    pub variants: Vec<SynVariant>,
    pub rbrace: TokenRBrace,
}

#[derive(Debug, Clone)]
pub struct SynTypedArg {
    pub lparen: TokenLParen,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType,
    pub rparen: TokenRParen,
}

impl Parse for SynTypedArg {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
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

        let Some(rparen) = TokenRParen::parse(tokens, &mut k)? else {
            return Err(());
        };

        Ok(Some(SynTypedArg {
            lparen,
            name,
            colon,
            ty,
            rparen,
        }))
    }
}

#[derive(Debug, Clone)]
pub enum SynType {
    App(SynTypeApp),
    Atom(SynTypeAtom),
}

impl Parse for SynType {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct SynTypeApp {
    pub left: Box<SynType>,
    pub right: Box<SynType>,
}

impl Parse for SynTypeApp {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct SynTypeAtom {
    pub ident: TokenIdent,
}

#[derive(Debug, Clone)]
pub struct SynVariant {
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType,
    pub camma: TokenCamma,
}

impl Parse for SynVariant {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

pub trait Parse: Sized {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()>;
}

impl Parse for SynTypeDef {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;
        let Some(keyword_type) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_type.keyword != "#type" {
            return Ok(None);
        }
        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let mut args = vec![];
        while let Some(arg) = SynTypedArg::parse(tokens, &mut k)? {
            args.push(arg);
        }

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(ty_ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        let mut variants = vec![];
        while let Some(variant) = SynVariant::parse(tokens, &mut k)? {
            variants.push(variant);
        }

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };
        Err(())
    }
}
