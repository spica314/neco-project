mod syn_expr_app;
mod syn_expr_block;
mod syn_expr_ident_with_path;
mod syn_expr_match;
mod syn_expr_number;
mod syn_expr_paren;
mod syn_expr_string;

pub use syn_expr_app::*;
pub use syn_expr_block::*;
pub use syn_expr_ident_with_path::*;
pub use syn_expr_match::*;
pub use syn_expr_number::*;
pub use syn_expr_paren::*;
pub use syn_expr_string::*;

use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    to_felis_string::ToFelisString,
    token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynExpr<D: Decoration> {
    IdentWithPath(SynExprIdentWithPath<D>),
    App(SynExprApp<D>),
    Match(SynExprMatch<D>),
    Paren(SynExprParen<D>),
    String(SynExprString<D>),
    Number(SynExprNumber<D>),
    Block(SynExprBlock<D>),
}

impl Parse for SynExpr<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        // In order to correctly parse `#match x { ... }`, Block is not included in SynExprNoApp.
        if let Some(expr_block) = SynExprBlock::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExpr::Block(expr_block)));
        }

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

impl<D: Decoration> ToFelisString for SynExpr<D> {
    fn to_felis_string(&self) -> String {
        match self {
            SynExpr::Match(_expr_match) => todo!(),
            SynExpr::App(expr_app) => expr_app.to_felis_string(),
            SynExpr::Paren(_) => todo!(),
            SynExpr::IdentWithPath(expr_ident_with_path) => expr_ident_with_path.to_felis_string(),
            SynExpr::String(string) => string.to_felis_string(),
            SynExpr::Block(expr_block) => expr_block.to_felis_string(),
            SynExpr::Number(number) => number.to_felis_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SynExprNoApp<D: Decoration> {
    IdentWithPath(SynExprIdentWithPath<D>),
    Match(SynExprMatch<D>),
    Paren(SynExprParen<D>),
    String(SynExprString<D>),
    Number(SynExprNumber<D>),
}

impl Parse for SynExprNoApp<UD> {
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

        if let Some(expr_number) = SynExprNumber::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::Number(expr_number)));
        }

        Ok(None)
    }
}

impl<D: Decoration> From<SynExprNoApp<D>> for SynExpr<D> {
    fn from(value: SynExprNoApp<D>) -> Self {
        match value {
            SynExprNoApp::IdentWithPath(expr_ident_with_path) => {
                SynExpr::IdentWithPath(expr_ident_with_path)
            }
            SynExprNoApp::Match(expr_match) => SynExpr::Match(expr_match),
            SynExprNoApp::Paren(expr_paren) => SynExpr::Paren(expr_paren),
            SynExprNoApp::String(expr_string) => SynExpr::String(expr_string),
            SynExprNoApp::Number(expr_number) => SynExpr::Number(expr_number),
        }
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
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::IdentWithPath(_)));
        let SynExpr::IdentWithPath(ident_with_path) = res else {
            panic!()
        };
        assert_eq!(ident_with_path.ident.ident.as_str(), "x");
    }

    #[test]
    fn felis_syn_expr_parse_test_2() {
        let s = "#match x { }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else {
            panic!()
        };
        assert!(expr_match.arms.is_empty());
    }

    #[test]
    fn felis_syn_expr_parse_test_3() {
        let s = "#match x { y z => t, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else {
            panic!()
        };
        assert_eq!(expr_match.arms.len(), 1);
        assert_eq!(expr_match.arms[0].pattern.to_felis_string(), "y z");
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t");
    }

    #[test]
    fn felis_syn_expr_parse_test_4() {
        let s = "#match x { y z => t, a => b, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else {
            panic!()
        };
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
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::App(_)));
        // todo: test for detail
    }

    #[test]
    fn felis_syn_expr_parse_test_6() {
        let s = "#match x { y::z w => t, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else {
            panic!()
        };
        assert_eq!(expr_match.arms.len(), 1);
        assert_eq!(expr_match.arms[0].pattern.to_felis_string(), "y::z w");
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t");
    }

    #[test]
    fn felis_syn_expr_parse_test_7() {
        let s = "#match x { y::z w => t::u, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else {
            panic!()
        };
        assert_eq!(expr_match.arms.len(), 1);
        assert_eq!(expr_match.arms[0].pattern.to_felis_string(), "y::z w");
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t::u");
    }

    #[test]
    fn felis_syn_expr_parse_test_8() {
        let s = "#match x { y::z w => t::u a, }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else {
            panic!()
        };
        assert_eq!(expr_match.arms.len(), 1);
        assert_eq!(expr_match.arms[0].pattern.to_felis_string(), "y::z w");
        assert_eq!(expr_match.arms[0].expr.to_felis_string(), "t::u a");
    }

    #[test]
    fn felis_syn_expr_parse_test_9() {
        let s = "{ 42 }";
        let mut parser = Parser::new();
        let res = parser.parse::<SynExpr<UD>>(s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Block(_)));
        assert_eq!(res.to_felis_string(), s);
    }
}
