use crate::{
    parse::Parse,
    syn_fn_def::SynFnDef,
    syn_ident::SynIdent,
    syn_type::SynType,
    token::{Token, TokenEq, TokenKeyword, TokenLBrace, TokenRBrace},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynTheoremDef {
    pub keyword_theorem: TokenKeyword,
    pub name: SynIdent,
    pub eq: TokenEq,
    pub ty: SynType,
    pub lbrace: TokenLBrace,
    pub fn_def: SynFnDef,
    pub rbrace: TokenRBrace,
}

impl Parse for SynTheoremDef {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_theorem) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_theorem.keyword.as_str() != "#theorem" {
            return Ok(None);
        }

        let Some(name) = SynIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(eq) = TokenEq::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(fn_def) = SynFnDef::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynTheoremDef {
            keyword_theorem,
            name,
            eq,
            ty,
            lbrace,
            fn_def,
            rbrace,
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::token::{lex, FileId};

    use super::*;

    #[test]
    fn felis_syn_theorem_def_parse_test_1() {
        let s = std::fs::read_to_string("../../library/wip/theorem.fe").unwrap();
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynTheoremDef::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "theorem1");
    }
}
