use felis_syn::{
    syn_expr::SynExpr,
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_theorem_def::SynTheoremDef,
    syn_type::SynType,
    syn_type_def::SynTypeDef,
};
use neco_table::{Table, TableId};

pub fn rename_defs_file(file: &SynFile) -> Result<Table<TableId>, ()> {
    let mut res = Table::new();
    for item in &file.items {
        let table = rename_defs_file_item(item)?;
        res.merge_mut(table);
    }
    Ok(res)
}

pub fn rename_defs_file_item(item: &SynFileItem) -> Result<Table<TableId>, ()> {
    match item {
        SynFileItem::TypeDef(type_def) => rename_defs_type_def(type_def),
        SynFileItem::FnDef(fn_def) => rename_defs_fn_def(fn_def),
        SynFileItem::TheoremDef(theorem_def) => rename_defs_theorem_def(theorem_def),
    }
}

pub fn rename_defs_type_def(type_def: &SynTypeDef) -> Result<Table<TableId>, ()> {
    let mut table = Table::new();
    // name
    {
        let id = TableId::new();
        table.insert(type_def.name.table_id(), id);
    }
    // args
    for arg in &type_def.args {
        let id = TableId::new();
        table.insert(arg.name.table_id(), id);
    }
    // variants
    for variant in &type_def.variants {
        let id = TableId::new();
        table.insert(variant.name.table_id(), id);
    }
    eprintln!("len = {}", table.len());
    Ok(table)
}

pub fn rename_defs_fn_def(fn_def: &SynFnDef) -> Result<Table<TableId>, ()> {
    let mut table = Table::new();
    // name
    {
        let id = TableId::new();
        table.insert(fn_def.name.table_id(), id);
    }
    // args
    for arg in &fn_def.args {
        let id = TableId::new();
        table.insert(arg.name.table_id(), id);
    }
    // statements
    for statement in &fn_def.fn_block.statements {
        match statement {
            felis_syn::syn_fn_def::SynStatement::Expr(expr) => {
                let table2 = rename_defs_expr(expr)?;
                table.merge_mut(table2);
            }
        }
    }
    Ok(table)
}

pub fn rename_defs_expr(expr: &SynExpr) -> Result<Table<TableId>, ()> {
    let mut table = Table::new();
    match expr {
        SynExpr::Ident(_) => {}
        SynExpr::App(_) => {}
        SynExpr::Match(expr_match) => {
            for arm in &expr_match.arms {
                for ident in arm.pattern.idents.iter().skip(1) {
                    let id = TableId::new();
                    table.insert(ident.table_id(), id);
                }
            }
        }
        SynExpr::Paren(_) => {}
    }
    Ok(table)
}

pub fn rename_defs_theorem_def(theorem_def: &SynTheoremDef) -> Result<Table<TableId>, ()> {
    let mut table = Table::new();
    // name
    {
        let id = TableId::new();
        table.insert(theorem_def.name.table_id(), id);
    }
    // ty
    {
        let table2 = rename_defs_type(&theorem_def.ty)?;
        table.merge_mut(table2);
    }
    // fn_def
    {
        let table2 = rename_defs_fn_def(&theorem_def.fn_def)?;
        table.merge_mut(table2);
    }
    Ok(table)
}

pub fn rename_defs_type(ty: &SynType) -> Result<Table<TableId>, ()> {
    let mut table = Table::new();
    match ty {
        SynType::Forall(type_forall) => {
            let id = TableId::new();
            table.insert(type_forall.typed_arg.name.table_id(), id);

            let table2 = rename_defs_type(&type_forall.ty)?;
            table.merge_mut(table2);
        }
        SynType::App(_) => {}
        SynType::Atom(_) => {}
        SynType::Map(_) => {}
        SynType::Paren(_) => {}
    }
    Ok(table)
}

#[cfg(test)]
mod test {
    use super::*;
    use felis_syn::test_utils::parse_from_str;

    #[test]
    fn felis_raename_defs_type_def_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let type_def = parse_from_str::<SynTypeDef>(&s).unwrap().unwrap();
        let table = rename_defs_type_def(&type_def).unwrap();
        // A, hoge
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn felis_rename_defs_file_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let table = rename_defs_file(&file).unwrap();
        // A, hoge
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn felis_rename_defs_file_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let table = rename_defs_file(&file).unwrap();
        // proof, A, B, x, l, r
        assert_eq!(table.len(), 6);
    }

    #[test]
    fn felis_rename_defs_file_test_3() {
        let s = std::fs::read_to_string("../../library/wip/prop2.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let table = rename_defs_file(&file).unwrap();
        // And, A, B, conj, Or, A, B, or_introl, or_intror, theorem1, A, B, proof, A, B, x, l, r
        assert_eq!(table.len(), 18);
    }
}
