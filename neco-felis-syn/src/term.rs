use crate::{
    ItemStruct, Parse, ParseError, Phase, PhaseParse, TermApply, TermArrowDep, TermArrowNodep,
    TermAssign, TermConstructorCall, TermFieldAccess, TermFieldAssign, TermLet, TermLetMut,
    TermMatch, TermNumber, TermParen, TermUnit, TermVariable, token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term<P: Phase> {
    Paren(TermParen<P>),
    ArrowNodep(TermArrowNodep<P>),
    ArrowDep(TermArrowDep<P>),
    Apply(TermApply<P>),
    Variable(TermVariable<P>),
    Match(TermMatch<P>),
    Unit(TermUnit<P>),
    Number(TermNumber<P>),
    Let(TermLet<P>),
    LetMut(TermLetMut<P>),
    Assign(TermAssign<P>),
    FieldAccess(TermFieldAccess<P>),
    FieldAssign(TermFieldAssign<P>),
    ConstructorCall(TermConstructorCall<P>),
    Struct(ItemStruct<P>),
}

impl Parse for Term<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(term_let_mut) = TermLetMut::parse(tokens, i)? {
            return Ok(Some(Term::LetMut(term_let_mut)));
        }

        if let Some(term_let) = TermLet::parse(tokens, i)? {
            return Ok(Some(Term::Let(term_let)));
        }

        if let Some(term_field_assign) = TermFieldAssign::parse(tokens, i)? {
            return Ok(Some(Term::FieldAssign(term_field_assign)));
        }

        if let Some(term_assign) = TermAssign::parse(tokens, i)? {
            return Ok(Some(Term::Assign(term_assign)));
        }

        if let Some(term_match) = TermMatch::parse(tokens, i)? {
            return Ok(Some(Term::Match(term_match)));
        }

        if let Some(term_arrow_dep) = TermArrowDep::parse(tokens, i)? {
            return Ok(Some(Term::ArrowDep(term_arrow_dep)));
        }

        if let Some(term_arrow_nodep) = TermArrowNodep::parse(tokens, i)? {
            return Ok(Some(Term::ArrowNodep(term_arrow_nodep)));
        }

        if let Some(term_constructor_call) = TermConstructorCall::parse(tokens, i)? {
            return Ok(Some(Term::ConstructorCall(term_constructor_call)));
        }

        if let Some(term_field_access) = TermFieldAccess::parse(tokens, i)? {
            return Ok(Some(Term::FieldAccess(term_field_access)));
        }

        if let Some(term_apply) = TermApply::parse(tokens, i)? {
            return Ok(Some(Term::Apply(term_apply)));
        }

        if let Some(item_struct) = ItemStruct::parse(tokens, i)? {
            return Ok(Some(Term::Struct(item_struct)));
        }

        if let Some(term_variable) = TermVariable::parse(tokens, i)? {
            return Ok(Some(Term::Variable(term_variable)));
        }

        if let Some(term_number) = TermNumber::parse(tokens, i)? {
            return Ok(Some(Term::Number(term_number)));
        }

        if let Some(term_unit) = TermUnit::parse(tokens, i)? {
            return Ok(Some(Term::Unit(term_unit)));
        }

        if let Some(term_paren) = TermParen::parse(tokens, i)? {
            return Ok(Some(Term::Paren(term_paren)));
        }

        Ok(None)
    }
}
