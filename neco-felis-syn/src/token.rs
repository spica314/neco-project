use crate::{FileId, Pos};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenKeyword {
    pos: Pos,
    s: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenVariable {
    pos: Pos,
    s: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenOperator {
    pos: Pos,
    s: String,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenParenR {
    pos: Pos,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenBraceR {
    pos: Pos,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenComma {
    pos: Pos,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenColon {
    pos: Pos,
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
