use neco_felis_syn::{File, PhaseParse};

use crate::phase_renamed::PhaseRenamed;

pub mod phase_renamed;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// (file_id, variable_id_in_the_file)
pub struct VariableId(pub usize, pub usize);

pub fn rename_file(_file: File<PhaseParse>) -> File<PhaseRenamed> {
    todo!()
}
