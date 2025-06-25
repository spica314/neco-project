use crate::{FileId, Parse, ParseError, Pos};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenKeyword {
    pos: Pos,
    s: String,
}

impl TokenKeyword {
    pub fn parse_keyword(
        tokens: &[Token],
        i: &mut usize,
        s: &str,
    ) -> Result<Option<Self>, ParseError> {
        if *i >= tokens.len() {
            return Ok(None);
        }

        if let Token::Keyword(token_keyword) = &tokens[*i]
            && token_keyword.s == s
        {
            *i += 1;
            return Ok(Some(token_keyword.clone()));
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenVariable {
    pos: Pos,
    s: String,
}

impl TokenVariable {
    pub fn s(&self) -> &str {
        &self.s
    }
}

impl Parse for TokenVariable {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Token::Variable(variable) = &tokens[*i] {
            *i += 1;
            Ok(Some(variable.clone()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenOperator {
    pos: Pos,
    s: String,
}

impl TokenOperator {
    pub fn parse_operator(
        tokens: &[Token],
        i: &mut usize,
        s: &str,
    ) -> Result<Option<Self>, ParseError> {
        if let Token::Operator(token_operator) = &tokens[*i]
            && token_operator.s == s
        {
            *i += 1;
            return Ok(Some(token_operator.clone()));
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenNumber {
    pos: Pos,
    s: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenParenL {
    pos: Pos,
}

impl Parse for TokenParenL {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Token::ParenL(paren_l) = &tokens[*i] {
            *i += 1;
            Ok(Some(paren_l.clone()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenParenR {
    pos: Pos,
}

impl Parse for TokenParenR {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Token::ParenR(paren_r) = &tokens[*i] {
            *i += 1;
            Ok(Some(paren_r.clone()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenBracketL {
    pos: Pos,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenBracketR {
    pos: Pos,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenBraceL {
    pos: Pos,
}

impl Parse for TokenBraceL {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Token::BraceL(brace_l) = &tokens[*i] {
            *i += 1;
            Ok(Some(brace_l.clone()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenBraceR {
    pos: Pos,
}

impl Parse for TokenBraceR {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Token::BraceR(brace_r) = &tokens[*i] {
            *i += 1;
            Ok(Some(brace_r.clone()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenComma {
    pos: Pos,
}

impl Parse for TokenComma {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Token::Comma(comma) = &tokens[*i] {
            *i += 1;
            Ok(Some(comma.clone()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenColon {
    pos: Pos,
}

impl Parse for TokenColon {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Token::Colon(colon) = &tokens[*i] {
            *i += 1;
            Ok(Some(colon.clone()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenSemicolon {
    pos: Pos,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Keyword(TokenKeyword),
    Variable(TokenVariable),
    Operator(TokenOperator),
    Number(TokenNumber),
    ParenL(TokenParenL),
    ParenR(TokenParenR),
    BracketL(TokenBracketL),
    BracketR(TokenBracketR),
    BraceL(TokenBraceL),
    BraceR(TokenBraceR),
    Comma(TokenComma),
    Colon(TokenColon),
    Semicolon(TokenSemicolon),
}

fn is_operator_char(c: char) -> bool {
    ['+', '-', '*', '/', '%', '=', '<', '>'].contains(&c)
}

impl Token {
    pub fn lex(s: &str, file_id: FileId) -> Vec<Token> {
        let cs: Vec<_> = s.chars().collect();

        let mut tokens = vec![];
        let mut i = 0;
        let mut line = 1;
        let mut column = 1;

        while i < cs.len() {
            if cs[i] == '\n' {
                i += 1;
                line += 1;
                column = 1;
                continue;
            }

            while cs[i].is_whitespace() {
                i += 1;
                column += 1;
                continue;
            }

            // TokenParenL
            if cs[i] == '(' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::ParenL(TokenParenL {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenParenR
            if cs[i] == ')' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::ParenR(TokenParenR {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenBracketL
            if cs[i] == '[' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::BracketL(TokenBracketL {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenBracketR
            if cs[i] == ']' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::BracketR(TokenBracketR {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenBraceL
            if cs[i] == '{' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::BraceL(TokenBraceL {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenBraceR
            if cs[i] == '}' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::BraceR(TokenBraceR {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenComma
            if cs[i] == ',' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::Comma(TokenComma {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenColon
            if cs[i] == ':' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::Colon(TokenColon {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenSemicolon
            if cs[i] == ';' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;

                let token = Token::Semicolon(TokenSemicolon {
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenKeyword
            if cs[i] == '#' {
                let token_line = line;
                let token_column = column;
                i += 1;
                column += 1;
                let mut buf = String::new();
                while i < cs.len() && cs[i].is_ascii_alphanumeric() {
                    buf.push(cs[i]);
                    i += 1;
                    column += 1;
                }

                let token = Token::Keyword(TokenKeyword {
                    s: buf,
                    pos: Pos::new(file_id, token_line, token_column),
                });

                tokens.push(token);
                continue;
            }

            // TokenNumber
            if cs[i].is_numeric() {
                let token_line = line;
                let token_column = column;
                let mut buf = String::new();
                while i < cs.len()
                    && (cs[i].is_ascii_alphanumeric() || cs[i] == '.' || cs[i] == '_')
                {
                    buf.push(cs[i]);
                    i += 1;
                    column += 1;
                }

                let token = Token::Number(TokenNumber {
                    pos: Pos::new(file_id, token_line, token_column),
                    s: buf,
                });

                tokens.push(token);
                continue;
            }

            // TokenVariable
            if cs[i].is_ascii_alphabetic() || cs[i] == '_' {
                let token_line = line;
                let token_column = column;
                let mut buf = String::new();
                while i < cs.len()
                    && (cs[i].is_ascii_alphanumeric() || cs[i] == '.' || cs[i] == '_')
                {
                    buf.push(cs[i]);
                    i += 1;
                    column += 1;
                }

                let token = Token::Variable(TokenVariable {
                    pos: Pos::new(file_id, token_line, token_column),
                    s: buf,
                });

                tokens.push(token);
                continue;
            }

            // TokenVariable
            if is_operator_char(cs[i]) {
                let token_line = line;
                let token_column = column;
                let mut buf = String::new();
                while i < cs.len() && is_operator_char(cs[i]) {
                    buf.push(cs[i]);
                    i += 1;
                    column += 1;
                }

                let token = Token::Operator(TokenOperator {
                    pos: Pos::new(file_id, token_line, token_column),
                    s: buf,
                });

                tokens.push(token);
                continue;
            }

            panic!("line = {line}, column = {column}");
        }

        tokens
    }
}

#[cfg(test)]
mod test {
    use crate::FileIdGenerator;

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_lex_eq_and_nat() {
        let mut file_id_generator = FileIdGenerator::new();
        let file_id = file_id_generator.generate_file_id();
        let s = std::fs::read_to_string("../testcases/felis/single/eq_and_nat.fe").unwrap();
        let tokens = Token::lex(&s, file_id);

        assert_debug_snapshot!(tokens);
    }
}
