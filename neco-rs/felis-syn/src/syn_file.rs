use crate::{
    parse::Parse, syn_fn_def::SynFnDef, syn_theorem_def::SynTheoremDef, syn_type_def::SynTypeDef,
    token::Token,
};

pub struct SynFile {
    pub items: Vec<SynFileItem>,
}

impl Parse for SynFile {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut items = vec![];
        while let Some(item) = SynFileItem::parse(tokens, &mut k)? {
            items.push(item);
        }

        *i = k;
        Ok(Some(SynFile { items }))
    }
}

pub enum SynFileItem {
    TypeDef(SynTypeDef),
    FnDef(SynFnDef),
    TheoremDef(SynTheoremDef),
}

impl Parse for SynFileItem {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(type_def) = SynTypeDef::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynFileItem::TypeDef(type_def)));
        }

        if let Some(fn_def) = SynFnDef::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynFileItem::FnDef(fn_def)));
        }

        if let Some(theorem_def) = SynTheoremDef::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynFileItem::TheoremDef(theorem_def)));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use crate::token::{lex, FileId};

    use super::*;

    #[test]
    fn felis_syn_file_parse_test_1() {
        let s = std::fs::read_to_string("../../library/wip/prop2.fe").unwrap();
        let cs: Vec<_> = s.chars().collect();
        let file_id = FileId(0);
        let tokens = lex(file_id, &cs);
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        let mut i = 0;
        let res = SynFile::parse(&tokens, &mut i);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.items.len(), 3);
    }
}
