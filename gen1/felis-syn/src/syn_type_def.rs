use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_type::SynType,
    token::{Token, TokenCamma, TokenColon, TokenIdent, TokenKeyword, TokenLBrace, TokenRBrace},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeDef<D: Decoration> {
    pub keyword_type: TokenKeyword,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty_ty: Box<SynType<D>>,
    pub lbrace: TokenLBrace,
    pub variants: Vec<SynVariant<D>>,
    pub rbrace: TokenRBrace,
    pub ext: D::TypeDef,
}

impl Parse for SynTypeDef<UD> {
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

        *i = k;
        Ok(Some(SynTypeDef {
            keyword_type,
            name,
            colon,
            ty_ty: Box::new(ty_ty),
            lbrace,
            variants,
            rbrace,
            ext: (),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynVariant<D: Decoration> {
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType<D>,
    pub camma: TokenCamma,
    pub ext: D::Variant,
}

impl Parse for SynVariant<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(camma) = TokenCamma::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynVariant {
            name,
            colon,
            ty,
            camma,
            ext: (),
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{to_felis_string::ToFelisString, Parser};

    use super::*;

    #[test]
    fn felis_syn_type_def_parse_test_3() {
        let s = include_str!("../../../library/wip/and2.fe");
        let mut parser = Parser::new();
        let res = parser.parse::<SynTypeDef<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "And");
        assert_eq!(res.ty_ty.to_felis_string(), "Prop -> Prop -> Prop");
        assert_eq!(res.variants.len(), 1);
        assert_eq!(res.variants[0].name.ident.as_str(), "conj");
        assert_eq!(
            res.variants[0].ty.to_felis_string(),
            "(A : Prop) -> (B : Prop) -> A -> B -> And A B"
        );
    }

    #[test]
    fn felis_syn_type_def_parse_test_4() {
        let s = include_str!("../../../library/wip/or2.fe");
        let mut parser = Parser::new();
        let res = parser.parse::<SynTypeDef<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.ident.as_str(), "Or");
        assert_eq!(res.ty_ty.to_felis_string(), "Prop -> Prop -> Prop");
        assert_eq!(res.variants.len(), 2);
        assert_eq!(res.variants[0].name.ident.as_str(), "or_introl");
        assert_eq!(
            res.variants[0].ty.to_felis_string(),
            "(A : Prop) -> (B : Prop) -> A -> Or A B"
        );
        assert_eq!(res.variants[1].name.ident.as_str(), "or_intror");
        assert_eq!(
            res.variants[1].ty.to_felis_string(),
            "(A : Prop) -> (B : Prop) -> B -> Or A B"
        );
    }
}
