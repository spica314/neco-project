use std::collections::HashMap;

use neco_table::define_wrapper_of_table;

use crate::SerialId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTableItem {
    pub children: HashMap<String, SerialId>,
    pub items: HashMap<String, SerialId>,
}

define_wrapper_of_table!(PathTable, SerialId, PathTableItem);
