use felis_rename::{SerialId, SerialIdTable};
use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_type::{SynType, SynTypeApp, SynTypeAtom, SynTypeDependentMap, SynTypeMap},
    syn_type_def::SynTypeDef,
    token::TokenIdent,
};
use felis_term::{IsTerm, Term, TermApp, TermAtom, TermDependentMap, TermMap, TypedTerm};
use neco_table::define_wrapper_of_table;

// 構文要素に対するTypedTerm
define_wrapper_of_table!(TypedTermTable, SerialId, TypedTerm);

pub fn type_check_syn_file(
    file: &SynFile,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
) {
    for item in &file.items {
        match item {
            SynFileItem::TypeDef(type_def) => {
                type_check_syn_type_def(type_def, rename_table, typed_term_table)
            }
            _ => unimplemented!(),
        }
    }
}

fn as_ref_ident(ty: &SynType) -> &TokenIdent {
    match ty {
        SynType::Forall(_) => panic!("not take into account"),
        SynType::App(_) => todo!(),
        SynType::Atom(atom) => &atom.ident,
        SynType::Map(_) => todo!(),
        SynType::Paren(_) => todo!(),
        SynType::DependentMap(_) => todo!(),
    }
}

pub fn type_check_syn_type(
    ty: &SynType,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
) -> TypedTerm {
    match ty {
        SynType::Forall(_) => todo!(),
        SynType::App(app) => type_check_syn_type_app(app, rename_table, typed_term_table),
        SynType::Atom(atom) => type_check_syn_type_atom(atom, rename_table, typed_term_table),
        SynType::Map(map) => type_check_syn_type_map(map, rename_table, typed_term_table),
        SynType::Paren(_) => todo!(),
        SynType::DependentMap(dep_map) => {
            type_check_syn_type_dep_map(dep_map, rename_table, typed_term_table)
        }
    }
    // let ty_id = ty.syn_tree_id();

    // // let id = as_ref_ident(&type_def.ty_ty).syn_tree_id();
    // // let id = *rename_table.get(id).unwrap();
    // // let ty_ty = type_table.get(id).unwrap();
    // // let ty_ty_level = ty_ty.level();
    // unimplemented!()
}

pub fn type_check_syn_type_app(
    ty_app: &SynTypeApp,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
) -> TypedTerm {
    let fun_typed_term = type_check_syn_type(&ty_app.left, rename_table, typed_term_table);
    let arg_typed_term = type_check_syn_type(&ty_app.right, rename_table, typed_term_table);
    // todo: check A -> B and A
    let term = TermApp::new(fun_typed_term.term.clone(), arg_typed_term.term.clone());
    let ty = match fun_typed_term.ty {
        Term::Map(ty_map) => ty_map.to.as_ref().clone(),
        Term::DependentMap(ty_dep_map) => ty_dep_map.to.as_ref().clone(),
        _ => panic!(),
    };
    TypedTerm {
        term: term.into(),
        ty,
    }
}

pub fn type_check_syn_type_map(
    ty_map: &SynTypeMap,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
) -> TypedTerm {
    let from_typed_term = type_check_syn_type(&ty_map.from, rename_table, typed_term_table);
    let to_typed_term = type_check_syn_type(&ty_map.to, rename_table, typed_term_table);
    let term = TermMap {
        from: Box::new(from_typed_term.term.clone()),
        to: Box::new(to_typed_term.term.clone()),
    };
    let ty = to_typed_term.ty.clone();
    TypedTerm {
        term: term.into(),
        ty: ty.into(),
    }
}

pub fn type_check_syn_type_dep_map(
    ty_map: &SynTypeDependentMap,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
) -> TypedTerm {
    let from_ty_typed_term =
        type_check_syn_type(&ty_map.from.as_ref().ty, rename_table, typed_term_table);
    let from_term = TermAtom::new(
        from_ty_typed_term.term.level() - 1,
        *rename_table.get(ty_map.from.name.syn_tree_id()).unwrap(),
    );
    typed_term_table.insert(
        *rename_table.get(ty_map.from.name.syn_tree_id()).unwrap(),
        TypedTerm {
            term: from_term.clone().into(),
            ty: from_ty_typed_term.term.clone(),
        },
    );
    let to_typed_term = type_check_syn_type(&ty_map.to, rename_table, typed_term_table);
    let term = TermDependentMap::new((from_term, from_ty_typed_term.term), to_typed_term.term);
    let ty = to_typed_term.ty.clone();
    TypedTerm {
        term: term.into(),
        ty: ty.into(),
    }
}

