use crate::{ItemDefinition, ItemInductive, ItemTheorem, Parse, ParseError, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item {
    Inductive(ItemInductive),
    Definition(ItemDefinition),
    Theorem(ItemTheorem),
}

impl Parse for Item {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ParseError> {
        if let Some(inductive) = ItemInductive::parse(tokens, i)? {
            Ok(Some(Item::Inductive(inductive)))
        } else if let Some(definition) = ItemDefinition::parse(tokens, i)? {
            Ok(Some(Item::Definition(definition)))
        } else if let Some(theorem) = ItemTheorem::parse(tokens, i)? {
            Ok(Some(Item::Theorem(theorem)))
        } else {
            Ok(None)
        }
    }
}
