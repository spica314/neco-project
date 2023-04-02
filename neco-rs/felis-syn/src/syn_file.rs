use crate::{
    parse::Parse, syn_fn_def::SynFnDef, syn_theorem_def::SynTheoremDef, syn_type_def::SynTypeDef,
    token::Token, SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFile {
    id: SynTreeId,
    pub items: Vec<SynFileItem>,
}

impl SynFile {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynFile {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut items = vec![];
        while let Some(item) = SynFileItem::parse(tokens, &mut k)? {
            items.push(item);
        }

        *i = k;
        Ok(Some(SynFile {
            id: SynTreeId::new(),
            items,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    use crate::Parser;

    use super::*;

    #[test]
    fn felis_syn_file_parse_test_1() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let mut parser = Parser::new();
        let res = parser.parse::<SynFile>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.items.len(), 3);
    }

    #[test]
    fn felis_syn_file_parse_test_2() {
        let s = std::fs::read_to_string("../../library/wip/prop3.fe").unwrap();
        let mut parser = Parser::new();
        let res = parser.parse::<SynFile>(&s);
        assert!(res.is_ok());
        let (res, _) = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.items.len(), 1);
    }
}
