use crate::{Parse, ParseError, Term, TermParen, TermVariable, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermApply {
    f: Box<Term>,
    args: Vec<Term>,
}

impl TermApply {
    pub fn f(&self) -> &Term {
        &self.f
    }

    pub fn args(&self) -> &[Term] {
        &self.args
    }
}

impl Parse for TermApply {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        let Some(f) = TermForApplyElem::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let mut args = vec![];
        while let Some(arg) = TermForApplyElem::parse(tokens, &mut k)? {
            args.push(arg.into());
        }

        let term_apply = TermApply {
            f: Box::new(f.into()),
            args,
        };

        *i = k;
        Ok(Some(term_apply))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TermForApplyElem {
    Paren(TermParen),
    Variable(TermVariable),
}

impl From<TermForApplyElem> for Term {
    fn from(value: TermForApplyElem) -> Self {
        match value {
            TermForApplyElem::Paren(term_paren) => Term::Paren(term_paren),
            TermForApplyElem::Variable(term_variable) => Term::Variable(term_variable),
        }
    }
}

impl Parse for TermForApplyElem {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(term_paren) = TermParen::parse(tokens, i)? {
            return Ok(Some(TermForApplyElem::Paren(term_paren)));
        }

        if let Some(term_variable) = TermVariable::parse(tokens, i)? {
            return Ok(Some(TermForApplyElem::Variable(term_variable)));
        }

        Ok(None)
    }
}
