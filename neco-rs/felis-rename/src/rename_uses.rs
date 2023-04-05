use felis_syn::{
    syn_expr::{SynExpr, SynExprApp, SynExprIdent, SynExprIdentWithPath, SynExprMatch},
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::{SynFnBlock, SynFnDef, SynStatement},
    syn_theorem_def::SynTheoremDef,
    syn_type::{
        SynType, SynTypeApp, SynTypeAtom, SynTypeDependentMap, SynTypeForall, SynTypeMap,
        SynTypeParen,
    },
    syn_type_def::SynTypeDef,
    syn_typed_arg::SynTypedArg,
};
use neco_resolver::Resolver;

use crate::{path_table::PathTable, SerialId, SerialIdTable};

pub fn rename_uses_file(
    file: &SynFile,
    defs_table: &SerialIdTable,
    mut resolver: Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    for item in &file.items {
        let table = rename_uses_file_item(item, defs_table, &mut resolver, path_table)?;
        res.merge_mut(table);
    }
    Ok(res)
}

fn rename_uses_file_item(
    item: &SynFileItem,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    match item {
        SynFileItem::TypeDef(type_def) => {
            let table = rename_uses_type_def(type_def, defs_table, resolver, path_table)?;
            res.merge_mut(table);
        }
        SynFileItem::FnDef(fn_def) => {
            let table = rename_uses_fn_def(fn_def, defs_table, resolver, path_table)?;
            res.merge_mut(table);
        }
        SynFileItem::TheoremDef(theorem_def) => {
            let table = rename_uses_theorem_def(theorem_def, defs_table, resolver, path_table)?;
            res.merge_mut(table);
        }
    }
    Ok(res)
}

