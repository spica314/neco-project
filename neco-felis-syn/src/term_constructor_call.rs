use crate::{
    Parse, ParseError, Phase, PhaseParse, Term,
    token::{Token, TokenOperator, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermConstructorCall<P: Phase> {
    pub type_name: TokenVariable,
    pub double_colon: TokenOperator,
    pub method: TokenVariable,
    pub args: Vec<Term<P>>,
    pub ext: P::TermConstructorCallExt,
}

impl<P: Phase> TermConstructorCall<P> {
    pub fn type_name(&self) -> &str {
        self.type_name.s()
    }

    pub fn method_name(&self) -> &str {
        self.method.s()
    }
}

impl Parse for TermConstructorCall<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse type name
        let Some(type_name) = TokenVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Parse "::" operator
        let Some(double_colon) = TokenOperator::parse_operator(tokens, &mut k, "::")? else {
            return Ok(None);
        };

        // Parse method name
        let Some(method) = TokenVariable::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected method name after '::'"));
        };

        // Parse arguments (simple terms only to avoid infinite recursion)
        let mut args = Vec::new();
        // For now, we'll only support simple arguments like numbers and variables
        while k < tokens.len() {
            if let Some(arg) = crate::TermNumber::parse(tokens, &mut k)? {
                args.push(Term::Number(arg));
            } else if let Some(arg) = crate::TermVariable::parse(tokens, &mut k)? {
                args.push(Term::Variable(arg));
            } else {
                break;
            }
        }

        let term_constructor_call = TermConstructorCall {
            type_name,
            double_colon,
            method,
            args,
            ext: (),
        };

        *i = k;
        Ok(Some(term_constructor_call))
    }
}
