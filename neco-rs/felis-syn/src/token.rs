use crate::{parse::Parse, to_felis_string::ToFelisString};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Ident(TokenIdent),
    Keyword(TokenKeyword),
    String(TokenString),
    LParen(TokenLParen),
    RParen(TokenRParen),
    Colon(TokenColon),
    ColonColon(TokenColonColon),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenIdent {
    pub info: TokenInfo,
    pub ident: String,
}

impl TokenIdent {
    pub fn as_str(&self) -> &str {
        &self.ident
    }
}

impl ToFelisString for TokenIdent {
    fn to_felis_string(&self) -> String {
        self.ident.clone()
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenString {
    pub info: TokenInfo,
    pub string: String,
}

impl ToFelisString for TokenString {
    fn to_felis_string(&self) -> String {
        format!("\"{}\"", self.string)
    }
}

impl Parse for TokenString {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<TokenString>, ()> {
        if *i >= tokens.len() {
            return Ok(None);
        }
        if let Token::String(token_string) = tokens[*i].clone() {
            *i += 1;
            Ok(Some(token_string))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenKeyword {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenLParen {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenRParen {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenLBrace {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenRBrace {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenColon {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenColonColon {
    pub info: TokenInfo,
}

impl Parse for TokenColonColon {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Ok(None);
        }
        if let Token::ColonColon(colon_colon) = &tokens[*i] {
            *i += 1;
            return Ok(Some(colon_colon.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenSemicolon {
    pub info: TokenInfo,
}

impl Parse for TokenSemicolon {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if *i >= tokens.len() {
            return Err(());
        }
        if let Token::Semicolon(semicolon) = &tokens[*i] {
            *i += 1;
            return Ok(Some(semicolon.clone()));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenCamma {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenEq {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenArrow {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenArrow2 {
    pub info: TokenInfo,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenInfo {
    pub span: Span,
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

        // Handle newline character
        if chars[i] == '\n' {
            i += 1;
            r += 1;
            c = 1;
            continue;
        }

        // Handle keyword tokens
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
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Keyword(TokenKeyword { info, keyword });
            res.push(token);
            continue;
        }

        // Handle string tokens
        if chars[i] == '"' {
            let begin = FilePos::new_with_pos(r, c);
            let mut string = String::new();
            i += 1;
            c += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    string.push('\\');
                    i += 1;
                    c += 1;
                    if i >= chars.len() {
                        return Err(());
                    }
                }
                string.push(chars[i]);
                i += 1;
                c += 1;
            }
            if i >= chars.len() {
                return Err(());
            }
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::String(TokenString { info, string });
            res.push(token);
            continue;
        }

        // Handle identifier tokens
        if is_ident_head_char(chars[i]) {
            let begin = FilePos::new_with_pos(r, c);
            let mut ident = String::new();
            while i < chars.len() && is_ident_tail_char(chars[i]) {
                ident.push(chars[i]);
                i += 1;
                c += 1;
            }
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Ident(TokenIdent { info, ident });
            res.push(token);
            continue;
        }

        // Handle double colon tokens
        if i + 1 < chars.len() && chars[i] == ':' && chars[i + 1] == ':' {
            let begin = FilePos::new_with_pos(r, c);
            i += 2;
            c += 2;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::ColonColon(TokenColonColon { info });
            res.push(token);
            continue;
        }

        // Handle arrow tokens
        if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '>' {
            let begin = FilePos::new_with_pos(r, c);
            i += 2;
            c += 2;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Arrow(TokenArrow { info });
            res.push(token);
            continue;
        }

        // Handle arrow2 tokens
        if i + 1 < chars.len() && chars[i] == '=' && chars[i + 1] == '>' {
            let begin = FilePos::new_with_pos(r, c);
            i += 2;
            c += 2;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Arrow2(TokenArrow2 { info });
            res.push(token);
            continue;
        }

        // Handle lparen tokens
        if chars[i] == '(' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::LParen(TokenLParen { info });
            res.push(token);
            continue;
        }

        // Handle rparen tokens
        if chars[i] == ')' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::RParen(TokenRParen { info });
            res.push(token);
            continue;
        }

        // Handle lbracket tokens
        if chars[i] == '{' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::LBrace(TokenLBrace { info });
            res.push(token);
            continue;
        }

        // Handle rbracket tokens
        if chars[i] == '}' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::RBrace(TokenRBrace { info });
            res.push(token);
            continue;
        }

        // Handle colon tokens
        if chars[i] == ':' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Colon(TokenColon { info });
            res.push(token);
            continue;
        }

        // Handle camma tokens
        if chars[i] == ',' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Camma(TokenCamma { info });
            res.push(token);
            continue;
        }

        // Handle semicolon tokens
        if chars[i] == ';' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Semicolon(TokenSemicolon { info });
            res.push(token);
            continue;
        }

        // Handle eq tokens
        if chars[i] == '=' {
            let begin = FilePos::new_with_pos(r, c);
            i += 1;
            c += 1;
            let end = FilePos::new_with_pos(r, c);
            let info = TokenInfo {
                span: Span::new(file_id, begin, end),
            };
            let token = Token::Eq(TokenEq { info });
            res.push(token);
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
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs).unwrap();
        assert_eq!(tokens.len(), 155);
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
