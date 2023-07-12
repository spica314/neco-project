use crate::{
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenArrow2, TokenCamma, TokenIdent, TokenKeyword, TokenLBrace, TokenRBrace},
    SynTreeId,
};

use super::{SynExpr, SynExprIdentWithPath};

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
