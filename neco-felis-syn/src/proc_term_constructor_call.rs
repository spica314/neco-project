use crate::{
    Parse, ParseError, Phase, PhaseParse, ProcTerm, ProcTermNumber, ProcTermVariable,
    token::{Token, TokenKeyword, TokenOperator, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcTermConstructorCall<P: Phase> {
    pub type_name: TokenVariable,
    pub double_colon: TokenOperator,
    pub method: TokenKeyword, // Changed from TokenVariable to TokenKeyword for #method_name
    pub args: Vec<ProcTerm<P>>,
    pub ext: P::ProcTermConstructorCallExt,
}

impl Parse for ProcTermConstructorCall<PhaseParse> {
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

        // Parse method name (now expects #method_name as a keyword)
        // Check if next token is a keyword (starts with #)
        if k >= tokens.len() {
            return Err(ParseError::Unknown("expected #method_name after '::'"));
        }

        let method = if let Token::Keyword(keyword) = &tokens[k] {
            k += 1;
            keyword.clone()
        } else {
            return Err(ParseError::Unknown("expected #method_name after '::'"));
        };

        // Parse arguments (simple terms only to avoid infinite recursion)
        let mut args = Vec::new();
        // For now, we'll only support simple arguments like numbers and variables
        while k < tokens.len() {
            if let Some(arg) = ProcTermNumber::parse(tokens, &mut k)? {
                args.push(ProcTerm::Number(arg));
            } else if let Some(arg) = ProcTermVariable::parse(tokens, &mut k)? {
                args.push(ProcTerm::Variable(arg));
            } else {
                break;
            }
        }

        let proc_term_constructor_call = ProcTermConstructorCall {
            type_name,
            double_colon,
            method,
            args,
            ext: (),
        };

        *i = k;
        Ok(Some(proc_term_constructor_call))
    }
}
