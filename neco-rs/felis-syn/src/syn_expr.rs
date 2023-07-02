use crate::{
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{
        Token, TokenArrow2, TokenCamma, TokenColonColon, TokenIdent, TokenKeyword, TokenLBrace,
        TokenLParen, TokenRBrace, TokenRParen, TokenString,
    },
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprString {
    id: SynTreeId,
    pub token_string: TokenString,
}

impl SynExprString {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprString {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let token_string = if let Some(token_string) = TokenString::parse(tokens, &mut k)? {
            token_string
        } else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprString {
            id: SynTreeId::new(),
            token_string,
        }))
    }
}

impl ToFelisString for SynExprString {
    fn to_felis_string(&self) -> String {
        self.token_string.to_felis_string()
    }
}

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
    id: SynTreeId,
    pub path: Vec<(TokenIdent, TokenColonColon)>,
    pub ident: TokenIdent,
}

impl SynExprIdentWithPath {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
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
        Ok(Some(SynExprIdentWithPath {
            id: SynTreeId::new(),
            path,
            ident,
        }))
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
    IdentWithPath(SynExprIdentWithPath),
    App(SynExprApp),
    Match(SynExprMatch),
    Paren(SynExprParen),
    String(SynExprString),
}

impl SynExpr {
    pub fn table_id(&self) -> SynTreeId {
        match self {
            SynExpr::Ident(expr_ident) => expr_ident.syn_tree_id(),
            SynExpr::App(expr_app) => expr_app.syn_tree_id(),
            SynExpr::Match(expr_match) => expr_match.syn_tree_id(),
            SynExpr::Paren(expr_paren) => expr_paren.syn_tree_id(),
            SynExpr::IdentWithPath(expr_ident_with_path) => expr_ident_with_path.syn_tree_id(),
            SynExpr::String(expr_string) => expr_string.syn_tree_id(),
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
            SynExpr::App(expr_app) => expr_app.to_felis_string(),
            SynExpr::Paren(_) => todo!(),
            SynExpr::IdentWithPath(expr_ident_with_path) => expr_ident_with_path.to_felis_string(),
            SynExpr::String(string) => string.to_felis_string(),
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
    IdentWithPath(SynExprIdentWithPath),
    Match(SynExprMatch),
    Paren(SynExprParen),
    String(SynExprString),
}

impl Parse for SynExprNoApp {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(expr_string) = SynExprString::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::String(expr_string)));
        }

        if let Some(expr_match) = SynExprMatch::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::Match(expr_match)));
        }

        if let Some(expr_paren) = SynExprParen::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::Paren(expr_paren)));
        }

        if let Some(expr_ident_with_path) = SynExprIdentWithPath::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::IdentWithPath(expr_ident_with_path)));
        }

        Ok(None)
    }
}

impl From<SynExprNoApp> for SynExpr {
    fn from(value: SynExprNoApp) -> Self {
        match value {
            SynExprNoApp::IdentWithPath(expr_ident_with_path) => {
                SynExpr::IdentWithPath(expr_ident_with_path)
            }
            SynExprNoApp::Match(expr_match) => SynExpr::Match(expr_match),
            SynExprNoApp::Paren(expr_paren) => SynExpr::Paren(expr_paren),
            SynExprNoApp::String(expr_string) => SynExpr::String(expr_string),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprApp {
    id: SynTreeId,
    pub exprs: Vec<SynExpr>,
}

impl ToFelisString for SynExprApp {
    fn to_felis_string(&self) -> String {
        let mut res = String::new();
        res.push_str(&self.exprs[0].to_felis_string());
        for expr in self.exprs.iter().skip(1) {
            res.push(' ');
            res.push_str(&expr.to_felis_string());
        }
        res
    }
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
        assert!(matches!(res, SynExpr::IdentWithPath(_)));
        let SynExpr::IdentWithPath(ident_with_path) = res else { panic!() };
        assert_eq!(ident_with_path.ident.ident.as_str(), "x");
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

    #[test]
    fn felis_syn_expr_parse_test_7() {
        let s = "#match x { y::z w => t::u, }";
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
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t::u");
    }

    #[test]
    fn felis_syn_expr_parse_test_8() {
        let s = "#match x { y::z w => t::u a, }";
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
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t::u a");
    }
}
