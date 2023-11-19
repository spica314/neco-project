use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    to_felis_string::ToFelisString,
    token::Token,
    SynTreeId,
};

use super::{SynExpr, SynExprNoApp};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprApp<D: Decoration> {
    id: SynTreeId,
    pub exprs: Vec<SynExpr<D>>,
    ext: D::ExprApp,
}

impl<D: Decoration> ToFelisString for SynExprApp<D> {
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

impl<D: Decoration> SynExprApp<D> {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprApp<UD> {
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
            ext: (),
        }))
    }
}
