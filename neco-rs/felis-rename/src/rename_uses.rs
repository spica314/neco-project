use felis_syn::{
    syn_expr::{SynExpr, SynExprApp, SynExprIdent, SynExprMatch},
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::{SynFnBlock, SynFnDef, SynStatement},
    syn_theorem_def::SynTheoremDef,
    syn_type::{SynType, SynTypeApp, SynTypeAtom, SynTypeForall, SynTypeMap, SynTypeParen},
    syn_type_def::SynTypeDef,
    syn_typed_arg::SynTypedArg,
};
use neco_resolver::Resolver;
use neco_table::{Table, TableId};

pub fn rename_uses_file(
    file: &SynFile,
    defs_table: &Table<TableId>,
    mut resolver: Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    for item in &file.items {
        let table = rename_uses_file_item(item, defs_table, &mut resolver)?;
        res.merge_mut(table);
    }
    Ok(res)
}

fn rename_uses_file_item(
    item: &SynFileItem,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    match item {
        SynFileItem::TypeDef(type_def) => {
            let table = rename_uses_type_def(type_def, defs_table, resolver)?;
            res.merge_mut(table);
        }
        SynFileItem::FnDef(fn_def) => {
            let table = rename_uses_fn_def(fn_def, defs_table, resolver)?;
            res.merge_mut(table);
        }
        SynFileItem::TheoremDef(theorem_def) => {
            let table = rename_uses_theorem_def(theorem_def, defs_table, resolver)?;
            res.merge_mut(table);
        }
    }
    Ok(res)
}

