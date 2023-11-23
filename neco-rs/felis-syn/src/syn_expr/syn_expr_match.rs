use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{Token, TokenArrow2, TokenCamma, TokenIdent, TokenKeyword, TokenLBrace, TokenRBrace},
    SynTreeId,
};

use super::{SynExpr, SynExprIdentWithPath};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprMatch<D: Decoration> {
    pub id: SynTreeId,
    pub keyword_match: TokenKeyword,
    pub expr: Box<SynExpr<D>>,
    pub lbrace: TokenLBrace,
    pub arms: Vec<SynExprMatchArm<D>>,
    pub rbrace: TokenRBrace,
    pub ext: D::ExprMatch,
}

impl<D: Decoration> SynExprMatch<D> {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprMatch<UD> {
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
            ext: (),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprMatchArm<D: Decoration> {
    pub pattern: SynExprMatchPattern<D>,
    pub arrow2: TokenArrow2,
    pub expr: SynExpr<D>,
    pub camma: TokenCamma,
}

impl Parse for SynExprMatchArm<UD> {
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
pub struct SynExprMatchPattern<D: Decoration> {
    pub type_constructor: SynExprIdentWithPath<D>,
    pub idents: Vec<TokenIdent>,
    pub ext: D::ExprMatchPattern,
}

impl Parse for SynExprMatchPattern<UD> {
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
            ext: (),
        }))
    }
}

impl<D: Decoration> ToFelisString for SynExprMatchPattern<D> {
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
