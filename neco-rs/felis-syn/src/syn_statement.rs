use crate::{parse::Parse, syn_expr::SynExpr, to_felis_string::ToFelisString, token::Token};

pub use syn_statement_let::*;

pub mod syn_statement_let;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynStatement {
    Expr(SynExpr),
    Let(SynStatementLet),
}

impl Parse for SynStatement {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(statement_let) = SynStatementLet::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::Let(statement_let)));
        }

        if let Some(expr) = SynExpr::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynStatement::Expr(expr)));
        }

        Ok(None)
    }
}

impl ToFelisString for SynStatement {
    fn to_felis_string(&self) -> String {
        match self {
            SynStatement::Expr(expr) => expr.to_felis_string(),
            SynStatement::Let(statement_let) => statement_let.to_felis_string(),
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
        let res = parser.parse::<SynStatement>(&s);
        assert!(res.is_ok());
        let (statement, _) = res.unwrap();
        assert!(statement.is_some());
        let statement = statement.unwrap();
        assert!(matches!(statement, SynStatement::Let(_)));
    }
}
