use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_fn_def::SynFnDef,
    syn_formula::SynFormula,
    token::{Token, TokenEq, TokenIdent, TokenKeyword, TokenLBrace, TokenRBrace},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTheoremDef<D: Decoration> {
    pub keyword_theorem: TokenKeyword,
    pub name: TokenIdent,
    pub eq: TokenEq,
    pub formula: SynFormula<D>,
    pub lbrace: TokenLBrace,
    pub fn_def: SynFnDef<D>,
    pub rbrace: TokenRBrace,
    pub ext: D::TheoremDef,
}

impl Parse for SynTheoremDef<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_theorem) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_theorem.keyword.as_str() != "#theorem" {
            return Ok(None);
        }

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(eq) = TokenEq::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(formula) = SynFormula::parse(tokens, &mut k)? else {
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
            formula,
            lbrace,
            fn_def,
            rbrace,
            ext: (),
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::Parser;

    use super::*;

    #[test]
    fn felis_syn_theorem_def_parse_test_1() {
        let s = std::fs::read_to_string("../../library/wip/theorem.fe").unwrap();
        let mut parser = Parser::new();
        let res = parser.parse::<SynTheoremDef<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "theorem1");
    }
}
