use neco_table::TableId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Atom(ValueAtom),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueAtom {
    id: TableId,
}
