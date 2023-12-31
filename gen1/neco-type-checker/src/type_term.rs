use crate::{TypeId, VarId};

pub mod type_term_base;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeTerm {
    Base(TypeId),
    Var(VarId),
    App(Box<TypeTerm>, Box<TypeTerm>),
    Arrow(Box<TypeTerm>, Box<TypeTerm>),
    Star(TypeLevel),
    Candidates(Vec<TypeTerm>),
    Unknown,
}

impl TypeTerm {
    pub fn is_unknown(&self) -> bool {
        matches!(self, TypeTerm::Unknown)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeLevel {
    level: usize,
}

impl TypeLevel {
    pub fn new(level: usize) -> Self {
        Self { level }
    }

    pub fn as_usize(&self) -> usize {
        self.level
    }
}
