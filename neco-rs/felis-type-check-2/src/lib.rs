use felis_rename::{SerialId, SerialIdTable};
use felis_syn::{
    syn_expr::{SynExpr, SynExprIdentWithPath, SynExprMatch},
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_theorem_def::SynTheoremDef,
    syn_type::{SynType, SynTypeApp, SynTypeAtom, SynTypeDependentMap, SynTypeMap},
    syn_type_def::SynTypeDef,
    SynTreeId,
};
use felis_term::{
    compute_apped_typed_term, remap_term, IsTerm, Term, TermApp, TermAtom, TermDependentMap,
    TermMap, TermMatch, TermStar, TypedTerm,
};
use felis_type_defs::{TypeDef, TypeDefTable};
use neco_table::define_wrapper_of_table;

// 構文要素に対するTypedTerm
define_wrapper_of_table!(TypedTermTable, SynTreeId, TypedTerm);

// Atomに対応するTypedTerm
define_wrapper_of_table!(TypedTermTableForAtom, SerialId, TypedTerm);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckError {
    TypeMismatched { expected_ty: Term, actual_ty: Term },
}

pub fn type_check_syn_file(
    file: &SynFile,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
    type_def_table: &TypeDefTable,
) -> Result<(), TypeCheckError> {
    for item in &file.items {
        match item {
            SynFileItem::TypeDef(type_def) => type_check_syn_type_def(
                type_def,
                rename_table,
                typed_term_table,
                typed_term_table_for_atom,
                type_def_table,
            )?,
            SynFileItem::TheoremDef(theorem_def) => type_check_syn_theorem_def(
                theorem_def,
                rename_table,
                typed_term_table,
                typed_term_table_for_atom,
                type_def_table,
            )?,
            _ => unimplemented!(),
        }
    }
    Ok(())
}

pub fn type_check_syn_theorem_def(
    theorem_def: &SynTheoremDef,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
    type_def_table: &TypeDefTable,
) -> Result<(), TypeCheckError> {
    type_check_syn_fn_def(
        &theorem_def.fn_def,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
        type_def_table,
    )
}

