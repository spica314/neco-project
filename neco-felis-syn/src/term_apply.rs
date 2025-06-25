use crate::{Parse, ParseError, Phase, PhaseParse, Term, TermParen, TermVariable, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermApply<P: Phase> {
    f: Box<Term<P>>,
    args: Vec<Term<P>>,
    #[allow(dead_code)]
    ext: P::TermApplyExt,
}

impl<P: Phase> TermApply<P> {
    pub fn f(&self) -> &Term<P> {
        &self.f
    }

    pub fn args(&self) -> &[Term<P>] {
        &self.args
    }
}

impl Parse for TermApply<PhaseParse> {
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
            ext: (),
        };

        *i = k;
        Ok(Some(term_apply))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TermForApplyElem {
    Paren(TermParen<PhaseParse>),
    Variable(TermVariable<PhaseParse>),
}

impl From<TermForApplyElem> for Term<PhaseParse> {
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
