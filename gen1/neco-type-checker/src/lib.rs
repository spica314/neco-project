pub mod type_checker;
pub mod type_term;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct TypeId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct VarId(usize);
