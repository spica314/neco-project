use crate::{
    parse::Parse,
    syn_type::SynType,
    syn_typed_arg::SynTypedArg,
    to_felis_string::ToFelisString,
    token::{
        Token, TokenArrow, TokenArrow2, TokenCamma, TokenIdent, TokenKeyword, TokenLBrace,
        TokenRBrace,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynFnDef {
    pub keyword_fn: TokenKeyword,
    pub name: TokenIdent,
    pub args: Vec<SynTypedArg>,
    pub arrow: TokenArrow,
    pub res_ty: SynType,
    pub fn_block: SynFnBlock,
}

impl Parse for SynFnDef {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_fn) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_fn.keyword != "#fn" {
            return Ok(None);
        };

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let mut args = vec![];
        while let Some(arg) = SynTypedArg::parse(tokens, &mut k)? {
            args.push(arg);
        }

        let Some(arrow) = TokenArrow::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(res_ty) = SynType::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(fn_block) = SynFnBlock::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynFnDef {
            keyword_fn,
            name,
            args,
            arrow,
            res_ty,
            fn_block,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynFnBlock {
    pub lbrace: TokenLBrace,
    pub statements: Vec<SynStatement>,
    pub rbrace: TokenRBrace,
}

impl Parse for SynFnBlock {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let mut statements = vec![];
        while let Some(statement) = SynStatement::parse(tokens, &mut k)? {
            statements.push(statement);
        }

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynFnBlock {
            lbrace,
            statements,
            rbrace,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SynStatement {
    Expr(SynExpr),
}

impl Parse for SynStatement {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(expr) = SynExpr::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::Expr(expr)));
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynExprMatch {
    pub keyword_match: TokenKeyword,
    pub expr: Box<SynExpr>,
    pub lbrace: TokenLBrace,
    pub arms: Vec<SynExprMatchArm>,
    pub rbrace: TokenRBrace,
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
            keyword_match,
            expr: Box::new(expr),
            lbrace,
            arms,
            rbrace,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynExprMatchPattern {
    pub idents: Vec<TokenIdent>,
}

impl Parse for SynExprMatchPattern {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut idents = vec![];
        while let Some(ident) = TokenIdent::parse(tokens, &mut k)? {
            idents.push(ident);
        }

        if idents.is_empty() {
            return Ok(None);
        }

        *i = k;
        Ok(Some(SynExprMatchPattern { idents }))
    }
}

impl ToFelisString for SynExprMatchPattern {
    fn to_felis_string(&self) -> String {
        let mut s = String::new();
        if !self.idents.is_empty() {
            s.push_str(self.idents[0].as_str());
        }
        for x in &self.idents[1..] {
            s.push(' ');
            s.push_str(x.as_str());
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SynExpr {
    Ident(SynExprIdent),
    App(SynExprApp),
    Match(SynExprMatch),
}

impl Parse for SynExpr {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(expr_app) = SynExprApp::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExpr::App(expr_app)));
        }

        if let Some(expr_match) = SynExprMatch::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExpr::Match(expr_match)));
        }

        if let Some(ident) = SynExprIdent::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExpr::Ident(ident)));
        }

        Ok(None)
    }
}

impl ToFelisString for SynExpr {
    fn to_felis_string(&self) -> String {
        match self {
            SynExpr::Ident(expr_ident) => expr_ident.ident.as_str().to_string(),
            SynExpr::Match(_expr_match) => todo!(),
            SynExpr::App(_expr_app) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SynExprNoApp {
    Ident(SynExprIdent),
    Match(SynExprMatch),
}

impl Parse for SynExprNoApp {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(expr_match) = SynExprMatch::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynExprNoApp::Match(expr_match)));
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynExprApp {
    pub exprs: Vec<SynExpr>,
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
        Ok(Some(SynExprApp { exprs }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynExprIdent {
    pub ident: TokenIdent,
}

impl Parse for SynExprIdent {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynExprIdent { ident }))
    }
}

#[cfg(test)]
mod test {
    use crate::token::{lex, FileId};

    use super::*;

    #[test]
    fn felis_syn_expr_parse_test_1() {
        let s = "x";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynExpr::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Ident(_)));
        let SynExpr::Ident(ident) = res else { panic!() };
        assert_eq!(ident.ident.as_str(), "x");
    }

    #[test]
    fn felis_syn_expr_parse_test_2() {
        let s = "#match x { }";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynExpr::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(matches!(res, SynExpr::Match(_)));
        let SynExpr::Match(expr_match) = res else { panic!() };
        assert!(expr_match.arms.is_empty());
    }

    #[test]
    fn felis_syn_expr_parse_test_3() {
        let s = "#match x { y z => t, }";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynExpr::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
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
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynExpr::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
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
    fn felis_syn_fn_def_parse_test_1() {
        let s = "#fn f -> T { }";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynFnDef::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.as_str(), "f");
        assert_eq!(res.args.len(), 0);
        assert_eq!(res.fn_block.statements.len(), 0);
    }

    #[test]
    fn felis_syn_fn_def_parse_test_2() {
        let s = "#fn f -> T { x }";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynFnDef::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.as_str(), "f");
        assert_eq!(res.args.len(), 0);
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }

    #[test]
    fn felis_syn_fn_def_parse_test_3() {
        let s = "#fn f (x : T) -> T { x }";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynFnDef::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.as_str(), "f");
        assert_eq!(res.args.len(), 1);
        assert_eq!(res.args[0].to_felis_string(), "(x : T)");
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }

    #[test]
    fn felis_syn_fn_def_parse_test_4() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynFnDef::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name.as_str(), "proof");
        assert_eq!(res.args.len(), 3);
        assert_eq!(res.args[0].to_felis_string(), "(A : Prop)");
        assert_eq!(res.args[1].to_felis_string(), "(B : Prop)");
        assert_eq!(res.args[2].to_felis_string(), "(x : And A B)");
        assert_eq!(res.res_ty.to_felis_string(), "Or A B");
        assert_eq!(res.fn_block.statements.len(), 1);
        // todo: check expr
    }
}
