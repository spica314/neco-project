use std::collections::HashMap;

use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_theorem_def::SynTheoremDef,
    syn_type_def::SynTypeDef,
};
use neco_resolver::Resolver;
use neco_table::define_wrapper_of_table;

use crate::{SerialId, SerialIdTable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTableItem {
    pub children: HashMap<String, SerialId>,
}

impl PathTableItem {
    pub fn new() -> PathTableItem {
        PathTableItem {
            children: HashMap::new(),
        }
    }
}

impl Default for PathTableItem {
    fn default() -> Self {
        Self::new()
    }
}

define_wrapper_of_table!(PathTable, SerialId, PathTableItem);

impl PathTable {
    pub fn setup_resolver(&self, file: SerialId, resolver: &mut Resolver<SerialId>) {
        for item in &self.get(file).unwrap().children {
            resolver.set(item.0.clone(), *item.1).unwrap();
        }
    }
}

pub fn construct_path_table_syn_file(
    file: &SynFile,
    rename_table: &SerialIdTable,
) -> Result<PathTable, ()> {
    let mut path_table = PathTable::new();
    // record
    let mut item = PathTableItem::new();
    for syn_item in &file.items {
        match syn_item {
            SynFileItem::TypeDef(type_def) => {
                let id = type_def.name.syn_tree_id();
                let id = *rename_table.get(id).unwrap();
                item.children.insert(type_def.name.as_str().to_string(), id);
            }
            SynFileItem::FnDef(fn_def) => {
                let id = fn_def.name.syn_tree_id();
                let id = *rename_table.get(id).unwrap();
                item.children.insert(fn_def.name.as_str().to_string(), id);
            }
            SynFileItem::TheoremDef(theorem_def) => {
                let id = theorem_def.name.syn_tree_id();
                let id = *rename_table.get(id).unwrap();
                item.children
                    .insert(theorem_def.name.as_str().to_string(), id);
            }
        }
    }
    {
        let id = file.syn_tree_id();
        let id = *rename_table.get(id).unwrap();
        path_table.insert(id, item);
    }
    // children
    for syn_item in &file.items {
        match syn_item {
            SynFileItem::TypeDef(type_def) => {
                construct_path_table_syn_type_def(type_def, rename_table, &mut path_table);
            }
            SynFileItem::FnDef(fn_def) => {
                construct_path_table_syn_fn_def(fn_def, rename_table, &mut path_table);
            }
            SynFileItem::TheoremDef(theorem_def) => {
                construct_path_table_syn_theorem_def(theorem_def, rename_table, &mut path_table);
            }
        }
    }
    Ok(path_table)
}

fn construct_path_table_syn_type_def(
    type_def: &SynTypeDef,
    rename_table: &SerialIdTable,
    path_table: &mut PathTable,
) {
    let mut item = PathTableItem::new();
    for variant in &type_def.variants {
        let id = variant.name.syn_tree_id();
        let id = *rename_table.get(id).unwrap();
        item.children.insert(variant.name.as_str().to_string(), id);
    }
    {
        let id = type_def.name.syn_tree_id();
        let id = *rename_table.get(id).unwrap();
        path_table.insert(id, item);
    }
}

fn construct_path_table_syn_fn_def(
    _fn_def: &SynFnDef,
    _rename_table: &SerialIdTable,
    _path_table: &mut PathTable,
) {
}

fn construct_path_table_syn_theorem_def(
    _theorem_def: &SynTheoremDef,
    _rename_table: &SerialIdTable,
    _path_table: &mut PathTable,
) {
}

#[cfg(test)]
mod test {
    use felis_syn::{syn_file::SynFile, test_utils::parse_from_str};

    use crate::{path_table::construct_path_table_syn_file, rename_defs::rename_defs_file};

    #[test]
    fn felis_construct_path_table_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let def_table = rename_defs_file(&file).unwrap();
        // [file], A, hoge
        assert_eq!(def_table.len(), 3);
        let path_table = construct_path_table_syn_file(&file, &def_table).unwrap();
        // [file] -> A, A -> hoge
        assert_eq!(path_table.len(), 2);
    }

    #[test]
    fn felis_construct_path_table_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let def_table = rename_defs_file(&file).unwrap();
        // [file], proof, A, B, x, l, r
        assert_eq!(def_table.len(), 7);
        let path_table = construct_path_table_syn_file(&file, &def_table).unwrap();
        // [file] -> proof
        assert_eq!(path_table.len(), 1);
    }

    #[test]
    fn felis_construct_path_table_test_4() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let def_table = rename_defs_file(&file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or or_introl, A, B, or_intror, A, B
        // (11) theorem1, A, B, proof, A, B, x, _, _, l, r
        assert_eq!(def_table.len(), 23);
        let path_table = construct_path_table_syn_file(&file, &def_table).unwrap();
        // [file] -> (And, Or, theorem1), And -> (conj), Or -> (or_introl, or_intror)
        assert_eq!(path_table.len(), 3);
    }
}
