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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum SynType {
    App(SynTypeApp),
    Atom(SynTypeAtom),
    Map(SynTypeMap),
    Paren(SynTypeParen),
}

impl SynType {
    pub fn to_felis_string(&self) -> String {
        match self {
            SynType::App(app) => {
                format!(
                    "{} {}",
                    app.left.to_felis_string(),
                    app.right.to_felis_string()
                )
            }
            SynType::Atom(atom) => atom.ident.as_str().to_string(),
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

#[derive(Debug, Clone)]
pub enum SynTypeNoMapAndApp {
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

#[derive(Debug, Clone)]
pub enum SynTypeNoMap {
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

impl Parse for SynType {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

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

#[derive(Debug, Clone)]
pub struct SynTypeApp {
    pub left: Box<SynType>,
    pub right: Box<SynType>,
}

impl Parse for SynTypeApp {
    fn parse(_tokens: &[Token], _i: &mut usize) -> Result<Option<Self>, ()> {
        todo!()
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct SynTypeAtom {
    pub ident: TokenIdent,
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

#[derive(Debug, Clone)]
pub struct SynVariant {
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: SynType,
    pub camma: TokenCamma,
}

impl Parse for SynVariant {
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
        }))
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

        *i = k;
        Ok(Some(SynTypeDef {
            keyword_type,
            name,
            args,
            colon,
            ty_ty: Box::new(ty_ty),
            lbrace,
            variants,
            rbrace,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn felis_syn_parse_test_1() {
        let s = "And A B";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs).unwrap();
        let mut i = 0;
        let res = SynType::parse(&tokens, &mut i);
        assert_eq!(i, 3);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.to_felis_string(), "And A B");
    }

    #[test]
    fn felis_syn_parse_test_2() {
        let s = include_str!("../../../library/wip/and.fe");
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs).unwrap();
        eprintln!("tokens = {:?}", tokens);
        let mut i = 0;
        let res = SynTypeDef::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(i, tokens.len());
        assert_eq!(res.name.as_str(), "And");
        assert_eq!(res.args.len(), 2);
        assert_eq!(res.args[0].name.as_str(), "A");
        assert_eq!(res.args[0].ty.to_felis_string(), "Prop");
        assert_eq!(res.args[1].name.as_str(), "B");
        assert_eq!(res.args[1].ty.to_felis_string(), "Prop");
        assert_eq!(res.ty_ty.to_felis_string(), "Prop");
        assert_eq!(res.variants.len(), 1);
        assert_eq!(res.variants[0].name.as_str(), "conj");
        assert_eq!(res.variants[0].ty.to_felis_string(), "A -> B -> And A B");
    }

    #[test]
    fn felis_syn_parse_test_3() {
        let s = include_str!("../../../library/wip/or.fe");
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs).unwrap();
        eprintln!("tokens = {:?}", tokens);
        let mut i = 0;
        let res = SynTypeDef::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(i, tokens.len());
        assert_eq!(res.name.as_str(), "Or");
        assert_eq!(res.args.len(), 2);
        assert_eq!(res.args[0].name.as_str(), "A");
        assert_eq!(res.args[0].ty.to_felis_string(), "Prop");
        assert_eq!(res.args[1].name.as_str(), "B");
        assert_eq!(res.args[1].ty.to_felis_string(), "Prop");
        assert_eq!(res.ty_ty.to_felis_string(), "Prop");
        assert_eq!(res.variants.len(), 2);
        assert_eq!(res.variants[0].name.as_str(), "or_introl");
        assert_eq!(res.variants[0].ty.to_felis_string(), "A -> Or A B");
        assert_eq!(res.variants[1].name.as_str(), "or_intror");
        assert_eq!(res.variants[1].ty.to_felis_string(), "B -> Or A B");
    }
}
