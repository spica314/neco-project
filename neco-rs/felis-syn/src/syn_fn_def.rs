use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_statement::SynStatement,
    syn_type::SynType,
    token::{Token, TokenColon, TokenIdent, TokenKeyword, TokenLBrace, TokenRBrace},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFnDef<D: Decoration> {
    pub keyword_fn: TokenKeyword,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType<D>,
    pub fn_block: SynFnBlock<D>,
    pub ext: D::FnDef,
}

impl Parse for SynFnDef<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_fn) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_fn.keyword != "#fn" {
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

        let Some(fn_block) = SynFnBlock::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynFnDef {
            keyword_fn,
            name,
            colon,
            ty,
            fn_block,
            ext: (),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFnBlock<D: Decoration> {
    pub lbrace: TokenLBrace,
    pub statements: Vec<SynStatement<D>>,
    pub rbrace: TokenRBrace,
}

impl Parse for SynFnBlock<UD> {
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
        Ok(Some(SynFnBlock {
            lbrace,
            statements,
            rbrace,
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{to_felis_string::ToFelisString, Parser};

    use super::*;

    #[test]
    fn felis_syn_fn_def_parse_test_1() {
        let s = "#fn f : T { }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef<UD>>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "f");
        assert_eq!(res.ty.to_felis_string(), "T");
        assert_eq!(res.fn_block.statements.len(), 0);
    }

    #[test]
    fn felis_syn_fn_def_parse_test_2() {
        let s = "#fn f : T { x }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef<UD>>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "f");
        assert_eq!(res.ty.to_felis_string(), "T");
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }

    #[test]
    fn felis_syn_fn_def_parse_test_3() {
        let s = "#fn f : (x : T) -> T { x }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef<UD>>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "f");
        assert_eq!(res.ty.to_felis_string(), "(x : T) -> T");
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }

    #[test]
    fn felis_syn_fn_def_parse_test_4() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let mut parser = Parser::new();
        let res = parser.parse::<SynFnDef<UD>>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "proof");
        assert_eq!(
            res.ty.to_felis_string(),
            "(A : Prop) -> (B : Prop) -> (x : And A B) -> Or A B"
        );
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }
}
