use crate::Parser;

pub fn parse_from_str<T: crate::parse::Parse>(s: &str) -> Result<Option<T>, ()> {
    let mut parser = Parser::new();
    let (res, _) = parser.parse::<T>(s)?;
    Ok(res)
}
