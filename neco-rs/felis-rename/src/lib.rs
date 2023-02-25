use felis_package::FelisPackage;
use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_theorem_def::SynTheoremDef,
    syn_type::SynType,
    syn_type_def::SynTypeDef,
};
use neco_resolver::Resolver;
use neco_table::{Table, TableId};

struct Context<'a> {
    dependencies: &'a [&'a FelisPackage],
    type_resolver: Resolver<SynType>,
    ident_unique_id_table: Table<TableId>,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            dependencies: &[],
            type_resolver: Resolver::new(),
            ident_unique_id_table: Table::new(),
        }
    }
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn felis_rename(package: &mut FelisPackage, dependencies: &[&FelisPackage]) -> Result<(), ()> {
    let mut context = Context::new();
    felis_rename_defs_file(&mut context, &mut package.file)?;
    felis_rename_uses_file(&mut context, &mut package.file)?;
    Ok(())
}

fn felis_rename_defs_file(context: &mut Context, file: &mut SynFile) -> Result<(), ()> {
    for item in file.items.iter_mut() {
        felis_rename_defs_file_item(context, item)?;
    }
    Ok(())
}

fn felis_rename_defs_file_item(context: &mut Context, item: &mut SynFileItem) -> Result<(), ()> {
    match item {
        SynFileItem::TypeDef(type_def) => felis_rename_defs_type_def(context, type_def)?,
        SynFileItem::FnDef(fn_def) => felis_rename_defs_fn_def(context, fn_def)?,
        SynFileItem::TheoremDef(theorem_def) => {
            felis_rename_defs_theorem_def(context, theorem_def)?
        }
    }
    Ok(())
}

fn felis_rename_defs_type_def(context: &mut Context, type_def: &mut SynTypeDef) -> Result<(), ()> {
    // name
    {
        let unique_id = TableId::new();
        context
            .ident_unique_id_table
            .insert(type_def.name.table_id(), unique_id);
    }
    Ok(())
}

fn felis_rename_defs_fn_def(context: &mut Context, fn_def: &mut SynFnDef) -> Result<(), ()> {
    // name
    {
        let unique_id = TableId::new();
        context
            .ident_unique_id_table
            .insert(fn_def.name.table_id(), unique_id);
    }
    Ok(())
}

fn felis_rename_defs_theorem_def(
    context: &mut Context,
    theorem_def: &mut SynTheoremDef,
) -> Result<(), ()> {
    // name
    {
        let unique_id = TableId::new();
        context
            .ident_unique_id_table
            .insert(theorem_def.name.table_id(), unique_id);
    }
    Ok(())
}

fn felis_rename_uses_file(context: &mut Context, file: &mut SynFile) -> Result<(), ()> {
    Ok(())
}

#[cfg(test)]
mod test {
    use felis_syn::test_utils::parse_from_str;

    use super::*;

    #[test]
    fn felis_rename_defs_type_def_test_1() {
        let mut context = Context::new();
        let s = "#type A : Prop { hoge : A, }";
        let mut type_def = parse_from_str::<SynTypeDef>(&s).unwrap().unwrap();
        felis_rename_defs_type_def(&mut context, &mut type_def).unwrap();
        let _id = *context
            .ident_unique_id_table
            .get(type_def.name.table_id())
            .unwrap();
    }

    #[test]
    fn felis_rename_defs_file_test_1() {
        let mut context = Context::new();
        let s = "#type A : Prop { hoge : A, }";
        let mut file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        felis_rename_defs_file(&mut context, &mut file).unwrap();
        if let SynFileItem::TypeDef(type_def) = &file.items[0] {
            let _id = *context
                .ident_unique_id_table
                .get(type_def.name.table_id())
                .unwrap();
        } else {
            panic!();
        }
    }

    #[test]
    fn felis_rename_defs_file_test_2() {
        let mut context = Context::new();
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let mut file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        felis_rename_defs_file(&mut context, &mut file).unwrap();
        if let SynFileItem::FnDef(fn_def) = &file.items[0] {
            let _id = *context
                .ident_unique_id_table
                .get(fn_def.name.table_id())
                .unwrap();
        } else {
            panic!();
        }
    }

    #[test]
    fn felis_rename_defs_file_test_3() {
        let mut context = Context::new();
        let s = std::fs::read_to_string("../../library/wip/prop2.fe").unwrap();
        let mut file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        felis_rename_defs_file(&mut context, &mut file).unwrap();
        if let SynFileItem::TypeDef(type_def) = &file.items[0] {
            let _id = *context
                .ident_unique_id_table
                .get(type_def.name.table_id())
                .unwrap();
        } else {
            panic!();
        }
        if let SynFileItem::TypeDef(type_def) = &file.items[1] {
            let _id = *context
                .ident_unique_id_table
                .get(type_def.name.table_id())
                .unwrap();
        } else {
            panic!();
        }
        if let SynFileItem::TheoremDef(theorem_def) = &file.items[2] {
            let _id = *context
                .ident_unique_id_table
                .get(theorem_def.name.table_id())
                .unwrap();
        } else {
            panic!();
        }
    }
}
