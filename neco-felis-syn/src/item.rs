use crate::{
    ItemDefinition, ItemInductive, ItemTheorem, Parse, ParseError, Phase, PhaseParse, token::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item<P: Phase> {
    Inductive(ItemInductive<P>),
    Definition(ItemDefinition<P>),
    Theorem(ItemTheorem<P>),
}

impl Parse for Item<PhaseParse> {
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
