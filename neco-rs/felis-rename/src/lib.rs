use std::collections::HashMap;

use felis_package::FelisPackage;
use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_theorem_def::SynTheoremDef,
    syn_type::SynType,
    syn_type_def::SynTypeDef,
    token_id::TokenId,
};
use neco_resolver::Resolver;

struct Context<'a> {
    dependencies: &'a [&'a FelisPackage],
    type_resolver: Resolver<SynType>,
    ident_unique_id_table: HashMap<TokenId, usize>,
    next_ident_unique_id: usize,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            dependencies: &[],
            type_resolver: Resolver::new(),
            ident_unique_id_table: HashMap::new(),
            next_ident_unique_id: 1,
        }
    }
    pub fn make_ident_unique_id(&mut self, s: &str) -> usize {
        let res = self.next_ident_unique_id;
        self.next_ident_unique_id += 1;
        res
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
        let unique_id = context.make_ident_unique_id(type_def.name.ident.as_str());
        context
            .ident_unique_id_table
            .insert(type_def.name.token_id(), unique_id);
    }
    Ok(())
}

fn felis_rename_defs_fn_def(context: &mut Context, fn_def: &mut SynFnDef) -> Result<(), ()> {
    // name
    {
        let unique_id = context.make_ident_unique_id(fn_def.name.ident.as_str());
        context
            .ident_unique_id_table
            .insert(fn_def.name.token_id(), unique_id);
    }
    Ok(())
}

fn felis_rename_defs_theorem_def(
    context: &mut Context,
    theorem_def: &mut SynTheoremDef,
) -> Result<(), ()> {
    // name
    {
        let unique_id = context.make_ident_unique_id(theorem_def.name.ident.as_str());
        context
            .ident_unique_id_table
            .insert(theorem_def.name.token_id(), unique_id);
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
        let id = *context
            .ident_unique_id_table
            .get(&type_def.name.token_id())
            .unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn felis_rename_defs_file_test_1() {
        let mut context = Context::new();
        let s = "#type A : Prop { hoge : A, }";
        let mut file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        felis_rename_defs_file(&mut context, &mut file).unwrap();
        if let SynFileItem::TypeDef(type_def) = &file.items[0] {
            let id = *context
                .ident_unique_id_table
                .get(&type_def.name.token_id())
                .unwrap();
            assert_eq!(id, 1);
        }
    }

    #[test]
    fn felis_rename_defs_file_test_2() {
        let mut context = Context::new();
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let mut file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        felis_rename_defs_file(&mut context, &mut file).unwrap();
        if let SynFileItem::FnDef(fn_def) = &file.items[0] {
            let id = *context
                .ident_unique_id_table
                .get(&fn_def.name.token_id())
                .unwrap();
            assert_eq!(id, 1);
        }
    }

    #[test]
    fn felis_rename_defs_file_test_3() {
        let mut context = Context::new();
        let s = std::fs::read_to_string("../../library/wip/prop2.fe").unwrap();
        let mut file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        felis_rename_defs_file(&mut context, &mut file).unwrap();
        if let SynFileItem::TypeDef(type_def) = &file.items[0] {
            let id = *context
                .ident_unique_id_table
                .get(&type_def.name.token_id())
                .unwrap();
            assert_eq!(id, 1);
        }
        if let SynFileItem::TypeDef(type_def) = &file.items[1] {
            let id = *context
                .ident_unique_id_table
                .get(&type_def.name.token_id())
                .unwrap();
            assert_eq!(id, 2);
        }
        if let SynFileItem::TheoremDef(theorem_def) = &file.items[2] {
            let id = *context
                .ident_unique_id_table
                .get(&theorem_def.name.token_id())
                .unwrap();
            assert_eq!(id, 3);
        }
    }
}
