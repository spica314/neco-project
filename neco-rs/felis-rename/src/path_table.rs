use std::collections::HashMap;

use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_proc::SynProcDef,
    syn_type_def::SynTypeDef,
};
use neco_resolver::Resolver;

use crate::rename_defs::{DefDecoration, DefId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTableItem {
    pub children: HashMap<String, DefId>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTable {
    table: HashMap<DefId, PathTableItem>,
}

impl PathTable {
    pub fn new() -> PathTable {
        PathTable {
            table: HashMap::new(),
        }
    }
    pub fn insert(&mut self, id: DefId, item: PathTableItem) {
        self.table.insert(id, item);
    }
    pub fn get(&self, id: DefId) -> Option<&PathTableItem> {
        self.table.get(&id)
    }
    pub fn len(&self) -> usize {
        self.table.len()
    }
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

impl PathTable {
    pub fn setup_resolver(&self, file: DefId, resolver: &mut Resolver<DefId>) {
        for item in &self.get(file).unwrap().children {
            resolver.set(item.0.clone(), *item.1);
        }
    }
}

impl Default for PathTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn construct_path_table_syn_file(file: &SynFile<DefDecoration>) -> Result<PathTable, ()> {
    let mut path_table = PathTable::new();
    // record
    let mut item = PathTableItem::new();
    for syn_item in &file.items {
        match syn_item {
            SynFileItem::TypeDef(type_def) => {
                let id = type_def.ext.id;
                item.children.insert(type_def.name.as_str().to_string(), id);
            }
            SynFileItem::FnDef(fn_def) => {
                let id = fn_def.ext.id;
                item.children.insert(fn_def.name.as_str().to_string(), id);
            }
            SynFileItem::ProcDef(proc_def) => {
                let id = proc_def.ext.id;
                item.children.insert(proc_def.name.as_str().to_string(), id);
            }
            SynFileItem::Entrypoint(_) => {}
        }
    }
    {
        let id = file.ext.id;
        path_table.insert(id, item);
    }
    // children
    for syn_item in &file.items {
        match syn_item {
            SynFileItem::TypeDef(type_def) => {
                construct_path_table_syn_type_def(type_def, &mut path_table);
            }
            SynFileItem::FnDef(fn_def) => {
                construct_path_table_syn_fn_def(fn_def, &mut path_table);
            }
            SynFileItem::ProcDef(proc_def) => {
                construct_path_table_syn_proc_def(proc_def, &mut path_table);
            }
            SynFileItem::Entrypoint(_) => {}
        }
    }
    Ok(path_table)
}

fn construct_path_table_syn_type_def(
    type_def: &SynTypeDef<DefDecoration>,
    path_table: &mut PathTable,
) {
    let mut item = PathTableItem::new();
    for variant in &type_def.variants {
        let id = variant.ext.id;
        item.children.insert(variant.name.as_str().to_string(), id);
    }
    {
        let id = type_def.ext.id;
        path_table.insert(id, item);
    }
}

fn construct_path_table_syn_fn_def(_fn_def: &SynFnDef<DefDecoration>, _path_table: &mut PathTable) {
}

fn construct_path_table_syn_proc_def(
    _proc_def: &SynProcDef<DefDecoration>,
    _path_table: &mut PathTable,
) {
}

#[cfg(test)]
mod test {
    use felis_syn::{decoration::UD, syn_file::SynFile, test_utils::parse_from_str};

    use crate::{
        path_table::construct_path_table_syn_file, rename_defs::rename_defs_file,
        rename_defs::RenameDefContext,
    };

    #[test]
    fn felis_construct_path_table_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();

        let mut rename_def_context = RenameDefContext::new();
        let file_with_def = rename_defs_file(&mut rename_def_context, &file).unwrap();
        // [file], A, hoge
        assert_eq!(rename_def_context.def_count(), 3);

        let path_table = construct_path_table_syn_file(&file_with_def).unwrap();
        // [file] -> A, A -> hoge
        assert_eq!(path_table.len(), 2);
    }

    #[test]
    fn felis_construct_path_table_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();

        let mut rename_def_context = RenameDefContext::new();
        let file_with_def = rename_defs_file(&mut rename_def_context, &file).unwrap();
        // [file], proof, A, B, x, l, r
        assert_eq!(rename_def_context.def_count(), 7);

        let path_table = construct_path_table_syn_file(&file_with_def).unwrap();
        // [file] -> proof
        assert_eq!(path_table.len(), 1);
    }
}
