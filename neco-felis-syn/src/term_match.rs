use crate::{
    Parse, ParseError, TermMatchBranch,
    token::{Token, TokenBraceL, TokenBraceR, TokenKeyword, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermMatch {
    pub keyword_match: TokenKeyword,
    pub scrutinee: TokenVariable,
    pub brace_l: TokenBraceL,
    pub branches: Vec<TermMatchBranch>,
    pub brace_r: TokenBraceR,
}

impl Parse for TermMatch {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse #match keyword
        let Some(keyword_match) = TokenKeyword::parse_keyword(tokens, &mut k, "match")? else {
            return Ok(None);
        };

        // Parse scrutinee (the expression being matched)
        let Some(scrutinee) = TokenVariable::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected variable after #match"));
        };

        // Parse opening brace
        let Some(brace_l) = TokenBraceL::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected { after match scrutinee"));
        };

        // Parse branches
        let mut branches = vec![];
        while let Some(branch) = TermMatchBranch::parse(tokens, &mut k)? {
            branches.push(branch);

            // Check for optional comma
            if let Some(Token::Comma(_)) = tokens.get(k) {
                k += 1;
            }
        }

        if branches.is_empty() {
            return Err(ParseError::Unknown(
                "match expression must have at least one branch",
            ));
        }

        // Parse closing brace
        let Some(brace_r) = TokenBraceR::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected } to close match expression"));
        };

        let term_match = TermMatch {
            keyword_match,
            scrutinee,
            brace_l,
            branches,
            brace_r,
        };

        *i = k;
        Ok(Some(term_match))
    }
}
