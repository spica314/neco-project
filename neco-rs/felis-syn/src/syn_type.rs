use crate::{
    parse::Parse,
    syn_typed_arg::SynTypedArg,
    to_felis_string::ToFelisString,
    token::{Token, TokenArrow, TokenIdent, TokenLParen, TokenRParen},
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynType {
    // And A B
    App(SynTypeApp),
    // A
    Atom(SynTypeAtom),
    // A -> B
    Map(SynTypeMap),
    // (And A B)
    Paren(SynTypeParen),
    // (A : Prop) -> B A
    DependentMap(SynTypeDependentMap),
}

impl SynType {
    pub fn syn_tree_id(&self) -> SynTreeId {
        match self {
            SynType::App(app) => app.syn_tree_id(),
            SynType::Atom(atom) => atom.syn_tree_id(),
            SynType::Map(map) => map.syn_tree_id(),
            SynType::Paren(_) => todo!(),
            SynType::DependentMap(dep_map) => dep_map.syn_tree_id(),
        }
    }
    pub fn as_dependent_map(&self) -> Option<&SynTypeDependentMap> {
        match self {
            SynType::DependentMap(dep_map) => Some(dep_map),
            _ => None,
        }
    }
}

impl ToFelisString for SynType {
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
        }
    }
}

impl Parse for SynType {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(dependent_map) = SynTypeDependentMap::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynType::DependentMap(dependent_map)));
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
                id: SynTreeId::new(),
                from: Box::new(res),
                arrow,
                to: Box::new(to),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
                id: SynTreeId::new(),
                left: Box::new(res.into()),
                right: Box::new(right.into()),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeApp {
    id: SynTreeId,
    pub left: Box<SynType>,
    pub right: Box<SynType>,
}

impl SynTypeApp {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynTypeApp {
    fn parse(_tokens: &[Token], _i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeDependentMap {
    id: SynTreeId,
    pub from: Box<SynTypedArg>,
    pub arrow: TokenArrow,
    pub to: Box<SynType>,
}

impl SynTypeDependentMap {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynTypeDependentMap {
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
            id: SynTreeId::new(),
            from: Box::new(from),
            arrow,
            to: Box::new(to),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeMap {
    id: SynTreeId,
    pub from: Box<SynType>,
    pub arrow: TokenArrow,
    pub to: Box<SynType>,
}

impl SynTypeMap {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynTypeMap {
    fn parse(_tokens: &[Token], _i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynTypeAtom {
    pub ident: TokenIdent,
}

impl SynTypeAtom {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.ident.syn_tree_id()
    }
}

impl Parse for SynTypeAtom {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;
        if *i >= tokens.len() {
            return Ok(None);
        };

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynTypeAtom { ident }))
    }
}

impl ToFelisString for SynTypeAtom {
    fn to_felis_string(&self) -> String {
        self.ident.as_str().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        let (res, _) = res.unwrap();
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
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(And A B)");
    }

    #[test]
    fn felis_syn_type_parse_test_3() {
        let s = "(A : Prop) -> Hoge A";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), s);
    }

    #[test]
    fn felis_syn_type_parse_test_4() {
        let s = "(A : Prop) -> (B : Prop) -> And A B -> Or A B";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), s);
    }

    #[test]
    fn felis_syn_type_parse_test_5() {
        let s = "(A -> B)";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(A -> B)");
    }

    #[test]
    fn felis_syn_type_parse_test_6() {
        let s = "(x : Nat) -> P x";
        let mut parser = Parser::new();
        let res = parser.parse::<SynType>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "(x : Nat) -> P x");
    }
}
