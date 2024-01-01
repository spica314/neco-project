use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_entrypoint::SynEntrypoint,
    syn_proc::SynProcDef,
    syn_type_def::SynTypeDef,
    token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFile<D: Decoration> {
    pub items: Vec<SynFileItem<D>>,
    pub ext: D::File,
}

impl Parse for SynFile<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut items = vec![];
        while let Some(item) = SynFileItem::parse(tokens, &mut k)? {
            items.push(item);
        }

        *i = k;
        Ok(Some(SynFile { items, ext: () }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynFileItem<D: Decoration> {
    TypeDef(SynTypeDef<D>),
    Entrypoint(SynEntrypoint<D>),
    ProcDef(SynProcDef<D>),
}

impl Parse for SynFileItem<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(entrypoint) = SynEntrypoint::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynFileItem::Entrypoint(entrypoint)));
        }

        if let Some(type_def) = SynTypeDef::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynFileItem::TypeDef(type_def)));
        }

        if let Some(proc_def) = SynProcDef::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynFileItem::ProcDef(proc_def)));
        }

        Ok(None)
    }
}
