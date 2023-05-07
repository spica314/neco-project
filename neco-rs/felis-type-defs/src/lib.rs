use felis_rename::{SerialId, SerialIdTable};
use felis_syn::syn_file::{SynFile, SynFileItem};
use neco_table::define_wrapper_of_table;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDef {
    User(TypeDefUser),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefUser {
    pub variants: Vec<SerialId>,
}

define_wrapper_of_table!(TypeDefTable, SerialId, TypeDef);

pub fn gen_type_def_table_file(file: &SynFile, rename_table: &SerialIdTable) -> TypeDefTable {
    let mut res = TypeDefTable::new();
    for item in &file.items {
        match item {
            SynFileItem::TypeDef(type_def) => {
                let id = type_def.name.syn_tree_id();
                let id = *rename_table.get(id).unwrap();
                let mut variants = vec![];
                for variant in &type_def.variants {
                    let v_id = variant.name.syn_tree_id();
                    let v_id = *rename_table.get(v_id).unwrap();
                    variants.push(v_id);
                }
                let def = TypeDefUser { variants };
                res.insert(id, TypeDef::User(def));
            }
            _ => {}
        }
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;
    use felis_rename::rename_defs::rename_defs_file;
    use felis_syn::test_utils::parse_from_str;

    #[test]
    fn gen_type_def_table_file_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let rename_table = rename_defs_file(&file).unwrap();
        // [file], A, hoge
        assert_eq!(rename_table.len(), 3);
        let SynFileItem::TypeDef(type_def) = file.items[0].clone() else { panic!() };
        let a_id = type_def.name.syn_tree_id();
        let a_id = *rename_table.get(a_id).unwrap();
        let hoge_id = type_def.variants[0].name.syn_tree_id();
        let hoge_id = *rename_table.get(hoge_id).unwrap();
        let def = TypeDef::User(TypeDefUser {
            variants: vec![hoge_id],
        });
        let type_def_table = gen_type_def_table_file(&file, &rename_table);
        assert_eq!(type_def_table.get(a_id), Some(&def));
    }

    #[test]
    fn gen_type_def_table_file_2() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let rename_table = rename_defs_file(&file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or or_introl, A, B, or_intror, A, B
        // (9) theorem1, A, B, proof, A, B, x, l, r
        assert_eq!(rename_table.len(), 21);

        let type_def_table = gen_type_def_table_file(&file, &rename_table);

        // And
        let SynFileItem::TypeDef(type_def_and) = file.items[0].clone() else { panic!() };
        let and_id = type_def_and.name.syn_tree_id();
        let and_id = *rename_table.get(and_id).unwrap();
        let conj_id = type_def_and.variants[0].name.syn_tree_id();
        let conj_id = *rename_table.get(conj_id).unwrap();
        let def = TypeDef::User(TypeDefUser {
            variants: vec![conj_id],
        });
        assert_eq!(type_def_table.get(and_id), Some(&def));

        // Or
        let SynFileItem::TypeDef(type_def_or) = file.items[1].clone() else { panic!() };
        let or_id = type_def_or.name.syn_tree_id();
        let or_id = *rename_table.get(or_id).unwrap();
        let or_introl_id = type_def_or.variants[0].name.syn_tree_id();
        let or_introl_id = *rename_table.get(or_introl_id).unwrap();
        let or_intror_id = type_def_or.variants[1].name.syn_tree_id();
        let or_intror_id = *rename_table.get(or_intror_id).unwrap();
        let def = TypeDef::User(TypeDefUser {
            variants: vec![or_introl_id, or_intror_id],
        });
        assert_eq!(type_def_table.get(or_id), Some(&def));

        assert_eq!(type_def_table.len(), 2);
    }
}
