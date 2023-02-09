use crate::parse::Parse;

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

// Todo: crate外から.0にアクセスさせないようにする
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilePos {
    None,
    Pos { r: usize, c: usize },
}

impl FilePos {
    pub fn new() -> FilePos {
        FilePos::None
    }
    pub fn new_with_pos(r: usize, c: usize) -> FilePos {
        FilePos::Pos { r, c }
    }
}

impl Default for FilePos {
    fn default() -> Self {
        Self::new()
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
    pub span: Span,
    pub ident: String,
}

impl TokenIdent {
    pub fn as_str(&self) -> &str {
        &self.ident
    }
}

impl Parse for TokenIdent {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<TokenIdent>, ()> {
        if *i >= tokens.len() {
            return Ok(None);
        }
        if let Token::Ident(token_ident) = tokens[*i].clone() {
            *i += 1;
            Ok(Some(token_ident))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenKeyword {
    pub span: Span,
    pub keyword: String,
}

impl Parse for TokenKeyword {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<TokenKeyword>, ()> {
        if *i >= tokens.len() {
            return Ok(None);
        }
        if let Token::Keyword(token_keyword) = tokens[*i].clone() {
            *i += 1;
            Ok(Some(token_keyword))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenLParen {
    span: Span,
}

impl Parse for TokenLParen {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Ok(None);
        }
        if let Token::LParen(lparen) = &tokens[*i] {
            *i += 1;
            return Ok(Some(lparen.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenRParen {
    span: Span,
}

impl Parse for TokenRParen {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }
        if let Token::RParen(rparen) = &tokens[*i] {
            *i += 1;
            return Ok(Some(rparen.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenLBrace {
    span: Span,
}

impl Parse for TokenLBrace {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }
        if let Token::LBrace(lbrace) = &tokens[*i] {
            *i += 1;
            return Ok(Some(lbrace.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenRBrace {
    span: Span,
}

impl Parse for TokenRBrace {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }
        if let Token::RBrace(rbrace) = &tokens[*i] {
            *i += 1;
            return Ok(Some(rbrace.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenColon {
    span: Span,
}

impl Parse for TokenColon {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }
        if let Token::Colon(colon) = &tokens[*i] {
            *i += 1;
            return Ok(Some(colon.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenSemicolon {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenCamma {
    span: Span,
}

impl Parse for TokenCamma {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }
        if let Token::Camma(camma) = &tokens[*i] {
            *i += 1;
            return Ok(Some(camma.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenEq {
    span: Span,
}

impl Parse for TokenEq {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }
        if let Token::Eq(eq) = &tokens[*i] {
            *i += 1;
            return Ok(Some(eq.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenArrow {
    span: Span,
}

impl Parse for TokenArrow {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Ok(None);
        }
        if let Token::Arrow(arrow) = &tokens[*i] {
            *i += 1;
            return Ok(Some(arrow.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenArrow2 {
    span: Span,
}

impl Parse for TokenArrow2 {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }

        if let Token::Arrow2(arrow2) = &tokens[*i] {
            *i += 1;
            return Ok(Some(arrow2.clone()));
        }

        Ok(None)
    }
}

fn is_ident_head_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "_".contains(c)
}

fn is_ident_tail_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "-_".contains(c)
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
            let begin = FilePos::new_with_pos(r, c);
            let mut keyword = String::new();
            keyword.push('#');
            i += 1;
            c += 1;
            while i < chars.len() && is_ident_tail_char(chars[i]) {
                keyword.push(chars[i]);
                i += 1;
                c += 1;
            }
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::Keyword(TokenKeyword {
                span: Span::new(file_id, begin, end),
                keyword,
            }));
            continue;
        }
        if is_ident_head_char(chars[i]) {
            let begin = FilePos::new_with_pos(r, c);
            let mut ident = String::new();
            while i < chars.len() && is_ident_tail_char(chars[i]) {
                ident.push(chars[i]);
                i += 1;
                c += 1;
            }
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::Ident(TokenIdent {
                span: Span::new(file_id, begin, end),
                ident,
            }));
            continue;
        }
        if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '>' {
            let begin = FilePos::new_with_pos(r, c);
            i += 2;
            c += 2;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::Arrow(TokenArrow {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if i + 1 < chars.len() && chars[i] == '=' && chars[i + 1] == '>' {
            let begin = FilePos::new_with_pos(r, c);
            i += 2;
            c += 2;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::Arrow2(TokenArrow2 {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '(' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::LParen(TokenLParen {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ')' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::RParen(TokenRParen {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '{' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::LBrace(TokenLBrace {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '}' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::RBrace(TokenRBrace {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ':' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::Colon(TokenColon {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ',' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::Camma(TokenCamma {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == ';' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            res.push(Token::Camma(TokenCamma {
                span: Span::new(file_id, begin, end),
            }));
            continue;
        }
        if chars[i] == '=' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
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
        assert_eq!(tokens.len(), 120);
    }

    #[test]
    fn felis_syn_lex_test_2() {
        let s = "test test_1 test-1 __test";
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs).unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(tokens.iter().all(|t| matches!(t, Token::Ident(_))));
    }
}
