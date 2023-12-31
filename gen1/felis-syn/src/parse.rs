use crate::token::Token;

pub trait Parse: Sized {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()>;
}