pub fn type_check_syn_fn_def(
    fn_def: &SynFnDef,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
    type_def_table: &TypeDefTable,
) -> Result<(), TypeCheckError> {
    let ty = type_check_syn_type(
        &fn_def.ty,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    let final_return_type = ty.term.final_return_type();
    let mut final_ty = None;
    for statement in &fn_def.fn_block.statements {
        match statement {
            felis_syn::syn_fn_def::SynStatement::Expr(expr) => {
                final_ty = Some(type_check_syn_expr(
                    expr,
                    rename_table,
                    typed_term_table,
                    typed_term_table_for_atom,
                    type_def_table,
                )?);
            }
        }
    }
    if let Some(final_ty) = final_ty {
        if final_ty.ty != final_return_type {
            return Err(TypeCheckError::TypeMismatched {
                expected_ty: final_return_type,
                actual_ty: final_ty.ty,
            });
        }
    }
    Ok(())
}

pub fn type_check_syn_expr(
    expr: &SynExpr,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
    type_def_table: &TypeDefTable,
) -> Result<TypedTerm, TypeCheckError> {
    match expr {
        SynExpr::Ident(_) => todo!(),
        SynExpr::IdentWithPath(expr_ident_with_path) => type_check_syn_expr_ident_with_path(
            expr_ident_with_path,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
        ),
        SynExpr::App(app) => {
            eprintln!("app = {:?}", app);
            let mut terms = vec![];
            for expr in &app.exprs {
                let term = type_check_syn_expr(
                    expr,
                    rename_table,
                    typed_term_table,
                    typed_term_table_for_atom,
                    type_def_table,
                )?;
                terms.push(term);
            }
            eprintln!("terms = {:?}", terms);
            let mut res = terms[0].clone();
            for arg in &terms[1..] {
                match compute_apped_typed_term(&res, arg) {
                    Ok(r) => {
                        res = r;
                    }
                    Err(err) => {
                        return Err(TypeCheckError::TypeMismatched {
                            expected_ty: err.expected_ty,
                            actual_ty: err.actual_ty,
                        });
                    }
                }
            }
            Ok(res)
        }
        SynExpr::Match(expr_match) => type_check_syn_expr_match(
            expr_match,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
            type_def_table,
        ),
        SynExpr::Paren(expr_paren) => type_check_syn_expr(
            &expr_paren.expr,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
            type_def_table,
        ),
    }
}

pub fn type_check_syn_expr_ident_with_path(
    expr_ident_with_path: &SynExprIdentWithPath,
    rename_table: &SerialIdTable,
    _typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
) -> Result<TypedTerm, TypeCheckError> {
    eprintln!("expr = {:?}", expr_ident_with_path);
    let syn_tree_id = expr_ident_with_path.ident.syn_tree_id();
    let serial_id = *rename_table.get(syn_tree_id).unwrap();
    let typed_term = typed_term_table_for_atom.get(serial_id).unwrap().clone();
    Ok(typed_term)
}

fn calc_type_of_term(term: &Term, typed_term_table_for_atom: &mut TypedTermTableForAtom) -> Term {
    match term {
        Term::Atom(atom) => typed_term_table_for_atom.get(atom.id()).unwrap().ty.clone(),
        Term::Star(star) => TermStar::new(star.level() + 1).into(),
        Term::Map(map) => calc_type_of_term(&map.to, typed_term_table_for_atom),
        Term::DependentMap(dependent_map) => {
            calc_type_of_term(&dependent_map.to, typed_term_table_for_atom)
        }
        Term::App(app) => {
            eprintln!("app = {:?}", app);
            // todo: check
            match calc_type_of_term(app.fun.as_ref(), typed_term_table_for_atom) {
                Term::Map(map) => map.to.as_ref().clone(),
                Term::DependentMap(dependent_map) => dependent_map.to.as_ref().clone(),
                _ => panic!(),
            }
        }
        Term::Match(_term_match) => todo!(),
    }
}

fn get_most_left_id(term: &Term) -> SerialId {
    match term {
        Term::Atom(atom) => atom.id(),
        Term::App(app) => get_most_left_id(app.fun.as_ref()),
        _ => panic!(),
    }
}

pub fn type_check_syn_expr_match(
    expr_match: &SynExprMatch,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
    type_def_table: &TypeDefTable,
) -> Result<TypedTerm, TypeCheckError> {
    let expr_typed = type_check_syn_expr(
        &expr_match.expr,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
        type_def_table,
    )
    .unwrap();
    let expr_typed_ty = &expr_typed.ty;
    eprintln!("expr_typed_ty = {:?}", expr_typed_ty);
    let expr_typed_ty_args = {
        let mut expr_typed_ty_args = vec![];
        let mut ty = expr_typed_ty.clone();
        loop {
            match &ty {
                Term::Atom(_atom) => {
                    break;
                }
                Term::App(app) => {
                    expr_typed_ty_args.push(app.arg.as_ref().clone());
                    ty = app.fun.as_ref().clone();
                }
                _ => panic!(),
            }
        }
        expr_typed_ty_args.reverse();
        expr_typed_ty_args
    };
    eprintln!("expr_typed_ty_args = {:?}", expr_typed_ty_args);

    let expr_typed_ty_ty = calc_type_of_term(&expr_typed.ty, typed_term_table_for_atom);
    eprintln!("expr_typed_ty_ty = {:?}", expr_typed_ty_ty);
    if !matches!(expr_typed_ty_ty, Term::Atom(_)) {
        panic!();
    }
    let type_serial_id = get_most_left_id(expr_typed_ty);
    eprintln!("type_serial_id = {:?}", type_serial_id);

    let expr_match_arms: Vec<_> = expr_match
        .arms
        .iter()
        .map(|a| {
            rename_table
                .get(a.pattern.type_constructor.ident.syn_tree_id())
                .unwrap()
        })
        .cloned()
        .collect();
    eprintln!("expr_match_arms = {:?}", expr_match_arms);

    let type_def = type_def_table.get(type_serial_id).unwrap();
    let TypeDef::User(type_def_user) = type_def;
    let type_def_arms = type_def_user.variants.clone();
    eprintln!("type_def_arms = {:?}", type_def_arms);

    {
        let mut expr_match_arms = expr_match_arms;
        expr_match_arms.sort();

        let mut type_def_arms = type_def_arms;
        type_def_arms.sort();

        if expr_match_arms != type_def_arms {
            panic!();
        }
    }

    let mut arms = vec![];
    let mut arm_type = vec![];
    for arm in &expr_match.arms {
        let a = *rename_table
            .get(arm.pattern.type_constructor.ident.syn_tree_id())
            .unwrap();
        let ty_a = typed_term_table_for_atom.get(a).unwrap();
        eprintln!("ty_a = {:?}", ty_a);
        let b: Vec<_> = arm
            .pattern
            .idents
            .iter()
            .map(|x| *rename_table.get(x.syn_tree_id()).unwrap())
            .collect();
        {
            let mut ty = ty_a.ty.clone();
            let mut ty2 = ty.clone();
            eprintln!("ty2 = {:?}", ty2);
            for arg in &expr_typed_ty_args {
                match &ty2 {
                    Term::Map(map) => {
                        ty2 = map.to.as_ref().clone();
                    }
                    Term::DependentMap(dep_map) => {
                        ty = remap_term(&(dep_map.from.0.id(), arg.clone()), &ty);
                        ty2 = dep_map.to.as_ref().clone();
                    }
                    _ => panic!(),
                }
            }

            for &b in &b {
                match &ty {
                    Term::Map(map) => {
                        let from = &map.from;
                        typed_term_table_for_atom.insert(
                            b,
                            TypedTerm {
                                term: Term::Atom(TermAtom::new(from.level() - 1, b)),
                                ty: from.as_ref().clone(),
                            },
                        );
                        ty = map.to.as_ref().clone();
                    }
                    Term::DependentMap(dep_map) => {
                        let from = &dep_map.from;
                        typed_term_table_for_atom.insert(
                            b,
                            TypedTerm {
                                term: Term::Atom(TermAtom::new(from.1.level() - 1, b)),
                                ty: from.1.as_ref().clone(),
                            },
                        );
                        ty = dep_map.to.as_ref().clone();
                    }
                    _ => panic!(),
                }
            }
        }
        let arm_expr_typed = type_check_syn_expr(
            &arm.expr,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
            type_def_table,
        )
        .unwrap();
        arm_type.push(arm_expr_typed.ty.clone());
        arms.push((a, b, arm_expr_typed.term));
    }

    let term_match = TermMatch {
        expr: Box::new(expr_typed.term.clone()),
        arms,
    };
    Ok(TypedTerm {
        term: Term::Match(term_match),
        ty: arm_type[0].clone(),
    })
}

pub fn type_check_syn_type(
    ty: &SynType,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
) -> TypedTerm {
    match ty {
        SynType::Forall(_) => todo!(),
        SynType::App(app) => type_check_syn_type_app(
            app,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
        ),
        SynType::Atom(atom) => type_check_syn_type_atom(
            atom,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
        ),
        SynType::Map(map) => type_check_syn_type_map(
            map,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
        ),
        SynType::Paren(_) => todo!(),
        SynType::DependentMap(dep_map) => type_check_syn_type_dep_map(
            dep_map,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
        ),
    }
}

pub fn type_check_syn_type_app(
    ty_app: &SynTypeApp,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
) -> TypedTerm {
    let fun_typed_term = type_check_syn_type(
        &ty_app.left,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    let arg_typed_term = type_check_syn_type(
        &ty_app.right,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    // todo: check A -> B and A
    let term = TermApp::new(fun_typed_term.term.clone(), arg_typed_term.term);
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
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
) -> TypedTerm {
    let from_typed_term = type_check_syn_type(
        &ty_map.from,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    let to_typed_term = type_check_syn_type(
        &ty_map.to,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    let term = TermMap {
        from: Box::new(from_typed_term.term),
        to: Box::new(to_typed_term.term.clone()),
    };
    let ty = to_typed_term.ty;
    TypedTerm {
        term: term.into(),
        ty,
    }
}

pub fn type_check_syn_type_dep_map(
    ty_map: &SynTypeDependentMap,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
) -> TypedTerm {
    let from_ty_typed_term = type_check_syn_type(
        &ty_map.from.as_ref().ty,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    let from_term = TermAtom::new(
        from_ty_typed_term.term.level() - 1,
        *rename_table.get(ty_map.from.name.syn_tree_id()).unwrap(),
    );
    typed_term_table_for_atom.insert(
        *rename_table.get(ty_map.from.name.syn_tree_id()).unwrap(),
        TypedTerm {
            term: from_term.clone().into(),
            ty: from_ty_typed_term.term.clone(),
        },
    );
    let to_typed_term = type_check_syn_type(
        &ty_map.to,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    let term = TermDependentMap::new((from_term, from_ty_typed_term.term), to_typed_term.term);
    let ty = to_typed_term.ty;
    TypedTerm {
        term: term.into(),
        ty,
    }
}

pub fn type_check_syn_type_atom(
    ty_atom: &SynTypeAtom,
    rename_table: &SerialIdTable,
    _typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
) -> TypedTerm {
    let id = ty_atom.syn_tree_id();
    let id = *rename_table.get(id).unwrap();
    typed_term_table_for_atom.get(id).unwrap().clone()
}

pub fn type_check_syn_type_def(
    type_def: &SynTypeDef,
    rename_table: &SerialIdTable,
    typed_term_table: &mut TypedTermTable,
    typed_term_table_for_atom: &mut TypedTermTableForAtom,
    _type_def_table: &TypeDefTable,
) -> Result<(), TypeCheckError> {
    let typed_ty_ty = type_check_syn_type(
        &type_def.ty_ty,
        rename_table,
        typed_term_table,
        typed_term_table_for_atom,
    );
    let ty_ty_level = typed_ty_ty.term.level();
    // todo: check typed_ty_ty.ty is *

    // name
    {
        let id2 = type_def.name.syn_tree_id();
        let id2 = *rename_table.get(id2).unwrap();
        let term = TermAtom::new(ty_ty_level - 1, id2).into();
        let ty = typed_ty_ty.term;
        let typed_term = TypedTerm { term, ty };
        typed_term_table_for_atom.insert(id2, typed_term);
    }

    // variants
    for variant in &type_def.variants {
        let id2 = variant.name.syn_tree_id();
        let id2 = *rename_table.get(id2).unwrap();
        let typed_term = type_check_syn_type(
            &variant.ty,
            rename_table,
            typed_term_table,
            typed_term_table_for_atom,
        );
        let term = TermAtom::new(typed_term.term.level() - 1, id2).into();
        let ty = typed_term.term.clone();
        let typed_term = TypedTerm { term, ty };
        typed_term_table_for_atom.insert(id2, typed_term);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use felis_rename::{
        path_table::construct_path_table_syn_file, rename_defs::rename_defs_file,
        rename_uses::rename_uses_file, SerialId, SerialIdTable,
    };
    use felis_syn::test_utils::parse_from_str;
    use felis_term::{TermDependentMap, TermMap, TermStar};
    use felis_type_defs::gen_type_def_table_file;
    use neco_resolver::Resolver;

    use super::*;

    #[test]
    fn compute_apped_typed_term_test_1() {
        // { term: f, ty: (A : Prop) -> A } { term: X, ty: Prop }
        let f_id = SerialId::new();
        let a_id = SerialId::new();
        let prop_id = SerialId::new();
        let x_id = SerialId::new();
        let fun = TypedTerm {
            term: Term::Atom(TermAtom::new(0, f_id)),
            ty: Term::DependentMap(TermDependentMap {
                from: (
                    TermAtom::new(1, a_id),
                    Box::new(Term::Atom(TermAtom::new(2, prop_id))),
                ),
                to: Box::new(Term::Atom(TermAtom::new(1, a_id))),
            }),
        };
        let arg = TypedTerm {
            term: Term::Atom(TermAtom::new(1, x_id)),
            ty: Term::Atom(TermAtom::new(2, prop_id)),
        };
        let res = compute_apped_typed_term(&fun, &arg).unwrap();
        let expected = TypedTerm {
            term: Term::App(TermApp {
                fun: Box::new(Term::Atom(TermAtom::new(0, f_id))),
                arg: Box::new(Term::Atom(TermAtom::new(1, x_id))),
            }),
            ty: Term::Atom(TermAtom::new(1, x_id)),
        };
        assert_eq!(res, expected);
    }

    #[test]
    fn type_check_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id);
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
        let mut typed_term_table = TypedTermTable::new();
        typed_term_table_for_atom.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        let type_def_table = gen_type_def_table_file(&file, &rename_table);
        type_check_syn_file(
            &file,
            &rename_table,
            &mut typed_term_table,
            &mut typed_term_table_for_atom,
            &type_def_table,
        )
        .unwrap();
        assert_eq!(typed_term_table_for_atom.len(), 3);
        // Prop : *
        assert_eq!(
            typed_term_table_for_atom.get(prop_id).unwrap(),
            &TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            }
        );
        // A : Prop
        let SynFileItem::TypeDef(ref type_def) = file.items[0] else { panic!() };
        let a_id = *rename_table.get(type_def.name.syn_tree_id()).unwrap();
        assert_eq!(
            typed_term_table_for_atom.get(a_id).unwrap(),
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
            typed_term_table_for_atom.get(hoge_id).unwrap(),
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
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id);
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        /* merge def and use */
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut typed_term_table = TypedTermTable::new();
        let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
        typed_term_table_for_atom.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        let type_def_table = gen_type_def_table_file(&file, &rename_table);
        type_check_syn_file(
            &file,
            &rename_table,
            &mut typed_term_table,
            &mut typed_term_table_for_atom,
            &type_def_table,
        )
        .unwrap();
        assert_eq!(
            typed_term_table_for_atom.len(),
            5,
            "{:?}",
            typed_term_table_for_atom
        );
        // Prop : *
        assert_eq!(
            typed_term_table_for_atom.get(prop_id).unwrap(),
            &TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            }
        );
        // And : Prop -> Prop -> Prop
        let SynFileItem::TypeDef(ref type_def) = file.items[0] else { panic!() };
        let a_id = *rename_table.get(type_def.name.syn_tree_id()).unwrap();
        assert_eq!(
            typed_term_table_for_atom.get(a_id).unwrap(),
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
            typed_term_table_for_atom.get(conj_id).unwrap(),
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

    #[test]
    fn type_check_test_3() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or, or_introl, A, B, or_intror, A, B
        // (11) theorem1, A, B, proof, A, B, x, _, _, l, r
        assert_eq!(defs_table.len(), 23);
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        assert_eq!(path_table.len(), 3);
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id);
        path_table.setup_resolver(*defs_table.get(file.syn_tree_id()).unwrap(), &mut resolver);
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        // (3) Prop, Prop, Prop
        // (7) Prop, Prop, A, B, And, A, B
        // (3) Prop, Prop, Prop
        // (6) Prop, Prop, A, Or, A, B
        // (6) Prop, Prop, B, Or, A, B
        // (8) Prop, Prop, And A, B, Or, A, B
        // (8) Prop, Prop, And, A, B, Or, A, B
        // (1) x
        // (5) And::conj, Or::or_introl, A, B, l
        assert_eq!(uses_table.len(), 47);
        /* merge def and use */
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut typed_term_table = TypedTermTable::new();
        let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
        typed_term_table_for_atom.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        let type_def_table = gen_type_def_table_file(&file, &rename_table);
        type_check_syn_file(
            &file,
            &rename_table,
            &mut typed_term_table,
            &mut typed_term_table_for_atom,
            &type_def_table,
        )
        .unwrap();
        // (4) And, conj, A, B
        // (7) Or, or_introl, A, B, or_intror, A, B
        // (8) proof, A, B, x, _, _, l, r
        assert_eq!(typed_term_table_for_atom.len(), 19);
    }

    #[test]
    fn type_check_test_4() {
        let s = std::fs::read_to_string("../../library/wip/prop5.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id);
        path_table.setup_resolver(*defs_table.get(file.syn_tree_id()).unwrap(), &mut resolver);
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        /* merge def and use */
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut typed_term_table = TypedTermTable::new();
        let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
        typed_term_table_for_atom.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        let type_def_table = gen_type_def_table_file(&file, &rename_table);
        type_check_syn_file(
            &file,
            &rename_table,
            &mut typed_term_table,
            &mut typed_term_table_for_atom,
            &type_def_table,
        )
        .unwrap();
        // proof, A, B, C, f, g, x
        assert_eq!(typed_term_table_for_atom.len(), 7);
    }

    #[test]
    fn type_check_test_5() {
        let s = std::fs::read_to_string("../../library/wip/prop5_error_1.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id);
        path_table.setup_resolver(*defs_table.get(file.syn_tree_id()).unwrap(), &mut resolver);
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        /* merge def and use */
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut typed_term_table = TypedTermTable::new();
        let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
        typed_term_table_for_atom.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        let type_def_table = gen_type_def_table_file(&file, &rename_table);

        let t = file.items[0].clone();
        let SynFileItem::TheoremDef(theorem_def) = t else { panic!(); };
        let ty = theorem_def.fn_def.ty.clone();
        let SynType::DependentMap(dep_map) = ty else { panic!(); };
        let a_id = dep_map.from.name.syn_tree_id();
        let a_id = rename_table.get(a_id).unwrap().clone();
        let SynType::DependentMap(dep_map) = dep_map.to.as_ref().clone() else { panic!(); };
        let b_id = dep_map.from.name.syn_tree_id();
        let b_id = rename_table.get(b_id).unwrap().clone();

        let res = type_check_syn_file(
            &file,
            &rename_table,
            &mut typed_term_table,
            &mut typed_term_table_for_atom,
            &type_def_table,
        );
        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatched {
                expected_ty: Term::Atom(TermAtom { level: 1, id: b_id }),
                actual_ty: Term::Atom(TermAtom { level: 1, id: a_id }),
            })
        );
        // proof, A, B, C, f, g, x
        assert_eq!(typed_term_table_for_atom.len(), 7);
    }

    #[test]
    fn type_check_test_6() {
        let s = std::fs::read_to_string("../../library/wip/prop6.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id);
        path_table.setup_resolver(*defs_table.get(file.syn_tree_id()).unwrap(), &mut resolver);
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        /* merge def and use */
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut typed_term_table = TypedTermTable::new();
        let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
        typed_term_table_for_atom.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        let type_def_table = gen_type_def_table_file(&file, &rename_table);
        type_check_syn_file(
            &file,
            &rename_table,
            &mut typed_term_table,
            &mut typed_term_table_for_atom,
            &type_def_table,
        )
        .unwrap();
        // (5) Exists, exists, A, p, x
        // (6) proof, A, B, p, e, f
        // (3) _, _, x
        assert_eq!(typed_term_table_for_atom.len(), 14);
    }

    #[test]
    fn type_check_test_7() {
        let s = std::fs::read_to_string("../../library/wip/prop6_error_1.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        /* use */
        let mut resolver = Resolver::new();
        let prop_id = SerialId::new();
        resolver.set("Prop".to_string(), prop_id);
        path_table.setup_resolver(*defs_table.get(file.syn_tree_id()).unwrap(), &mut resolver);
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        /* merge def and use */
        let mut rename_table = SerialIdTable::new();
        rename_table.merge_mut(defs_table);
        rename_table.merge_mut(uses_table);
        let mut typed_term_table = TypedTermTable::new();
        let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
        typed_term_table_for_atom.insert(
            prop_id,
            TypedTerm {
                term: Term::Atom(TermAtom::new(2, prop_id)),
                ty: TermStar::new(3).into(),
            },
        );
        let type_def_table = gen_type_def_table_file(&file, &rename_table);

        let t = file.items[1].clone();
        let SynFileItem::TheoremDef(theorem_def) = t else { panic!(); };
        let ty = theorem_def.fn_def.ty.clone();
        let SynType::DependentMap(dep_map) = ty else { panic!(); };
        let a_id = dep_map.from.name.syn_tree_id();
        let a_id = rename_table.get(a_id).unwrap().clone();
        let SynType::DependentMap(dep_map) = dep_map.to.as_ref().clone() else { panic!(); };
        let b_id = dep_map.from.name.syn_tree_id();
        let b_id = rename_table.get(b_id).unwrap().clone();

        let res = type_check_syn_file(
            &file,
            &rename_table,
            &mut typed_term_table,
            &mut typed_term_table_for_atom,
            &type_def_table,
        );
        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatched {
                expected_ty: Term::Atom(TermAtom { level: 1, id: b_id }),
                actual_ty: Term::Atom(TermAtom { level: 1, id: a_id }),
            })
        );
        // (5) Exists, exists, A, p, x
        // (6) proof, A, B, p, e, f
        // (3) _, _, x
        assert_eq!(typed_term_table_for_atom.len(), 14);
    }
}
