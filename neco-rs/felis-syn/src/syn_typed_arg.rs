use crate::{
    parse::Parse,
    syn_type::SynType,
    to_felis_string::ToFelisString,
    token::{Token, TokenColon, TokenIdent, TokenLParen, TokenRParen},
};

#[derive(Debug, Clone)]
pub struct SynTypedArg {
    pub lparen: TokenLParen,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType,
    pub rparen: TokenRParen,
}

impl ToFelisString for SynTypedArg {
    fn to_felis_string(&self) -> String {
        let mut s = String::new();
        s.push('(');
        s.push_str(self.name.as_str());
        s.push_str(" : ");
        s.push_str(&self.ty.to_felis_string());
        s.push(')');
        s
    }
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

        *i = k;
        Ok(Some(SynTypedArg {
            lparen,
            name,
            colon,
            ty,
            rparen,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::{lex, FileId};

    #[test]
    fn felis_syn_typed_arg_parse_test_1() {
        let s = "(A : Prop)";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs).unwrap();
        let mut i = 0;
        let res = SynTypedArg::parse(&tokens, &mut i);
        assert_eq!(i, tokens.len());
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(A : Prop)");
    }
}