fn rename_uses_type_def(
    type_def: &SynTypeDef,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    resolver.enter_scope();
    // ty_ty
    {
        let table = rename_uses_type(&type_def.ty_ty, defs_table, resolver)?;
        res.merge_mut(table);
    }
    // name
    {
        let a = type_def.name.as_str();
        let b = type_def.name.table_id();
        let c = defs_table.get(b).unwrap();
        resolver.set(a.to_string(), *c).unwrap();
    }
    // args
    for arg in &type_def.args {
        let a = arg.name.as_str();
        let b = arg.name.table_id();
        resolver.set(a.to_string(), b).unwrap();
        let table = rename_uses_type(&arg.ty, defs_table, resolver)?;
        res.merge_mut(table);
    }
    // variants
    for variant in &type_def.variants {
        resolver.enter_scope();
        let table = rename_uses_type(&variant.ty, defs_table, resolver)?;
        res.merge_mut(table);
        resolver.leave_scope();
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_fn_def(
    fn_def: &SynFnDef,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    resolver.enter_scope();
    let mut res = Table::new();
    // args
    for arg in &fn_def.args {
        let table = rename_uses_typed_arg(arg, defs_table, resolver)?;
        res.merge_mut(table);
    }
    // ret_ty
    {
        let table = rename_uses_type(&fn_def.res_ty, defs_table, resolver)?;
        res.merge_mut(table);
    }
    // block
    {
        let table = rename_uses_fn_block(&fn_def.fn_block, defs_table, resolver)?;
        res.merge_mut(table);
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_fn_block(
    fn_block: &SynFnBlock,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    resolver.enter_scope();
    for statement in &fn_block.statements {
        let table = rename_uses_statement(statement, defs_table, resolver)?;
        res.merge_mut(table);
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_statement(
    statement: &SynStatement,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    match statement {
        SynStatement::Expr(expr) => rename_uses_expr(expr, defs_table, resolver),
    }
}

fn rename_uses_expr(
    expr: &SynExpr,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    match expr {
        SynExpr::Ident(expr_ident) => rename_uses_expr_ident(expr_ident, defs_table, resolver),
        SynExpr::App(expr_app) => rename_uses_expr_app(expr_app, defs_table, resolver),
        SynExpr::Match(expr_match) => rename_uses_expr_match(expr_match, defs_table, resolver),
        SynExpr::Paren(_expr_paren) => todo!(),
    }
}

fn rename_uses_expr_app(
    expr_app: &SynExprApp,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    for expr in &expr_app.exprs {
        let table = rename_uses_expr(expr, defs_table, resolver)?;
        res.merge_mut(table);
    }
    Ok(res)
}

fn rename_uses_expr_match(
    expr_match: &SynExprMatch,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    // 1
    {
        let table = rename_uses_expr(&expr_match.expr, defs_table, resolver)?;
        res.merge_mut(table);
    }
    for arm in &expr_match.arms {
        resolver.enter_scope();
        // constructor
        {
            let a = arm.pattern.idents[0].as_str();
            let b = arm.pattern.idents[0].table_id();
            let c = resolver.get(a).unwrap();
            res.insert(b, *c);
        }
        // constructor args
        for ident in arm.pattern.idents.iter().skip(1) {
            let a = ident.as_str();
            let b = ident.table_id();
            let c = defs_table.get(b).unwrap();
            resolver.set(a.to_string(), *c).unwrap();
        }
        // expr
        {
            let table = rename_uses_expr(&arm.expr, defs_table, resolver).unwrap();
            res.merge_mut(table);
        }
        resolver.leave_scope();
    }
    Ok(res)
}

fn rename_uses_expr_ident(
    expr_ident: &SynExprIdent,
    _defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    let a = expr_ident.ident.as_str();
    let b = expr_ident.ident.table_id();
    let Some(c) = resolver.get(a) else {
        panic!("unknown ident {}", a);
    };
    res.insert(b, *c);
    Ok(res)
}

fn rename_uses_theorem_def(
    theorem_def: &SynTheoremDef,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    resolver.enter_scope();
    // ty
    {
        let table = rename_uses_type(&theorem_def.ty, defs_table, resolver)?;
        res.merge_mut(table);
    }
    // fn
    {
        let table = rename_uses_fn_def(&theorem_def.fn_def, defs_table, resolver)?;
        res.merge_mut(table);
    }
    resolver.leave_scope();
    Ok(res)
}

fn rename_uses_type(
    ty: &SynType,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    match ty {
        SynType::Forall(type_forall) => rename_uses_type_forall(type_forall, defs_table, resolver),
        SynType::App(type_app) => rename_uses_type_app(type_app, defs_table, resolver),
        SynType::Atom(type_atom) => rename_uses_type_atom(type_atom, defs_table, resolver),
        SynType::Map(type_map) => rename_uses_type_map(type_map, defs_table, resolver),
        SynType::Paren(type_paren) => rename_uses_type_paren(type_paren, defs_table, resolver),
    }
}

fn rename_uses_typed_arg(
    typed_arg: &SynTypedArg,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    // 1
    {
        let table = rename_uses_type(&typed_arg.ty, defs_table, resolver)?;
        res.merge_mut(table);
    }

    // 2
    {
        let a = typed_arg.name.table_id();
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
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    // 1
    {
        let table = rename_uses_typed_arg(&type_forall.typed_arg, defs_table, resolver)?;
        res.merge_mut(table);
    }

    // 3
    {
        let table = rename_uses_type(&type_forall.ty, defs_table, resolver)?;
        res.merge_mut(table);
    }

    Ok(res)
}

// X^{1} X^{2}
fn rename_uses_type_app(
    type_app: &SynTypeApp,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    // 1
    {
        let table = rename_uses_type(&type_app.left, defs_table, resolver)?;
        res.merge_mut(table);
    }
    // 2
    {
        let table = rename_uses_type(&type_app.right, defs_table, resolver)?;
        res.merge_mut(table);
    }
    Ok(res)
}

// X^{use,1}
fn rename_uses_type_atom(
    type_atom: &SynTypeAtom,
    _defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    let Some(id) = resolver.get(type_atom.ident.as_str()) else {
        panic!("unknown name : {}", type_atom.ident.as_str());
        // return Err(());
    };
    res.insert(type_atom.ident.table_id(), *id);
    Ok(res)
}

// X^{1} -> Y^{2}
fn rename_uses_type_map(
    type_map: &SynTypeMap,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    // 1
    {
        let table = rename_uses_type(&type_map.from, defs_table, resolver)?;
        res.merge_mut(table);
    }
    // 2
    {
        let table = rename_uses_type(&type_map.to, defs_table, resolver)?;
        res.merge_mut(table);
    }
    Ok(res)
}

// X^{1}
fn rename_uses_type_paren(
    type_paren: &SynTypeParen,
    defs_table: &Table<TableId>,
    resolver: &mut Resolver<TableId>,
) -> Result<Table<TableId>, ()> {
    rename_uses_type(&type_paren.ty, defs_table, resolver)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::rename_defs::*;
    use felis_syn::test_utils::parse_from_str;

    #[test]
    fn felis_rename_uses_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // A, hoge
        assert_eq!(defs_table.len(), 2);
        /* use */
        let mut resolver = Resolver::new();
        let a = TableId::new();
        resolver.set("Prop".to_string(), a).unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver).unwrap();
        // Prop, A
        assert_eq!(uses_table.len(), 2);
    }

    #[test]
    fn felis_rename_uses_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // proof, A, B, x, l, r
        assert_eq!(defs_table.len(), 6);
        /* use */
        let mut resolver = Resolver::new();
        resolver.set("Prop".to_string(), TableId::new()).unwrap();
        resolver.set("And".to_string(), TableId::new()).unwrap();
        resolver.set("Or".to_string(), TableId::new()).unwrap();
        resolver.set("conj".to_string(), TableId::new()).unwrap();
        resolver
            .set("or_introl".to_string(), TableId::new())
            .unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver).unwrap();
        // Prop, Prop, And, A, B, Or, A, B, x, conj, or_introl, l
        assert_eq!(uses_table.len(), 12);
    }

    #[test]
    fn felis_rename_uses_test_3() {
        let s = std::fs::read_to_string("../../library/wip/prop2.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        /* def */
        let defs_table = rename_defs_file(&file).unwrap();
        // And, A, B, conj, Or, A, B, or_introl, or_intror, theorem1, A, B, proof, A, B, x, l, r
        assert_eq!(defs_table.len(), 18);
        /* use */
        let mut resolver = Resolver::new();
        resolver.set("Prop".to_string(), TableId::new()).unwrap();
        resolver.set("And".to_string(), TableId::new()).unwrap();
        resolver.set("Or".to_string(), TableId::new()).unwrap();
        resolver.set("conj".to_string(), TableId::new()).unwrap();
        resolver
            .set("or_introl".to_string(), TableId::new())
            .unwrap();
        let uses_table = rename_uses_file(&file, &defs_table, resolver).unwrap();
        // (8) Prop, Prop, Prop, A, B, And A, B,
        // (11) Prop, Prop, Prop, A, Or, A, B, B, Or, A, B,
        // (20) Prop, Prop, And, A, B, Or, A, B, Prop, Prop, And, A, B, Or, A, B, x, conj, or_introl, l
        assert_eq!(uses_table.len(), 39);
    }
}
