use felis_syn::SynTreeId;
use neco_table::{define_wrapper_of_table, define_wrapper_of_table_id};

pub mod path_table;
pub mod rename_defs;
pub mod rename_uses;

pub mod rename_defs2;

define_wrapper_of_table_id!(SerialId);
define_wrapper_of_table!(SerialIdTable, SynTreeId, SerialId);
