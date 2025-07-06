use crate::{Item, Parse, ParseError, Phase, PhaseParse, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File<P: Phase> {
    pub items: Vec<Item<P>>,
    pub ext: P::FileExt,
}

impl<P: Phase> File<P> {
    pub fn items(&self) -> &[Item<P>] {
        &self.items
    }
}

impl Parse for File<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        let mut items = vec![];
        while let Some(item) = Item::parse(tokens, &mut k)? {
            items.push(item);
        }

        let file = File { items, ext: () };

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

    #[test]
    fn test_parse_exit_42() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let s = std::fs::read_to_string("../testcases/felis/single/exit_42.fe").unwrap();
        let tokens = Token::lex(&s, file_id);

        let mut i = 0;
        let file = File::parse(&tokens, &mut i).unwrap().unwrap();

        assert_debug_snapshot!(file);
        assert_eq!(i, tokens.len());
    }

    #[test]
    fn test_parse_let_mut() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let s = std::fs::read_to_string("../testcases/felis/single/let_mut.fe").unwrap();
        let tokens = Token::lex(&s, file_id);

        let mut i = 0;
        let result = File::parse(&tokens, &mut i);

        match result {
            Ok(Some(file)) => {
                println!("Parsed successfully up to token {i} of {}", tokens.len());
                if i < tokens.len() {
                    println!("Remaining tokens: {:?}", &tokens[i..]);
                }
                assert_debug_snapshot!(file);
                assert_eq!(i, tokens.len());
            }
            Ok(None) => {
                println!("Parsing returned None at token {i} of {}", tokens.len());
                println!(
                    "Tokens around position: {:?}",
                    &tokens[i.saturating_sub(3)..tokens.len().min(i + 3)]
                );
                panic!("Parsing returned None");
            }
            Err(e) => {
                println!("Parse error at token {i} of {}: {:?}", tokens.len(), e);
                println!(
                    "Tokens around position: {:?}",
                    &tokens[i.saturating_sub(3)..tokens.len().min(i + 3)]
                );
                panic!("Parse error: {e:?}");
            }
        }
    }
}
