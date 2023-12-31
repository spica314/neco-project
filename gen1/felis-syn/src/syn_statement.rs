use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_expr::SynExpr,
    to_felis_string::ToFelisString,
    token::Token,
};

pub use syn_statement_let::*;

use self::{
    syn_statement_assign::SynStatementAssign, syn_statement_expr_semi::SynStatementExprSemi,
};

pub mod syn_statement_assign;
pub mod syn_statement_expr_semi;
pub mod syn_statement_let;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynStatement<D: Decoration> {
    Expr(SynExpr<D>),
    ExprSemi(SynStatementExprSemi<D>),
    Let(SynStatementLet<D>),
    Assign(SynStatementAssign<D>),
}

impl Parse for SynStatement<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(statement_let) = SynStatementLet::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::Let(statement_let)));
        }

        if let Some(statement_assign) = SynStatementAssign::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::Assign(statement_assign)));
        }

        if let Some(statement_expr_semi) = SynStatementExprSemi::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::ExprSemi(statement_expr_semi)));
        }

        if let Some(expr) = SynExpr::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::Expr(expr)));
        }

        Ok(None)
    }
}

impl<D: Decoration> ToFelisString for SynStatement<D> {
    fn to_felis_string(&self) -> String {
        match self {
            SynStatement::Expr(expr) => expr.to_felis_string(),
            SynStatement::ExprSemi(statement_expr_semi) => statement_expr_semi.to_felis_string(),
            SynStatement::Let(statement_let) => statement_let.to_felis_string(),
            SynStatement::Assign(_statement_assign) => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Parser;

    use super::*;

    #[test]
    fn test_parse_statement_let() {
        let s = "#let x = 1;";
        let mut parser = Parser::new();
        let res = parser.parse::<SynStatement<UD>>(&s);
        assert!(res.is_ok());
        let statement = res.unwrap();
        assert!(statement.is_some());
        let statement = statement.unwrap();
        assert!(matches!(statement, SynStatement::Let(_)));
    }
}
