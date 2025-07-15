use crate::{Parse, ParseError, Phase, PhaseParse, Statement, StatementsThen, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Statements<P: Phase> {
    Statement(Box<Statement<P>>),
    Then(StatementsThen<P>),
    Nil,
}

impl Parse for Statements<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(statements_then) = StatementsThen::parse(tokens, i)? {
            return Ok(Some(Statements::Then(statements_then)));
        }

        if let Some(statement) = Statement::parse(tokens, i)? {
            return Ok(Some(Statements::Statement(Box::new(statement))));
        }
        Ok(Some(Statements::Nil))
    }
}
