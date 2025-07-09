use crate::{
    Parse, ParseError, Phase, PhaseParse, Statements,
    token::{Token, TokenBraceL, TokenBraceR, TokenKeyword},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermIf<P: Phase> {
    pub keyword_if: TokenKeyword,
    pub condition: Box<Statements<P>>,
    pub brace_l: TokenBraceL,
    pub then_body: Box<Statements<P>>,
    pub brace_r: TokenBraceR,
    pub else_clause: Option<TermIfElse<P>>,
    pub ext: P::TermIfExt,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermIfElse<P: Phase> {
    pub keyword_else: TokenKeyword,
    pub brace_l: TokenBraceL,
    pub else_body: Box<Statements<P>>,
    pub brace_r: TokenBraceR,
}

impl<P: Phase> TermIf<P> {
    /// Get the condition expression
    pub fn condition(&self) -> &Statements<P> {
        &self.condition
    }

    /// Get the then body expression
    pub fn then_body(&self) -> &Statements<P> {
        &self.then_body
    }

    /// Get the else clause if present
    pub fn else_clause(&self) -> Option<&TermIfElse<P>> {
        self.else_clause.as_ref()
    }
}

impl<P: Phase> TermIfElse<P> {
    /// Get the else body expression
    pub fn else_body(&self) -> &Statements<P> {
        &self.else_body
    }
}

impl Parse for TermIf<PhaseParse> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        let mut k = *i;

        // Parse #if keyword
        let Some(keyword_if) = TokenKeyword::parse_keyword(tokens, &mut k, "if")? else {
            return Ok(None);
        };

        // Parse condition
        let Some(condition) = Statements::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown(
                "expected condition expression after #if",
            ));
        };

        // Parse opening brace
        let Some(brace_l) = TokenBraceL::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected { after if condition"));
        };

        // Parse then body
        let Some(then_body) = Statements::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected expression in if body"));
        };

        // Parse closing brace
        let Some(brace_r) = TokenBraceR::parse(tokens, &mut k)? else {
            return Err(ParseError::Unknown("expected } to close if body"));
        };

        // Parse optional else clause
        let else_clause =
            if let Some(keyword_else) = TokenKeyword::parse_keyword(tokens, &mut k, "else")? {
                // Parse else opening brace
                let Some(else_brace_l) = TokenBraceL::parse(tokens, &mut k)? else {
                    return Err(ParseError::Unknown("expected { after #else"));
                };

                // Parse else body
                let Some(else_body) = Statements::parse(tokens, &mut k)? else {
                    return Err(ParseError::Unknown("expected expression in else body"));
                };

                // Parse else closing brace
                let Some(else_brace_r) = TokenBraceR::parse(tokens, &mut k)? else {
                    return Err(ParseError::Unknown("expected } to close else body"));
                };

                Some(TermIfElse {
                    keyword_else,
                    brace_l: else_brace_l,
                    else_body: Box::new(else_body),
                    brace_r: else_brace_r,
                })
            } else {
                None
            };

        let term_if = TermIf {
            keyword_if,
            condition: Box::new(condition),
            brace_l,
            then_body: Box::new(then_body),
            brace_r,
            else_clause,
            ext: (),
        };

        *i = k;
        Ok(Some(term_if))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FileIdGenerator, Token};

    #[test]
    fn test_parse_if_without_else() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();

        // Now test the full if statement
        let source = "#if __u64_eq 0u64 0u64 { error_code = 42u64; } ;";
        let tokens = Token::lex(source, file_id);

        println!("Tokens: {tokens:?}");

        let mut i = 0;
        let result = TermIf::parse(&tokens, &mut i);
        match &result {
            Ok(Some(_)) => println!("Parsing succeeded"),
            Ok(None) => println!("No if expression found"),
            Err(e) => println!("Parsing error: {e:?}"),
        }
        let result = result.unwrap();
        assert!(result.is_some());

        let if_expr = result.unwrap();
        assert!(if_expr.else_clause.is_none());
    }

    #[test]
    fn test_parse_if_with_else() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let source = "#if __u64_eq 0u64 1u64 { error_code = 1u64; } #else { error_code = 42u64; };";
        let tokens = Token::lex(source, file_id);

        let mut i = 0;
        let result = TermIf::parse(&tokens, &mut i).unwrap();
        assert!(result.is_some());

        let if_expr = result.unwrap();
        assert!(if_expr.else_clause.is_some());
    }

    #[test]
    fn test_parse_proc_with_if() {
        use crate::ItemProc;

        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let source = "#proc main : () -> () { #if __u64_eq 0u64 0u64 { error_code = 42u64; }; }";
        let tokens = Token::lex(source, file_id);

        println!("Proc with if tokens: {tokens:?}");

        let mut i = 0;
        let result = ItemProc::parse(&tokens, &mut i);
        match &result {
            Ok(Some(proc)) => {
                println!("Successfully parsed proc: {}", proc.name.s());
            }
            Ok(None) => {
                println!("No proc found");
            }
            Err(e) => {
                println!("Proc parsing error: {e:?}");
            }
        }

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_full_if_file() {
        use crate::File;

        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let source = std::fs::read_to_string("../testcases/felis/single/if_1.fe").unwrap();
        let tokens = Token::lex(&source, file_id);

        println!("Total tokens in if_1.fe: {}", tokens.len());

        let mut i = 0;
        let result = File::parse(&tokens, &mut i);
        match &result {
            Ok(Some(file)) => {
                println!("Successfully parsed file with {} items", file.items().len());
                for (idx, item) in file.items().iter().enumerate() {
                    println!(
                        "Item {}: {:?}",
                        idx,
                        match item {
                            crate::Item::Entrypoint(_) => "Entrypoint",
                            crate::Item::UseBuiltin(_) => "UseBuiltin",
                            crate::Item::Proc(_) => "Proc",
                            crate::Item::Array(_) => "Array",
                            _ => "Other",
                        }
                    );
                }
            }
            Ok(None) => {
                println!("No file found");
            }
            Err(e) => {
                println!("File parsing error: {e:?}");
            }
        }

        assert!(result.is_ok());
        let file = result.unwrap();
        assert!(file.is_some());
        assert_eq!(file.unwrap().items().len(), 4); // Should have entrypoint + 2 use_builtin + proc
    }
}
