use std::collections::HashMap;

use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_theorem_def::SynTheoremDef,
    syn_type_def::SynTypeDef,
};
use neco_resolver::Resolver;

use crate::rename_defs2::{DefDecoration, DefId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTable2Item {
    pub children: HashMap<String, DefId>,
}

impl PathTable2Item {
    pub fn new() -> PathTable2Item {
        PathTable2Item {
            children: HashMap::new(),
        }
    }
}

impl Default for PathTable2Item {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTable2 {
    table: HashMap<DefId, PathTable2Item>,
}

impl PathTable2 {
    pub fn new() -> PathTable2 {
        PathTable2 {
            table: HashMap::new(),
        }
    }
    pub fn insert(&mut self, id: DefId, item: PathTable2Item) {
        self.table.insert(id, item);
    }
    pub fn get(&self, id: DefId) -> Option<&PathTable2Item> {
        self.table.get(&id)
    }
    pub fn len(&self) -> usize {
        self.table.len()
    }
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

impl PathTable2 {
    pub fn setup_resolver(&self, file: DefId, resolver: &mut Resolver<DefId>) {
        for item in &self.get(file).unwrap().children {
            resolver.set(item.0.clone(), *item.1);
        }
    }
}

impl Default for PathTable2 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn construct_path_table_syn_file(file: &SynFile<DefDecoration>) -> Result<PathTable2, ()> {
    let mut path_table = PathTable2::new();
    // record
    let mut item = PathTable2Item::new();
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
            SynFileItem::TheoremDef(theorem_def) => {
                let id = theorem_def.ext.id;
                item.children
                    .insert(theorem_def.name.as_str().to_string(), id);
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
            SynFileItem::TheoremDef(theorem_def) => {
                construct_path_table_syn_theorem_def(theorem_def, &mut path_table);
            }
            SynFileItem::Entrypoint(_) => {}
        }
    }
    Ok(path_table)
}

fn construct_path_table_syn_type_def(
    type_def: &SynTypeDef<DefDecoration>,
    path_table: &mut PathTable2,
) {
    let mut item = PathTable2Item::new();
    for variant in &type_def.variants {
        let id = variant.ext.id;
        item.children.insert(variant.name.as_str().to_string(), id);
    }
    {
        let id = type_def.ext.id;
        path_table.insert(id, item);
    }
}

fn construct_path_table_syn_fn_def(
    _fn_def: &SynFnDef<DefDecoration>,
    _path_table: &mut PathTable2,
) {
}

fn construct_path_table_syn_theorem_def(
    _theorem_def: &SynTheoremDef<DefDecoration>,
    _path_table: &mut PathTable2,
) {
}

#[cfg(test)]
mod test {
    use felis_syn::{decoration::UD, syn_file::SynFile, test_utils::parse_from_str};

    use crate::{
        path_table2::construct_path_table_syn_file, rename_defs2::rename_defs_file,
        rename_defs2::RenameDefContext,
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

    #[test]
    fn felis_construct_path_table_test_4() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();

        let mut rename_def_context = RenameDefContext::new();
        let file_with_def = rename_defs_file(&mut rename_def_context, &file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or or_introl, A, B, or_intror, A, B
        // (11) theorem1, A, B, proof, A, B, x, _, _, l, r
        assert_eq!(rename_def_context.def_count(), 23);

        let path_table = construct_path_table_syn_file(&file_with_def).unwrap();
        // [file] -> (And, Or, theorem1), And -> (conj), Or -> (or_introl, or_intror)
        assert_eq!(path_table.len(), 3);
    }
}
