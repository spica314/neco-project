use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseError {
    Unknown(&'static str),
}

pub trait Parse: Sized {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError>;
}
