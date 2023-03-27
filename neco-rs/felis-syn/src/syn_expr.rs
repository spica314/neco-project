use crate::{
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{
        Token, TokenArrow2, TokenCamma, TokenColonColon, TokenIdent, TokenKeyword, TokenLBrace,
        TokenLParen, TokenRBrace, TokenRParen,
    },
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprMatch {
    id: SynTreeId,
    pub keyword_match: TokenKeyword,
    pub expr: Box<SynExpr>,
    pub lbrace: TokenLBrace,
    pub arms: Vec<SynExprMatchArm>,
    pub rbrace: TokenRBrace,
}

impl SynExprMatch {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprMatch {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_match) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_match.keyword != "#match" {
            return Ok(None);
        }

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        let mut arms = vec![];
        while let Some(arm) = SynExprMatchArm::parse(tokens, &mut k)? {
            arms.push(arm);
        }

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynExprMatch {
            id: SynTreeId::new(),
            keyword_match,
            expr: Box::new(expr),
            lbrace,
            arms,
            rbrace,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprMatchArm {
    pub pattern: SynExprMatchPattern,
    pub arrow2: TokenArrow2,
    pub expr: SynExpr,
    pub camma: TokenCamma,
}

impl Parse for SynExprMatchArm {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(pattern) = SynExprMatchPattern::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(arrow2) = TokenArrow2::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(camma) = TokenCamma::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynExprMatchArm {
            pattern,
            arrow2,
            expr,
            camma,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprIdentWithPath {
    pub path: Vec<(TokenIdent, TokenColonColon)>,
    pub ident: TokenIdent,
}

impl ToFelisString for SynExprIdentWithPath {
    fn to_felis_string(&self) -> String {
        let mut res = String::new();
        for t in &self.path {
            res.push_str(t.0.as_str());
            res.push_str("::");
        }
        res.push_str(self.ident.as_str());
        res
    }
}

impl Parse for SynExprIdentWithPath {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut path = vec![];
        let Some(mut ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        while let Some(colon_colon) = TokenColonColon::parse(tokens, &mut k)? {
            path.push((ident.clone(), colon_colon));
            let Some(ident2) = TokenIdent::parse(tokens, &mut k)? else {
                return Err(());
            };
            ident = ident2;
        }

        *i = k;
        Ok(Some(SynExprIdentWithPath { path, ident }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprMatchPattern {
    pub type_constructor: SynExprIdentWithPath,
    pub idents: Vec<TokenIdent>,
}

impl Parse for SynExprMatchPattern {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(type_constructor) = SynExprIdentWithPath::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let mut idents = vec![];
        while let Some(ident) = TokenIdent::parse(tokens, &mut k)? {
            idents.push(ident);
        }

        *i = k;
        Ok(Some(SynExprMatchPattern {
            type_constructor,
            idents,
        }))
    }
}

impl ToFelisString for SynExprMatchPattern {
    fn to_felis_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.type_constructor.to_felis_string());
        for x in &self.idents[0..] {
            s.push(' ');
            s.push_str(x.ident.as_str());
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynExpr {
    Ident(SynExprIdent),
    App(SynExprApp),
    Match(SynExprMatch),
    Paren(SynExprParen),
}

impl SynExpr {
    pub fn table_id(&self) -> SynTreeId {
        match self {
            SynExpr::Ident(expr_ident) => expr_ident.syn_tree_id(),
            SynExpr::App(expr_app) => expr_app.syn_tree_id(),
            SynExpr::Match(expr_match) => expr_match.syn_tree_id(),
            SynExpr::Paren(expr_paren) => expr_paren.syn_tree_id(),
        }
    }
}

impl Parse for SynExpr {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(expr_app) = SynExprApp::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExpr::App(expr_app)));
        }

        if let Some(expr) = SynExprNoApp::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(expr.into()));
        }

        Ok(None)
    }
}

impl ToFelisString for SynExpr {
    fn to_felis_string(&self) -> String {
        match self {
            SynExpr::Ident(expr_ident) => expr_ident.ident.ident.as_str().to_string(),
            SynExpr::Match(_expr_match) => todo!(),
            SynExpr::App(_expr_app) => todo!(),
            SynExpr::Paren(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprParen {
    id: SynTreeId,
    pub lparen: TokenLParen,
    pub expr: Box<SynExpr>,
    pub rparen: TokenRParen,
}

impl SynExprParen {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprParen {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(expr) = SynExpr::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(rparen) = TokenRParen::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynExprParen {
            id: Default::default(),
            lparen,
            expr: Box::new(expr),
            rparen,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SynExprNoApp {
    Ident(SynExprIdent),
    Match(SynExprMatch),
    Paren(SynExprParen),
}

impl Parse for SynExprNoApp {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(expr_match) = SynExprMatch::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::Match(expr_match)));
        }

        if let Some(expr_paren) = SynExprParen::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::Paren(expr_paren)));
        }

        if let Some(expr_ident) = SynExprIdent::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::Ident(expr_ident)));
        }

        Ok(None)
    }
}

