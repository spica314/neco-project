use crate::{
    ItemStruct, Parse, ParseError, Phase, PhaseParse, ProcTermApply, ProcTermConstructorCall,
    ProcTermFieldAccess, ProcTermIf, ProcTermNumber, ProcTermParen, ProcTermUnit, ProcTermVariable,
    token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcTerm<P: Phase> {
    Paren(ProcTermParen<P>),
    Apply(ProcTermApply<P>),
    Variable(ProcTermVariable<P>),
    Unit(ProcTermUnit<P>),
    Number(ProcTermNumber<P>),
    FieldAccess(ProcTermFieldAccess<P>),
    ConstructorCall(ProcTermConstructorCall<P>),
    Struct(ItemStruct<P>),
    If(ProcTermIf<P>),
    Ext(P::ProcTermExt),
}

impl Parse for ProcTerm<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(proc_term_if) = ProcTermIf::parse(tokens, i)? {
            return Ok(Some(ProcTerm::If(proc_term_if)));
        }

        if let Some(proc_term_constructor_call) = ProcTermConstructorCall::parse(tokens, i)? {
            return Ok(Some(ProcTerm::ConstructorCall(proc_term_constructor_call)));
        }

        if let Some(proc_term_field_access) = ProcTermFieldAccess::parse(tokens, i)? {
            return Ok(Some(ProcTerm::FieldAccess(proc_term_field_access)));
        }

        if let Some(proc_term_apply) = ProcTermApply::parse(tokens, i)? {
            return Ok(Some(ProcTerm::Apply(proc_term_apply)));
        }

        if let Some(item_struct) = ItemStruct::parse(tokens, i)? {
            return Ok(Some(ProcTerm::Struct(item_struct)));
        }

        if let Some(proc_term_variable) = ProcTermVariable::parse(tokens, i)? {
            return Ok(Some(ProcTerm::Variable(proc_term_variable)));
        }

        if let Some(proc_term_number) = ProcTermNumber::parse(tokens, i)? {
            return Ok(Some(ProcTerm::Number(proc_term_number)));
        }

        if let Some(proc_term_unit) = ProcTermUnit::parse(tokens, i)? {
            return Ok(Some(ProcTerm::Unit(proc_term_unit)));
        }

        if let Some(proc_term_paren) = ProcTermParen::parse(tokens, i)? {
            return Ok(Some(ProcTerm::Paren(proc_term_paren)));
        }
        Ok(None)
    }
}
