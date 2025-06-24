use crate::{
    Parse, ParseError, Term,
    token::{Token, TokenOperator, TokenVariable},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pattern {
    Variable(TokenVariable),
    Constructor(TokenVariable, Vec<TokenVariable>),
}

impl Parse for Pattern {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse first variable (constructor or simple variable)
        let Some(first) = TokenVariable::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Check if there are arguments (making it a constructor pattern)
        let mut args = Vec::new();
        while let Some(arg) = TokenVariable::parse(tokens, &mut k)? {
            args.push(arg);

            // Check if next token is => to stop parsing arguments
            let mut test_k = k;
            if TokenOperator::parse_operator(tokens, &mut test_k, "=>")
                .unwrap_or(None)
                .is_some()
            {
                break;
            }
        }

        let pattern = if args.is_empty() {
            Pattern::Variable(first)
        } else {
            Pattern::Constructor(first, args)
        };

        *i = k;
        Ok(Some(pattern))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermMatchBranch {
    pub pattern: Pattern,
    pub arrow: TokenOperator,
    pub body: Box<Term>,
}

impl Parse for TermMatchBranch {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse pattern
        let Some(pattern) = Pattern::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        // Parse => operator
        let Some(arrow) = TokenOperator::parse_operator(tokens, &mut k, "=>")? else {
            return Err(ParseError::Unknown("expected => after match pattern"));
        };

        // Parse body term
        let Some(body) = Term::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected term after =>"));
        };

        let branch = TermMatchBranch {
            pattern,
            arrow,
            body: Box::new(body),
        };

        *i = k;
        Ok(Some(branch))
    }
}
