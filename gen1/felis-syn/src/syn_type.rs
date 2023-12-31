use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_typed_arg::SynTypedArg,
    to_felis_string::ToFelisString,
    token::{Token, TokenArrow, TokenIdent, TokenLParen, TokenRParen},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynType<D: Decoration> {
    // And A B
    App(SynTypeApp<D>),
    // A
    Atom(SynTypeAtom<D>),
    // A -> B
    Map(SynTypeMap<D>),
    // (And A B)
    Paren(SynTypeParen<D>),
    // (A : Prop) -> B A
    DependentMap(SynTypeDependentMap<D>),
    // () (unit type)
    Unit(SynTypeUnit<D>),
}

impl<D: Decoration> SynType<D> {
    pub fn as_dependent_map(&self) -> Option<&SynTypeDependentMap<D>> {
        match self {
            SynType::DependentMap(dep_map) => Some(dep_map),
            _ => None,
        }
    }
}

impl<D: Decoration> ToFelisString for SynType<D> {
    fn to_felis_string(&self) -> String {
        match self {
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
            SynType::DependentMap(dependent_map) => {
                format!(
                    "{} -> {}",
                    dependent_map.from.to_felis_string(),
                    dependent_map.to.to_felis_string()
                )
            }
            SynType::Unit(_unit) => "()".to_string(),
        }
    }
}

impl Parse for SynType<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(dependent_map) = SynTypeDependentMap::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynType::DependentMap(dependent_map)));
        }

        let Some(res) = SynTypeNoMap::parse(tokens, &mut k)? else {
            return Err(());
        };
        let mut res: SynType<UD> = res.into();

        if let Some(arrow) = TokenArrow::parse(tokens, &mut k)? {
            let Some(to) = SynType::parse(tokens, &mut k)? else {
                return Err(());
            };
            res = SynType::Map(SynTypeMap {
                from: Box::new(res),
                arrow,
                to: Box::new(to),
                ext: (),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeUnit<D: Decoration> {
    pub lparen: TokenLParen,
    pub rparen: TokenRParen,
    pub ext: D::TypeUnit,
}

impl Parse for SynTypeUnit<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let Some(rparen) = TokenRParen::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynTypeUnit {
            lparen,
            rparen,
            ext: (),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SynTypeNoMapAndApp<D: Decoration> {
    Atom(SynTypeAtom<D>),
    Paren(SynTypeParen<D>),
    Unit(SynTypeUnit<D>),
}

impl<D: Decoration> From<SynTypeNoMapAndApp<D>> for SynTypeNoMap<D> {
    fn from(ty: SynTypeNoMapAndApp<D>) -> SynTypeNoMap<D> {
        match ty {
            SynTypeNoMapAndApp::Atom(atom) => SynTypeNoMap::Atom(atom),
            SynTypeNoMapAndApp::Paren(paren) => SynTypeNoMap::Paren(paren),
            SynTypeNoMapAndApp::Unit(unit) => SynTypeNoMap::Unit(unit),
        }
    }
}

impl<D: Decoration> From<SynTypeNoMapAndApp<D>> for SynType<D> {
    fn from(ty: SynTypeNoMapAndApp<D>) -> SynType<D> {
        match ty {
            SynTypeNoMapAndApp::Atom(atom) => SynType::Atom(atom),
            SynTypeNoMapAndApp::Paren(paren) => SynType::Paren(paren),
            SynTypeNoMapAndApp::Unit(unit) => SynType::Unit(unit),
        }
    }
}

impl Parse for SynTypeNoMapAndApp<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(unit) = SynTypeUnit::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynTypeNoMapAndApp::Unit(unit)));
        }

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SynTypeNoMap<D: Decoration> {
    App(SynTypeApp<D>),
    Atom(SynTypeAtom<D>),
    Paren(SynTypeParen<D>),
    Unit(SynTypeUnit<D>),
}

impl<D: Decoration> From<SynTypeNoMap<D>> for SynType<D> {
    fn from(ty: SynTypeNoMap<D>) -> SynType<D> {
        match ty {
            SynTypeNoMap::App(app) => SynType::App(app),
            SynTypeNoMap::Atom(atom) => SynType::Atom(atom),
            SynTypeNoMap::Paren(paren) => SynType::Paren(paren),
            SynTypeNoMap::Unit(unit) => SynType::Unit(unit),
        }
    }
}

impl Parse for SynTypeNoMap<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(res) = SynTypeNoMapAndApp::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        let mut res: SynTypeNoMap<UD> = res.into();

        while let Some(right) = SynTypeNoMapAndApp::parse(tokens, &mut k)? {
            res = SynTypeNoMap::App(SynTypeApp {
                left: Box::new(res.into()),
                right: Box::new(right.into()),
                ext: (),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeApp<D: Decoration> {
    pub left: Box<SynType<D>>,
    pub right: Box<SynType<D>>,
    pub ext: D::TypeApp,
}

impl Parse for SynTypeApp<UD> {
    fn parse(_tokens: &[Token], _i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeDependentMap<D: Decoration> {
    pub from: Box<SynTypedArg<D>>,
    pub arrow: TokenArrow,
    pub to: Box<SynType<D>>,
    pub ext: D::TypeDependentMap,
}

impl Parse for SynTypeDependentMap<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;
        if *i >= tokens.len() {
            return Ok(None);
        }

        let Some(from) = SynTypedArg::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(arrow) = TokenArrow::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(to) = SynType::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynTypeDependentMap {
            from: Box::new(from),
            arrow,
            to: Box::new(to),
            ext: (),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeMap<D: Decoration> {
    pub from: Box<SynType<D>>,
    pub arrow: TokenArrow,
    pub to: Box<SynType<D>>,
    pub ext: D::TypeMap,
}

impl Parse for SynTypeMap<UD> {
    fn parse(_tokens: &[Token], _i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeAtom<D: Decoration> {
    pub ident: TokenIdent,
    pub ext: D::TypeAtom,
}

impl Parse for SynTypeAtom<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;
        if *i >= tokens.len() {
            return Ok(None);
        };

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynTypeAtom { ident, ext: () }))
    }
}

impl<D: Decoration> ToFelisString for SynTypeAtom<D> {
    fn to_felis_string(&self) -> String {
        self.ident.as_str().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeParen<D: Decoration> {
    pub lparen: TokenLParen,
    pub ty: Box<SynType<D>>,
    pub rparen: TokenRParen,
    pub ext: D::TypeParen,
}

impl Parse for SynTypeParen<UD> {
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
            ext: (),
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
        let res = parser.parse::<SynType<UD>>(&s);
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
        let res = parser.parse::<SynType<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(And A B)");
    }

    #[test]
    fn felis_syn_type_parse_test_3() {
        let s = "(A : Prop) -> Hoge A";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), s);
    }

    #[test]
    fn felis_syn_type_parse_test_4() {
        let s = "(A : Prop) -> (B : Prop) -> And A B -> Or A B";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), s);
    }

    #[test]
    fn felis_syn_type_parse_test_5() {
        let s = "(A -> B)";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(A -> B)");
    }

    #[test]
    fn felis_syn_type_parse_test_6() {
        let s = "(x : Nat) -> P x";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType<UD>>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(x : Nat) -> P x");
    }
}