pub fn type_check_syn_type_atom(
    ty_atom: &SynTypeAtom,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
) -> TypedTerm {
    eprintln!();
    eprintln!("ty_atom = {:?}", ty_atom);
    eprintln!("rename_table = {:?}", rename_table);
    let id = ty_atom.syn_tree_id();
    eprintln!("id = {:?}", id);
    let id = *rename_table.get(id).unwrap();
    eprintln!("id(serial) = {:?}", id);
    eprintln!("typed_term_table = {:?}", typed_term_table);
    typed_term_table.get(id).unwrap().clone()
}

pub fn type_check_syn_type_def(
    type_def: &SynTypeDef,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
) {
    let typed_ty_ty = type_check_syn_type(&type_def.ty_ty, rename_table, typed_term_table);
    let ty_ty = &typed_ty_ty.term;
    eprintln!("ty_ty = {:?}", ty_ty);
    let ty_ty_level = typed_ty_ty.term.level();
    eprintln!("ty_ty_level = {}", ty_ty_level);
    // todo: check typed_ty_ty.ty is *

    // name
    {
        let id2 = type_def.name.syn_tree_id();
        let id2 = *rename_table.get(id2).unwrap();
        let term = TermAtom::new(ty_ty_level - 1, id2).into();
        let ty = typed_ty_ty.term.clone();
        let typed_term = TypedTerm { term, ty };
        typed_term_table.insert(id2, typed_term);
    }

    // variants
    for variant in &type_def.variants {
        let id2 = variant.name.syn_tree_id();
        let id2 = *rename_table.get(id2).unwrap();
        let typed_term = type_check_syn_type(&variant.ty, rename_table, typed_term_table);
        let term = TermAtom::new(typed_term.term.level() - 1, id2).into();
        let ty = typed_term.term.clone();
        let typed_term = TypedTerm { term, ty };
        typed_term_table.insert(id2, typed_term);
    }
}

// fn syn_type_to_type(
//     syn_type: &SynType,
//     rename_table: &SerialIdTable,
//     type_table: &mut TypeTable,
// ) -> Term {
//     match syn_type {
//         SynType::Forall(_) => todo!(),
//         SynType::App(syn_type_app) => {
//             let fun_ty = syn_type_to_type(&syn_type_app.left, rename_table, type_table);
//             let arg_ty = syn_type_to_type(&syn_type_app.right, rename_table, type_table);
//             eprintln!("syn_type_app = {:?}", syn_type_app);
//             eprintln!("fun_ty = {:?}", fun_ty);
//             eprintln!("arg_ty = {:?}", arg_ty);
//             match fun_ty {
//                 Type::ForallValue(type_forall_value) => type_forall_value.ty.as_ref().clone(),
//                 Type::ForallType(type_forall_type) => type_forall_type.ty.as_ref().clone(),
//                 _ => panic!(),
//             }
//         }
//         SynType::Atom(syn_type_atom) => {
//             let id = syn_type_atom.ident.syn_tree_id();
//             let id = *rename_table.get(id).unwrap();
//             let ty = type_table
//                 .get(id)
//                 .unwrap_or_else(|| panic!("ident = {:?}", syn_type_atom.ident));
//             let ty_level = ty.level();
//             TypeAtom::new(ty_level - 1, id).into()
//         }
//         SynType::Map(syn_type_map) => {
//             let from_ty = syn_type_to_type(&syn_type_map.from, rename_table, type_table);
//             let ty = syn_type_to_type(&syn_type_map.to, rename_table, type_table);
//             if from_ty.level() == 0 {
//                 let v = ValueAtom::new(SerialId::new()).into();
//                 TypeForallValue::new(v, from_ty, ty).into()
//             } else {
//                 let v = TypeAtom::new(from_ty.level() - 1, SerialId::new()).into();
//                 TypeForallType::new(v, from_ty, ty).into()
//             }
//         }
//         SynType::Paren(_) => todo!(),
//         SynType::DependentMap(_) => todo!(),
//     }
// }

#[cfg(test)]
mod test {
    use felis_rename::{
        rename_defs::rename_defs_file, rename_uses::rename_uses_file, SerialId, SerialIdTable,
    };
    use felis_syn::test_utils::parse_from_str;
    use felis_term::{TermDependentMap, TermMap, TermStar};
    use neco_resolver::Resolver;

    use super::*;

