use crate::{Item, Parse, ParseError, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File {
    items: Vec<Item>,
}

impl File {
    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

impl Parse for File {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        let mut items = vec![];
        while let Some(item) = Item::parse(tokens, &mut k)? {
            items.push(item);
        }

        let file = File { items };

        *i = k;
        Ok(Some(file))
    }
}

#[cfg(test)]
mod test {
    use crate::{FileIdGenerator, token::Token};

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_parse_inductive_eq() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let s = std::fs::read_to_string("../testcases/felis/single/inductive_eq.fe").unwrap();
        let tokens = Token::lex(&s, file_id);

        let mut i = 0;
        let file = File::parse(&tokens, &mut i).unwrap().unwrap();

        assert_debug_snapshot!(file);
        assert_eq!(i, tokens.len());
    }

    #[test]
    fn test_parse_inductive_nat() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let s = std::fs::read_to_string("../testcases/felis/single/inductive_nat.fe").unwrap();
        let tokens = Token::lex(&s, file_id);

        let mut i = 0;
        let file = File::parse(&tokens, &mut i).unwrap().unwrap();

        assert_debug_snapshot!(file);
        assert_eq!(i, tokens.len());
    }

    #[test]
    fn test_parse_eq_and_nat() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let s = std::fs::read_to_string("../testcases/felis/single/eq_and_nat.fe").unwrap();
        let tokens = Token::lex(&s, file_id);

        let mut i = 0;
        let file = File::parse(&tokens, &mut i).unwrap().unwrap();

        assert_debug_snapshot!(file);
        assert_eq!(i, tokens.len());
    }
}