fn rename_uses_type_def(
    type_def: &SynTypeDef,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    resolver.enter_scope();
    // ty_ty
    {
        let table = rename_uses_type(&type_def.ty_ty, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    // name
    {
        let a = type_def.name.as_str();
        let b = type_def.name.syn_tree_id();
        let c = defs_table.get(b).unwrap();
        resolver.set(a.to_string(), *c).unwrap();
    }
    // variants
    for variant in &type_def.variants {
        resolver.enter_scope();
        let table = rename_uses_type(&variant.ty, defs_table, resolver, path_table)?;
        res.merge_mut(table);
        resolver.leave_scope();
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_fn_def(
    fn_def: &SynFnDef,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    resolver.enter_scope();
    let mut res = SerialIdTable::new();
    // ty
    {
        let table = rename_uses_type(&fn_def.ty, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    // block
    {
        let table = rename_uses_fn_block(&fn_def.fn_block, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_fn_block(
    fn_block: &SynFnBlock,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    resolver.enter_scope();
    for statement in &fn_block.statements {
        let table = rename_uses_statement(statement, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_statement(
    statement: &SynStatement,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    match statement {
        SynStatement::Expr(expr) => rename_uses_expr(expr, defs_table, resolver, path_table),
    }
}

fn rename_uses_expr(
    expr: &SynExpr,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    match expr {
        SynExpr::Ident(expr_ident) => {
            rename_uses_expr_ident(expr_ident, defs_table, resolver, path_table)
        }
        SynExpr::App(expr_app) => rename_uses_expr_app(expr_app, defs_table, resolver, path_table),
        SynExpr::Match(expr_match) => {
            rename_uses_expr_match(expr_match, defs_table, resolver, path_table)
        }
        SynExpr::Paren(_expr_paren) => todo!(),
        SynExpr::IdentWithPath(expr_ident_with_path) => {
            rename_uses_expr_ident_with_path(expr_ident_with_path, defs_table, resolver, path_table)
        }
    }
}

fn rename_uses_expr_app(
    expr_app: &SynExprApp,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    for expr in &expr_app.exprs {
        let table = rename_uses_expr(expr, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    Ok(res)
}

fn rename_uses_expr_match(
    expr_match: &SynExprMatch,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    // 1
    {
        let table = rename_uses_expr(&expr_match.expr, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    for arm in &expr_match.arms {
        resolver.enter_scope();
        // constructor
        {
            let b = arm.pattern.type_constructor.ident.syn_tree_id();
            let mut xs: Vec<_> = arm
                .pattern
                .type_constructor
                .path
                .iter()
                .map(|t| t.0.clone())
                .collect();
            xs.push(arm.pattern.type_constructor.ident.clone());
            let mut id = *resolver.get(xs[0].as_str()).unwrap();
            for next in &xs[1..] {
                id = *path_table
                    .get(id)
                    .unwrap()
                    .children
                    .get(next.as_str())
                    .unwrap();
            }
            res.insert(b, id);
        }
        // constructor args
        for ident in arm.pattern.idents.iter() {
            let a = ident.as_str();
            let b = ident.syn_tree_id();
            let c = defs_table.get(b).unwrap();
            resolver.set(a.to_string(), *c).unwrap();
        }
        // expr
        {
            let table = rename_uses_expr(&arm.expr, defs_table, resolver, path_table).unwrap();
            res.merge_mut(table);
        }
        resolver.leave_scope();
    }
    Ok(res)
}

fn rename_uses_expr_ident(
    expr_ident: &SynExprIdent,
    _defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    _path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    let a = expr_ident.ident.as_str();
    let b = expr_ident.ident.syn_tree_id();
    let Some(c) = resolver.get(a) else {
        panic!("unknown ident {a}");
    };
    res.insert(b, *c);
    Ok(res)
}

fn rename_uses_expr_ident_with_path(
    expr_ident_with_path: &SynExprIdentWithPath,
    _defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    let b = expr_ident_with_path.ident.syn_tree_id();
    let mut xs: Vec<_> = expr_ident_with_path
        .path
        .iter()
        .map(|t| t.0.clone())
        .collect();
    xs.push(expr_ident_with_path.ident.clone());
    let mut id = *resolver.get(xs[0].as_str()).unwrap();
    for next in &xs[1..] {
        id = *path_table
            .get(id)
            .unwrap()
            .children
            .get(next.as_str())
            .unwrap();
    }
    res.insert(b, id);
    Ok(res)
}

fn rename_uses_theorem_def(
    theorem_def: &SynTheoremDef,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    resolver.enter_scope();
    // ty
    {
        let table = rename_uses_type(&theorem_def.ty, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    // fn
    {
        let table = rename_uses_fn_def(&theorem_def.fn_def, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_type(
    ty: &SynType,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    match ty {
        SynType::Forall(type_forall) => {
            rename_uses_type_forall(type_forall, defs_table, resolver, path_table)
        }
        SynType::App(type_app) => rename_uses_type_app(type_app, defs_table, resolver, path_table),
        SynType::Atom(type_atom) => {
            rename_uses_type_atom(type_atom, defs_table, resolver, path_table)
        }
        SynType::Map(type_map) => rename_uses_type_map(type_map, defs_table, resolver, path_table),
        SynType::Paren(type_paren) => {
            rename_uses_type_paren(type_paren, defs_table, resolver, path_table)
        }
        SynType::DependentMap(type_dep_map) => {
            rename_uses_type_dep_map(type_dep_map, defs_table, resolver, path_table)
        }
    }
}

fn rename_uses_typed_arg(
    typed_arg: &SynTypedArg,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    // 1
    {
        let table = rename_uses_type(&typed_arg.ty, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }

    // 2
    {
        let a = typed_arg.name.syn_tree_id();
        let b = typed_arg.name.as_str().to_string();
        let Some(c) = defs_table.get(a) else {
            panic!();
            // return Err(());
        };
        resolver.set(b, *c).unwrap();
    }
    Ok(res)
}

// #forall (name^{2,def} : <Type>^{1}), <Type>^{3}
fn rename_uses_type_forall(
    type_forall: &SynTypeForall,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    // 1
    {
        let table =
            rename_uses_typed_arg(&type_forall.typed_arg, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }

    // 3
    {
        let table = rename_uses_type(&type_forall.ty, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }

    Ok(res)
}

// X^{1} X^{2}
fn rename_uses_type_app(
    type_app: &SynTypeApp,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    // 1
    {
        let table = rename_uses_type(&type_app.left, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    // 2
    {
        let table = rename_uses_type(&type_app.right, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    Ok(res)
}

// X^{use,1}
fn rename_uses_type_atom(
    type_atom: &SynTypeAtom,
    _defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    _path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    let Some(id) = resolver.get(type_atom.ident.as_str()) else {
        panic!("unknown name : {}", type_atom.ident.as_str());
        // return Err(());
    };
    res.insert(type_atom.ident.syn_tree_id(), *id);
    Ok(res)
}

// X^{1} -> Y^{2}
fn rename_uses_type_map(
    type_map: &SynTypeMap,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    // 1
    {
        let table = rename_uses_type(&type_map.from, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    // 2
    {
        let table = rename_uses_type(&type_map.to, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    Ok(res)
}

// (X^{2} : Y^{1}) -> Z^{3}
fn rename_uses_type_dep_map(
    type_dep_map: &SynTypeDependentMap,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    // 1
    {
        let table = rename_uses_type(&type_dep_map.from.ty, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    // 2
    {
        let a = type_dep_map.from.name.syn_tree_id();
        let b = type_dep_map.from.name.as_str().to_string();
        let Some(c) = defs_table.get(a) else {
            panic!();
        };
        resolver.set(b, *c).unwrap();
    }
    // 3
    {
        let table = rename_uses_type(&type_dep_map.to, defs_table, resolver, path_table)?;
        res.merge_mut(table);
    }
    Ok(res)
}

// X^{1}
fn rename_uses_type_paren(
    type_paren: &SynTypeParen,
    defs_table: &SerialIdTable,
    resolver: &mut Resolver<SerialId>,
    path_table: &PathTable,
) -> Result<SerialIdTable, ()> {
    rename_uses_type(&type_paren.ty, defs_table, resolver, path_table)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{path_table::construct_path_table_syn_file, rename_defs::*};
    use felis_syn::test_utils::parse_from_str;

    #[test]
    fn felis_rename_uses_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // [file], A, hoge
        assert_eq!(defs_table.len(), 3);
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        assert_eq!(path_table.len(), 2);
        /* use */
        let mut resolver = Resolver::new();
        let a = SerialId::new();
        resolver.set("Prop".to_string(), a).unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        // Prop, A
        assert_eq!(uses_table.len(), 2);
    }

    #[test]
    fn felis_rename_uses_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // [file], proof, A, B, x, l, r
        assert_eq!(defs_table.len(), 7);
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        assert_eq!(path_table.len(), 1);
        /* use */
        let mut resolver = Resolver::new();
        resolver.set("Prop".to_string(), SerialId::new()).unwrap();
        resolver.set("And".to_string(), SerialId::new()).unwrap();
        resolver.set("Or".to_string(), SerialId::new()).unwrap();
        resolver.set("conj".to_string(), SerialId::new()).unwrap();
        resolver
            .set("or_introl".to_string(), SerialId::new())
            .unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        // Prop, Prop, And, A, B, Or, A, B, x, conj, or_introl, l
        assert_eq!(uses_table.len(), 12);
    }

    #[test]
    fn felis_rename_uses_test_4() {
        let s = std::fs::read_to_string("../../library/wip/and2.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // [file], And, conj, A, B
        assert_eq!(defs_table.len(), 5);
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        assert_eq!(path_table.len(), 2);
        /* use */
        let mut resolver = Resolver::new();
        resolver.set("Prop".to_string(), SerialId::new()).unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
        // (3) Prop, Prop, Prop
        // (7) Prop, Prop, A, B, And, A, B
        assert_eq!(uses_table.len(), 10);
    }

    #[test]
    fn felis_rename_uses_test_5() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or, or_introl, A, B, or_intror, A, B
        // (9) theorem1, A, B, proof, A, B, x, l, r
        assert_eq!(defs_table.len(), 21);
        /* path */
        let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
        assert_eq!(path_table.len(), 3);
        /* use */
        let mut resolver = Resolver::new();
        resolver.set("Prop".to_string(), SerialId::new()).unwrap();
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
        // (3) And::conj, Or::or_introl, l
        assert_eq!(uses_table.len(), 45);
    }
}
