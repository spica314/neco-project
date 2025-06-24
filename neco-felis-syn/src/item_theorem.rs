use crate::{Parse, ParseError, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemTheorem {}

impl Parse for ItemTheorem {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if *i >= tokens.len() {
            return Ok(None);
        }

        todo!()
    }
}
