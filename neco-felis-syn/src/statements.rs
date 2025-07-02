use crate::{Parse, ParseError, Phase, PhaseParse, StatementsThen, Term, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Statements<P: Phase> {
    Term(Term<P>),
    Then(StatementsThen<P>),
    Nil,
}

impl Parse for Statements<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(statements_then) = StatementsThen::parse(tokens, i)? {
            return Ok(Some(Statements::Then(statements_then)));
        }

        if let Some(term) = Term::parse(tokens, i)? {
            return Ok(Some(Statements::Term(term)));
        }

        Ok(Some(Statements::Nil))
    }
}
