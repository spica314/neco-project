use felis_syn::{
    syn_expr::SynExpr,
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_formula::SynFormula,
    syn_statement::SynStatement,
    syn_theorem_def::SynTheoremDef,
    syn_type::SynType,
    syn_type_def::{SynTypeDef, SynVariant},
};

use crate::{SerialId, SerialIdTable};

pub fn rename_defs_file(file: &SynFile) -> Result<SerialIdTable, ()> {
    let mut res = SerialIdTable::new();
    res.insert(file.syn_tree_id(), SerialId::new());
    for item in &file.items {
        let table = rename_defs_file_item(item)?;
        res.merge_mut(table);
    }
    Ok(res)
}

fn rename_defs_file_item(item: &SynFileItem) -> Result<SerialIdTable, ()> {
    match item {
        SynFileItem::TypeDef(type_def) => rename_defs_type_def(type_def),
        SynFileItem::FnDef(fn_def) => rename_defs_fn_def(fn_def),
        SynFileItem::TheoremDef(theorem_def) => rename_defs_theorem_def(theorem_def),
        SynFileItem::Entrypoint(_) => Ok(SerialIdTable::new()),
    }
}

fn rename_defs_type_def(type_def: &SynTypeDef) -> Result<SerialIdTable, ()> {
    let mut table = SerialIdTable::new();
    // name
    {
        let id = SerialId::new();
        table.insert(type_def.name.syn_tree_id(), id);
    }
    // variants
    for variant in &type_def.variants {
        let t = rename_defs_variant(variant).unwrap();
        table.merge_mut(t);
    }
    Ok(table)
}

fn rename_defs_variant(variant: &SynVariant) -> Result<SerialIdTable, ()> {
    let mut table = SerialIdTable::new();

    let id = SerialId::new();
    table.insert(variant.name.syn_tree_id(), id);

    let t = rename_defs_type(&variant.ty).unwrap();
    table.merge_mut(t);

    Ok(table)
}

fn rename_defs_fn_def(fn_def: &SynFnDef) -> Result<SerialIdTable, ()> {
    let mut table = SerialIdTable::new();
    // name
    {
        let id = SerialId::new();
        table.insert(fn_def.name.syn_tree_id(), id);
    }
    // ty
    {
        let t = rename_defs_type(&fn_def.ty).unwrap();
        table.merge_mut(t);
    }
    // statements
    for statement in &fn_def.fn_block.statements {
        match statement {
            SynStatement::Expr(expr) => {
                let table2 = rename_defs_expr(expr)?;
                table.merge_mut(table2);
            }
            SynStatement::Let(_) => todo!(),
        }
    }
    Ok(table)
}

fn rename_defs_expr(expr: &SynExpr) -> Result<SerialIdTable, ()> {
    let mut table = SerialIdTable::new();
    match expr {
        SynExpr::Ident(_) => {}
        SynExpr::App(_) => {}
        SynExpr::Match(expr_match) => {
            for arm in &expr_match.arms {
                for ident in arm.pattern.idents.iter() {
                    let id = SerialId::new();
                    table.insert(ident.syn_tree_id(), id);
                }
            }
        }
        SynExpr::Paren(_) => {}
        SynExpr::IdentWithPath(_) => {}
        SynExpr::String(_) => todo!(),
        SynExpr::Block(_) => todo!(),
    }
    Ok(table)
}

fn rename_defs_formula(formula: &SynFormula) -> Result<SerialIdTable, ()> {
    let mut table = SerialIdTable::new();
    match formula {
        SynFormula::Forall(forall) => {
            let id = SerialId::new();
            table.insert(forall.name.syn_tree_id(), id);

            let table2 = rename_defs_formula(&forall.child)?;
            table.merge_mut(table2);
        }
        SynFormula::App(_) => {}
        SynFormula::Atom(_) => {}
        SynFormula::Paren(_) => {}
        SynFormula::Implies(implies) => {
            let table2 = rename_defs_formula(&implies.lhs)?;
            table.merge_mut(table2);
            let table2 = rename_defs_formula(&implies.rhs)?;
            table.merge_mut(table2);
        }
    }
    Ok(table)
}

fn rename_defs_theorem_def(theorem_def: &SynTheoremDef) -> Result<SerialIdTable, ()> {
    let mut table = SerialIdTable::new();
    // name
    {
        let id = SerialId::new();
        table.insert(theorem_def.name.syn_tree_id(), id);
    }
    // ty
    {
        let table2 = rename_defs_formula(&theorem_def.formula)?;
        table.merge_mut(table2);
    }
    // fn_def
    {
        let table2 = rename_defs_fn_def(&theorem_def.fn_def)?;
        table.merge_mut(table2);
    }
    Ok(table)
}

fn rename_defs_type(ty: &SynType) -> Result<SerialIdTable, ()> {
    let mut table = SerialIdTable::new();
    match ty {
        SynType::App(_) => {}
        SynType::Atom(_) => {}
        SynType::Map(_) => {}
        SynType::Paren(_) => {}
        SynType::DependentMap(dep_map) => {
            let id = SerialId::new();
            table.insert(dep_map.from.name.syn_tree_id(), id);

            let table2 = rename_defs_type(&dep_map.to)?;
            table.merge_mut(table2);
        }
        SynType::Unit(_) => {}
    }
    Ok(table)
}

#[cfg(test)]
mod test {
    use super::*;
    use felis_syn::test_utils::parse_from_str;

    #[test]
    fn felis_rename_defs_type_def_test_1() {
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
        // [file], A, hoge
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn felis_rename_defs_file_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let table = rename_defs_file(&file).unwrap();
        // [file], proof, A, B, x, l, r
        assert_eq!(table.len(), 7);
    }

    #[test]
    fn felis_rename_defs_file_test_3() {
        let s = std::fs::read_to_string("../../library/wip/fn_def_simple.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let table = rename_defs_file(&file).unwrap();
        // [file], proof, A, x
        assert_eq!(table.len(), 4);
    }

    #[test]
    fn felis_rename_defs_file_test_4() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        let table = rename_defs_file(&file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or or_introl, A, B, or_intror, A, B
        // (11) theorem1, A, B, proof, A, B, x, _, _, l, r
        assert_eq!(table.len(), 23);
    }
}
