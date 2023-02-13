pub fn parse_from_str<T: crate::parse::Parse>(s: &str) -> Result<Option<T>, ()> {
    use crate::token::{lex, FileId};

    let cs: Vec<_> = s.chars().collect();
    let file_id = FileId(0);
    let tokens = lex(file_id, &cs);
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap();
    let mut i = 0;
    let res = T::parse(&tokens, &mut i);
    assert_eq!(i, tokens.len());
    res
}