impl From<SynExprNoApp> for SynExpr {
    fn from(value: SynExprNoApp) -> Self {
        match value {
            SynExprNoApp::Ident(expr_ident) => SynExpr::Ident(expr_ident),
            SynExprNoApp::Match(expr_match) => SynExpr::Match(expr_match),
            SynExprNoApp::Paren(expr_paren) => SynExpr::Paren(expr_paren),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprApp {
    id: SynTreeId,
    pub exprs: Vec<SynExpr>,
}

impl SynExprApp {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprApp {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut exprs = vec![];
        while let Some(expr) = SynExprNoApp::parse(tokens, &mut k)? {
            exprs.push(expr.into());
        }

        if exprs.len() <= 1 {
            return Ok(None);
        }

        *i = k;
        Ok(Some(SynExprApp {
            id: Default::default(),
            exprs,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprIdent {
    id: SynTreeId,
    pub ident: TokenIdent,
}

impl SynExprIdent {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprIdent {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprIdent {
            id: Default::default(),
            ident,
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::Parser;

    use super::*;

    #[test]
    fn felis_syn_expr_parse_test_1() {
        let s = "x";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr>(s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Ident(_)));
        let SynExpr::Ident(ident) = res else { panic!() };
        assert_eq!(ident.ident.ident.as_str(), "x");
    }

    #[test]
    fn felis_syn_expr_parse_test_2() {
        let s = "#match x { }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr>(s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else { panic!() };
        assert!(expr_match.arms.is_empty());
    }

    #[test]
    fn felis_syn_expr_parse_test_3() {
        let s = "#match x { y z => t, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr>(s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else { panic!() };
        assert_eq!(expr_match.arms.len(), 1);
        assert_eq!(expr_match.arms[0].pattern.to_felis_string(), "y z");
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t");
    }

    #[test]
    fn felis_syn_expr_parse_test_4() {
        let s = "#match x { y z => t, a => b, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr>(s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else { panic!() };
        assert_eq!(expr_match.arms.len(), 2);
        assert_eq!(expr_match.arms[0].pattern.to_felis_string(), "y z");
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t");
        assert_eq!(expr_match.arms[1].pattern.to_felis_string(), "a");
        assert_eq!(expr_match.arms[1].expr.to_felis_string(), "b");
    }

    #[test]
    fn felis_syn_expr_parse_test_5() {
        let s = "f (g x)";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr>(s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::App(_)));
        // todo: test for detail
    }

    #[test]
    fn felis_syn_expr_parse_test_6() {
        let s = "#match x { y::z w => t, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr>(s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else { panic!() };
        assert_eq!(expr_match.arms.len(), 1);
        assert_eq!(expr_match.arms[0].pattern.to_felis_string(), "y::z w");
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t");
    }
}
