#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Ident(TokenIdent),
    Keyword(TokenKeyword),
    LParen(TokenLParen),
    RParen(TokenRParen),
    Colon(TokenColon),
    Camma(TokenCamma),
    LBrace(TokenLBrace),
    RBrace(TokenRBrace),
    Arrow(TokenArrow),
    Arrow2(TokenArrow2),
    Eq(TokenEq),
    Semicolon(TokenSemicolon),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilePos {
    r: usize,
    c: usize,
}

impl FilePos {
    pub fn new(r: usize, c: usize) -> FilePos {
        FilePos { r, c }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    file_id: FileId,
    begin: FilePos,
    end: FilePos,
}

impl Span {
    pub fn new(file_id: FileId, begin: FilePos, end: FilePos) -> Span {
        Span {
            file_id,
            begin,
            end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenIdent {
    span: Span,
    ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenKeyword {
    span: Span,
    keyword: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenLParen {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenRParen {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenLBrace {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenRBrace {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenColon {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenSemicolon {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenCamma {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenEq {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenArrow {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenArrow2 {
    span: Span,
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn lex(file_id: FileId, chars: &[char]) -> Result<Vec<Token>, ()> {
    let mut r = 1;
    let mut c = 1;
    let mut i = 0;
    let mut res = vec![];
    while i < chars.len() {
        // スペースは無視
        while i < chars.len() && chars[i] == ' ' {
            i += 1;
            c += 1;
        }
        // 改行
        if chars[i] == '\n' {
            i += 1;
            r += 1;
            c = 1;
            continue;
        }
        if chars[i] == '#' {
            let begin = FilePos::new(r, c);
            let mut keyword = String::new();
            i += 1;
            c += 1;
            while i < chars.len() && is_ident_char(chars[i]) {
                keyword.push(chars[i]);
                i += 1;
                c += 1;
            }
            let end = FilePos::new(r, c);
            res.push(Token::Keyword(TokenKeyword {
                span: Span::new(file_id, begin, end),
                keyword,
            }));
            continue;
        }
        if is_ident_char(chars[i]) {
            let begin = FilePos::new(r, c);
            let mut ident = String::new();
            while i < chars.len() && is_ident_char(chars[i]) {
                ident.push(chars[i]);
                i += 1;
                c += 1;
            }
            let end = FilePos::new(r, c);
            res.push(Token::Ident(TokenIdent {
                span: Span::new(file_id, begin, end),
                ident,
            }));
            continue;
        }
        if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '>' {
            let begin = FilePos::new(r, c);
            i += 2;
            c += 2;
            let end = FilePos::new(r, c);
            res.push(Token::Arrow(TokenArrow {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if i + 1 < chars.len() && chars[i] == '=' && chars[i + 1] == '>' {
            let begin = FilePos::new(r, c);
            i += 2;
            c += 2;
            let end = FilePos::new(r, c);
            res.push(Token::Arrow2(TokenArrow2 {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '(' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::LParen(TokenLParen {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ')' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::RParen(TokenRParen {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '{' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::LBrace(TokenLBrace {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '}' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::RBrace(TokenRBrace {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ':' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::Colon(TokenColon {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ',' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::Camma(TokenCamma {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ';' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::Camma(TokenCamma {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '=' {
            let begin = FilePos::new(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new(r, c);
            res.push(Token::Eq(TokenEq {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        panic!("unknown character '{}'", chars[i]);
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn felis_syn_lex_test_1() {
        let s = std::fs::read_to_string("../../library/wip/prop2.fe").unwrap();
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs).unwrap();
        eprintln!("tokens = {tokens:#?}");
        assert_eq!(tokens.len(), 122);
    }
}