    #[test]
    fn type_check_test_1() {
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
        let mut typed_term_table = TypedTermTable::new();
        typed_term_table.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        type_check_syn_file(&file, &rename_table, &mut typed_term_table);
        assert_eq!(typed_term_table.len(), 3);
        // Prop : *
        assert_eq!(
            typed_term_table.get(prop_id).unwrap(),
            &TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            }
        );
        // A : Prop
        let SynFileItem::TypeDef(ref type_def) = file.items[0] else { panic!() };
        let a_id = *rename_table.get(type_def.name.syn_tree_id()).unwrap();
        assert_eq!(
            typed_term_table.get(a_id).unwrap(),
            &TypedTerm {
                term: TermAtom::new(1, a_id).into(),
                ty: TermAtom::new(2, prop_id).into(),
            }
        );
        // hoge : A
        let hoge_id = *rename_table
            .get(type_def.variants[0].name.syn_tree_id())
            .unwrap();
        assert_eq!(
            typed_term_table.get(hoge_id).unwrap(),
            &TypedTerm {
                term: TermAtom::new(0, hoge_id).into(),
                ty: TermAtom::new(1, a_id).into(),
            }
        );
    }

    #[test]
    fn type_check_test_2() {
        let s = std::fs::read_to_string("../../library/wip/and2.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        eprintln!("file = {:?}", file);
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id).unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver).unwrap();
        /* merge def and use */
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);

        let mut typed_term_table = TypedTermTable::new();
        typed_term_table.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        type_check_syn_file(&file, &rename_table, &mut typed_term_table);
        assert_eq!(typed_term_table.len(), 5, "{:?}", typed_term_table);
        // Prop : *
        assert_eq!(
            typed_term_table.get(prop_id).unwrap(),
            &TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            }
        );
        // And : Prop -> Prop -> Prop
        let SynFileItem::TypeDef(ref type_def) = file.items[0] else { panic!() };
        let a_id = *rename_table.get(type_def.name.syn_tree_id()).unwrap();
        assert_eq!(
            typed_term_table.get(a_id).unwrap(),
            &TypedTerm {
                term: TermAtom::new(1, a_id).into(),
                ty: Term::Map(TermMap {
                    from: Box::new(Term::Atom(TermAtom::new(2, prop_id))),
                    to: Box::new(Term::Map(TermMap {
                        from: Box::new(Term::Atom(TermAtom::new(2, prop_id))),
                        to: Box::new(Term::Atom(TermAtom::new(2, prop_id))),
                    })),
                }),
            }
        );
        // conj : (A : Prop) -> (B : Prop) -> A -> B -> And A B
        let conj_id = *rename_table
            .get(type_def.variants[0].name.syn_tree_id())
            .unwrap();
        let conj_a_id = *rename_table
            .get(
                type_def.variants[0]
                    .ty
                    .as_dependent_map()
                    .unwrap()
                    .from
                    .name
                    .syn_tree_id(),
            )
            .unwrap();
        let conj_b_id = *rename_table
            .get(
                type_def.variants[0]
                    .ty
                    .as_dependent_map()
                    .unwrap()
                    .to
                    .as_dependent_map()
                    .unwrap()
                    .from
                    .name
                    .syn_tree_id(),
            )
            .unwrap();
        assert_eq!(
            typed_term_table.get(conj_id).unwrap(),
            &TypedTerm {
                term: TermAtom::new(0, conj_id).into(),
                ty: Term::DependentMap(TermDependentMap {
                    from: (
                        TermAtom::new(1, conj_a_id),
                        Box::new(Term::Atom(TermAtom::new(2, prop_id))),
                    ),
                    to: Box::new(Term::DependentMap(TermDependentMap {
                        from: (
                            TermAtom::new(1, conj_b_id),
                            Box::new(Term::Atom(TermAtom::new(2, prop_id))),
                        ),
                        to: Box::new(Term::Map(TermMap {
                            from: Box::new(TermAtom::new(1, conj_a_id).into()),
                            to: Box::new(Term::Map(TermMap {
                                from: Box::new(TermAtom::new(1, conj_b_id).into()),
                                to: Box::new(
                                    TermApp {
                                        fun: Box::new(
                                            TermApp {
                                                fun: Box::new(TermAtom::new(1, a_id).into()),
                                                arg: Box::new(TermAtom::new(1, conj_a_id).into()),
                                            }
                                            .into()
                                        ),
                                        arg: Box::new(TermAtom::new(1, conj_b_id).into())
                                    }
                                    .into()
                                )
                            }))
                        })),
                    })),
                }),
            }
        );
    }
}
