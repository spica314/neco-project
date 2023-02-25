use neco_table::Table;
use parse::Parse;
use syn_file::SynFile;
use token::TokenInfo;

use crate::token::{lex, FileId};

pub mod parse;
pub mod syn_expr;
pub mod syn_file;
pub mod syn_fn_def;
pub mod syn_theorem_def;
pub mod syn_type;
pub mod syn_type_def;
pub mod syn_typed_arg;
pub mod test_utils;
pub mod to_felis_string;
pub mod token;

pub struct Parser {
    next_file_id: usize,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Parser {
        Parser { next_file_id: 0 }
    }
    pub fn parse_file(&mut self, s: &str) -> Result<(SynFile, Table<TokenInfo>), ()> {
        let (syn_file, table) = self.parse::<SynFile>(s)?;
        Ok((syn_file.unwrap(), table))
    }
    pub fn parse<T: Parse>(&mut self, s: &str) -> Result<(Option<T>, Table<TokenInfo>), ()> {
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(self.next_file_id);
        self.next_file_id += 1;
        let (tokens, table) = lex(file_id, &cs).unwrap();
        let mut i = 0;
        let res = T::parse(&tokens, &mut i)?;
        assert_eq!(i, tokens.len());
        Ok((res, table))
    }
}
