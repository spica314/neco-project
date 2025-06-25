use crate::{
    Parse, ParseError, Phase, PhaseParse, Term,
    token::{Token, TokenBraceL, TokenBraceR, TokenColon, TokenKeyword, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemDefinition<P: Phase> {
    keyword_definition: TokenKeyword,
    name: TokenVariable,
    colon: TokenColon,
    type_: Box<Term<P>>,
    brace_l: TokenBraceL,
    body: Box<Term<P>>,
    brace_r: TokenBraceR,
    #[allow(dead_code)]
    ext: P::ItemDefinitionExt,
}

impl<P: Phase> ItemDefinition<P> {
    pub fn name(&self) -> &TokenVariable {
        &self.name
    }

    pub fn type_(&self) -> &Term<P> {
        &self.type_
    }

    pub fn body(&self) -> &Term<P> {
        &self.body
    }
}

impl Parse for ItemDefinition<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse #definition keyword
        let Some(keyword_definition) = TokenKeyword::parse_keyword(tokens, &mut k, "definition")?
        else {
            return Ok(None);
        };

        // Parse name
        let Some(name) = TokenVariable::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected name after #definition"));
        };

        // Parse colon
        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected : after definition name"));
        };

        // Parse type
        let Some(type_) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected type after :"));
        };

        // Parse opening brace
        let Some(brace_l) = TokenBraceL::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected { after definition type"));
        };

        // Parse body term
        let Some(body) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected definition body"));
        };

        // Parse closing brace
        let Some(brace_r) = TokenBraceR::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected } to close definition body"));
        };

        let definition = ItemDefinition {
            keyword_definition,
            name,
            colon,
            type_: Box::new(type_),
            brace_l,
            body: Box::new(body),
            brace_r,
            ext: (),
        };

        *i = k;
        Ok(Some(definition))
    }
}
