use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_type::SynType,
    to_felis_string::ToFelisString,
    token::{Token, TokenColon, TokenIdent, TokenLParen, TokenRParen},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypedArg<D: Decoration> {
    pub lparen: TokenLParen,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType<D>,
    pub rparen: TokenRParen,
}

impl<D: Decoration> ToFelisString for SynTypedArg<D> {
    fn to_felis_string(&self) -> String {
        let mut s = String::new();
        s.push('(');
        s.push_str(self.name.ident.as_str());
        s.push_str(" : ");
        s.push_str(&self.ty.to_felis_string());
        s.push(')');
        s
    }
}

impl Parse for SynTypedArg<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Ok(None);
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
    use crate::test_utils::parse_from_str;

    #[test]
    fn felis_syn_typed_arg_parse_test_1() {
        let s = "(A : Prop)";
        let res = parse_from_str::<SynTypedArg<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(A : Prop)");
    }

    #[test]
    fn felis_syn_typed_arg_parse_test_2() {
        let s = "(x : A -> B)";
        let res = parse_from_str::<SynTypedArg<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(x : A -> B)");
    }
}
