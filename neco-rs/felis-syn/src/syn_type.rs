use crate::{
    parse::Parse,
    syn_ident::SynIdent,
    syn_typed_arg::SynTypedArg,
    to_felis_string::ToFelisString,
    token::{Token, TokenArrow, TokenCamma, TokenKeyword, TokenLParen, TokenRParen},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SynType {
    // #forall (A : Prop), A -> B
    Forall(SynTypeForall),
    // And A B
    App(SynTypeApp),
    // A
    Atom(SynTypeAtom),
    // A -> B
    Map(SynTypeMap),
    // (And A B)
    Paren(SynTypeParen),
}

impl ToFelisString for SynType {
    fn to_felis_string(&self) -> String {
        match self {
            SynType::Forall(forall) => {
                format!(
                    "#forall {}, {}",
                    forall.typed_arg.to_felis_string(),
                    forall.ty.to_felis_string()
                )
            }
            SynType::App(app) => {
                format!(
                    "{} {}",
                    app.left.to_felis_string(),
                    app.right.to_felis_string()
                )
            }
            SynType::Atom(atom) => atom.ident.ident.as_str().to_string(),
            SynType::Map(map) => {
                format!(
                    "{} -> {}",
                    map.from.to_felis_string(),
                    map.to.to_felis_string()
                )
            }
            SynType::Paren(paren) => {
                format!("({})", paren.ty.to_felis_string())
            }
        }
    }
}

impl Parse for SynType {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(forall) = SynTypeForall::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynType::Forall(forall)));
        }

        let Some(res) = SynTypeNoMap::parse(tokens, &mut k)? else {
            return Err(());
        };
        let mut res: SynType = res.into();

        if let Some(arrow) = TokenArrow::parse(tokens, &mut k)? {
            let Some(to) = SynType::parse(tokens, &mut k)? else {
                return Err(());
            };
            res = SynType::Map(SynTypeMap {
                from: Box::new(res),
                arrow,
                to: Box::new(to),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynTypeForall {
    pub keyword_forall: TokenKeyword,
    pub typed_arg: Box<SynTypedArg>,
    pub camma: TokenCamma,
    pub ty: Box<SynType>,
}

impl Parse for SynTypeForall {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_forall) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_forall.keyword != "#forall" {
            return Ok(None);
        }

        let Some(typed_arg) = SynTypedArg::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(camma) = TokenCamma::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynTypeForall {
            keyword_forall,
            typed_arg: Box::new(typed_arg),
            camma,
            ty: Box::new(ty),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SynTypeNoMapAndApp {
    Atom(SynTypeAtom),
    Paren(SynTypeParen),
}

impl From<SynTypeNoMapAndApp> for SynTypeNoMap {
    fn from(ty: SynTypeNoMapAndApp) -> SynTypeNoMap {
        match ty {
            SynTypeNoMapAndApp::Atom(atom) => SynTypeNoMap::Atom(atom),
            SynTypeNoMapAndApp::Paren(paren) => SynTypeNoMap::Paren(paren),
        }
    }
}

impl From<SynTypeNoMapAndApp> for SynType {
    fn from(ty: SynTypeNoMapAndApp) -> SynType {
        match ty {
            SynTypeNoMapAndApp::Atom(atom) => SynType::Atom(atom),
            SynTypeNoMapAndApp::Paren(paren) => SynType::Paren(paren),
        }
    }
}

impl Parse for SynTypeNoMapAndApp {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(paren) = SynTypeParen::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynTypeNoMapAndApp::Paren(paren)));
        }

        if let Some(atom) = SynTypeAtom::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynTypeNoMapAndApp::Atom(atom)));
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SynTypeNoMap {
    App(SynTypeApp),
    Atom(SynTypeAtom),
    Paren(SynTypeParen),
}

impl From<SynTypeNoMap> for SynType {
    fn from(ty: SynTypeNoMap) -> SynType {
        match ty {
            SynTypeNoMap::App(app) => SynType::App(app),
            SynTypeNoMap::Atom(atom) => SynType::Atom(atom),
            SynTypeNoMap::Paren(paren) => SynType::Paren(paren),
        }
    }
}

impl Parse for SynTypeNoMap {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(res) = SynTypeNoMapAndApp::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let mut res: SynTypeNoMap = res.into();

        while let Some(right) = SynTypeNoMapAndApp::parse(tokens, &mut k)? {
            res = SynTypeNoMap::App(SynTypeApp {
                left: Box::new(res.into()),
                right: Box::new(right.into()),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynTypeApp {
    pub left: Box<SynType>,
    pub right: Box<SynType>,
}

impl Parse for SynTypeApp {
    fn parse(_tokens: &[Token], _i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynTypeMap {
    pub from: Box<SynType>,
    pub arrow: TokenArrow,
    pub to: Box<SynType>,
}

impl Parse for SynTypeMap {
    fn parse(_tokens: &[Token], _i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynTypeAtom {
    pub ident: SynIdent,
}

impl Parse for SynTypeAtom {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;
        if *i >= tokens.len() {
            return Ok(None);
        };

        let Some(ident) = SynIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynTypeAtom { ident }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynTypeParen {
    pub lparen: TokenLParen,
    pub ty: Box<SynType>,
    pub rparen: TokenRParen,
}

impl Parse for SynTypeParen {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(rparen) = TokenRParen::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynTypeParen {
            lparen,
            ty: Box::new(ty),
            rparen,
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::Parser;

    use super::*;

    #[test]
    fn felis_syn_type_parse_test_1() {
        let s = "And A B";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "And A B");
    }

    #[test]
    fn felis_syn_type_parse_test_2() {
        let s = "(And A B)";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(And A B)");
    }

    #[test]
    fn felis_syn_type_parse_test_3() {
        let s = "#forall (A : Prop), Hoge A";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "#forall (A : Prop), Hoge A");
    }

    #[test]
    fn felis_syn_type_parse_test_4() {
        let s = "#forall (A : Prop), #forall (B : Prop), And A B -> Or A B";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(
            res.to_felis_string(),
            "#forall (A : Prop), #forall (B : Prop), And A B -> Or A B"
        );
    }

    #[test]
    fn felis_syn_type_parse_test_5() {
        let s = "(A -> B)";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(A -> B)");
    }
}
