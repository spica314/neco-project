use felis_rename::SerialId;

use crate::values::Value;

pub trait IsType {
    fn level(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Star(TypeStar),
    ForallValue(TypeForallValue),
    ForallType(TypeForallType),
    App(TypeApp),
    Atom(TypeAtom),
}

impl IsType for Type {
    fn level(&self) -> usize {
        match self {
            Type::Star(type_star) => type_star.level(),
            Type::ForallValue(type_forall_value) => type_forall_value.level(),
            Type::ForallType(type_forall_type) => type_forall_type.level(),
            Type::App(type_app) => type_app.level(),
            Type::Atom(type_atom) => type_atom.level(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeForallValue {
    arg: (Box<Value>, Box<Type>),
    ty: Box<Type>,
}

impl IsType for TypeForallValue {
    fn level(&self) -> usize {
        self.ty.level()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeForallType {
    arg: (Box<Type>, Box<Type>),
    ty: Box<Type>,
}

impl IsType for TypeForallType {
    fn level(&self) -> usize {
        self.ty.level()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeApp {
    level: usize,
    left: Box<Type>,
    right: Box<Type>,
}

impl IsType for TypeApp {
    fn level(&self) -> usize {
        self.level
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeAtom {
    level: usize,
    ref_id: SerialId,
}

impl TypeAtom {
    pub fn new(level: usize, ref_id: SerialId) -> TypeAtom {
        TypeAtom { level, ref_id }
    }
}

impl From<TypeAtom> for Type {
    fn from(value: TypeAtom) -> Self {
        Self::Atom(value)
    }
}

impl IsType for TypeAtom {
    fn level(&self) -> usize {
        self.level
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeStar {
    level: usize,
}

impl IsType for TypeStar {
    fn level(&self) -> usize {
        self.level
    }
}

impl TypeStar {
    pub fn new(level: usize) -> TypeStar {
        TypeStar { level }
    }
}

impl From<TypeStar> for Type {
    fn from(value: TypeStar) -> Self {
        Self::Star(value)
    }
}
