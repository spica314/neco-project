use felis_rename::{SerialId, SerialIdTable};
use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_type::SynType,
    syn_type_def::SynTypeDef,
    token::TokenIdent,
};
use felis_types_and_values::types::{IsType, Type, TypeAtom};
use neco_table::define_wrapper_of_table;

define_wrapper_of_table!(TypeTable, SerialId, Type);

pub fn type_check_file(file: &SynFile, rename_table: &SerialIdTable, type_table: &mut TypeTable) {
    for item in &file.items {
        match item {
            SynFileItem::TypeDef(type_def) => {
                type_check_type_def(type_def, rename_table, type_table)
            }
            _ => unimplemented!(),
        }
    }
}

fn as_ref_ident(ty: &SynType) -> &TokenIdent {
    match ty {
        SynType::Forall(_) => todo!(),
        SynType::App(_) => todo!(),
        SynType::Atom(atom) => &atom.ident,
        SynType::Map(_) => todo!(),
        SynType::Paren(_) => todo!(),
        SynType::DependentMap(_) => todo!(),
    }
}

pub fn type_check_type_def(
    type_def: &SynTypeDef,
    rename_table: &SerialIdTable,
    type_table: &mut TypeTable,
) {
    let id = as_ref_ident(&type_def.ty_ty).syn_tree_id();
    let id = *rename_table.get(id).unwrap();
    let ty_ty = type_table.get(id).unwrap();
    let ty_ty_level = ty_ty.level();

    {
        let id2 = type_def.name.syn_tree_id();
        let id2 = *rename_table.get(id2).unwrap();
        type_table.insert(id2, TypeAtom::new(ty_ty_level - 1, id).into());
    }

    for variant in &type_def.variants {
        let id = variant.name.syn_tree_id();
        let id = *rename_table.get(id).unwrap();
        let ty = syn_type_to_type(&variant.ty, rename_table, type_table);
        type_table.insert(id, ty);
    }
}

fn syn_type_to_type(
    syn_type: &SynType,
    rename_table: &SerialIdTable,
    type_table: &mut TypeTable,
) -> Type {
    match syn_type {
        SynType::Forall(_) => todo!(),
        SynType::App(_) => todo!(),
        SynType::Atom(syn_type_atom) => {
            let id = syn_type_atom.ident.syn_tree_id();
            let id = *rename_table.get(id).unwrap();
            let ty = type_table.get(id).unwrap();
            let ty_level = ty.level();
            TypeAtom::new(ty_level - 1, id).into()
        }
        SynType::Map(_) => todo!(),
        SynType::Paren(_) => todo!(),
        SynType::DependentMap(_) => todo!(),
    }
}

#[cfg(test)]
mod test {
    use felis_rename::{
        rename_defs::rename_defs_file, rename_uses::rename_uses_file, SerialId, SerialIdTable,
    };
    use felis_syn::test_utils::parse_from_str;
    use felis_types_and_values::types::{TypeAtom, TypeStar};
    use neco_resolver::Resolver;

    use super::*;

    #[test]
    fn felis_rename_uses_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id).unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver).unwrap();
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut type_table = TypeTable::new();
        type_table.insert(prop_id, TypeStar::new(2).into());
        type_check_file(&file, &rename_table, &mut type_table);
        assert_eq!(type_table.len(), 3);
        // Prop : *
        assert_eq!(type_table.get(prop_id).unwrap(), &TypeStar::new(2).into());
        // A : Prop
        let SynFileItem::TypeDef(ref type_def) = file.items[0] else { panic!() };
        let a_id = *rename_table.get(type_def.name.syn_tree_id()).unwrap();
        assert_eq!(
            type_table.get(a_id).unwrap(),
            &TypeAtom::new(1, prop_id).into()
        );
        // hoge : A
        let hoge_id = *rename_table
            .get(type_def.variants[0].name.syn_tree_id())
            .unwrap();
        assert_eq!(
            type_table.get(hoge_id).unwrap(),
            &TypeAtom::new(0, a_id).into()
        );
    }
}
