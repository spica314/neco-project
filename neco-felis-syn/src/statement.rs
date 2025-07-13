use crate::{
    Parse, ParseError, Phase, PhaseParse, StatementAssign, StatementFieldAssign, StatementLet,
    StatementLetMut, Term, token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Statement<P: Phase> {
    Let(StatementLet<P>),
    LetMut(StatementLetMut<P>),
    Assign(StatementAssign<P>),
    FieldAssign(StatementFieldAssign<P>),
    Expr(Term<P>),
    Ext(P::StatementExt),
}

impl Parse for Statement<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(statement_let_mut) = StatementLetMut::parse(tokens, i)? {
            return Ok(Some(Statement::LetMut(statement_let_mut)));
        }

        if let Some(statement_let) = StatementLet::parse(tokens, i)? {
            return Ok(Some(Statement::Let(statement_let)));
        }

        if let Some(statement_field_assign) = StatementFieldAssign::parse(tokens, i)? {
            return Ok(Some(Statement::FieldAssign(statement_field_assign)));
        }

        if let Some(statement_assign) = StatementAssign::parse(tokens, i)? {
            return Ok(Some(Statement::Assign(statement_assign)));
        }

        if let Some(term) = Term::parse(tokens, i)? {
            return Ok(Some(Statement::Expr(term)));
        }

        Ok(None)
    }
}
