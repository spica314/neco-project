use parse::Parse;
use syn_file::SynFile;
use token_id::TokenIdGenerator;

use crate::token::{lex, FileId};

pub mod parse;
pub mod syn_expr;
pub mod syn_file;
pub mod syn_fn_def;
pub mod syn_ident;
pub mod syn_theorem_def;
pub mod syn_type;
pub mod syn_type_def;
pub mod syn_typed_arg;
pub mod test_utils;
pub mod to_felis_string;
pub mod token;
pub mod token_id;

pub struct Parser {
    token_id_generator: TokenIdGenerator,
    next_file_id: usize,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            token_id_generator: TokenIdGenerator::new(),
            next_file_id: 0,
        }
    }
    pub fn parse_file(&mut self, s: &str) -> Result<SynFile, ()> {
        let res = self.parse::<SynFile>(s)?;
        Ok(res.unwrap())
    }
    pub fn parse<T: Parse>(&mut self, s: &str) -> Result<Option<T>, ()> {
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(self.next_file_id);
        self.next_file_id += 1;
        let tokens = lex(&mut self.token_id_generator, file_id, &cs).unwrap();
        let mut i = 0;
        let res = T::parse(&tokens, &mut i);
        assert_eq!(i, tokens.len());
        res
    }
}
