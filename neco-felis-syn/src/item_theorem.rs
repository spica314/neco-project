use crate::{
    Parse, ParseError, Term,
    token::{Token, TokenBraceL, TokenBraceR, TokenColon, TokenKeyword, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemTheorem {
    keyword_theorem: TokenKeyword,
    name: TokenVariable,
    colon: TokenColon,
    type_: Box<Term>,
    brace_l: TokenBraceL,
    body: Box<Term>,
    brace_r: TokenBraceR,
}

impl Parse for ItemTheorem {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse #theorem keyword
        let Some(keyword_theorem) = TokenKeyword::parse_keyword(tokens, &mut k, "theorem")? else {
            return Ok(None);
        };

        // Parse name
        let Some(name) = TokenVariable::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected name after #theorem"));
        };

        // Parse colon
        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected : after theorem name"));
        };

        // Parse type
        let Some(type_) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected type after :"));
        };

        // Parse opening brace
        let Some(brace_l) = TokenBraceL::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected { after theorem type"));
        };

        // Parse body term
        let Some(body) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected theorem body"));
        };

        // Parse closing brace
        let Some(brace_r) = TokenBraceR::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected } after theorem body"));
        };

        let theorem = ItemTheorem {
            keyword_theorem,
            name,
            colon,
            type_: Box::new(type_),
            brace_l,
            body: Box::new(body),
            brace_r,
        };

        *i = k;
        Ok(Some(theorem))
    }
}
