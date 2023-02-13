use felis_package::FelisPackage;
use felis_syn::{
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::SynFnDef,
    syn_theorem_def::SynTheoremDef,
    syn_type::SynType,
    syn_type_def::SynTypeDef,
};
use neco_resolver::Resolver;

struct Context<'a> {
    dependencies: &'a [&'a FelisPackage],
    type_resolver: Resolver<SynType>,
    rename_index: usize,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            dependencies: &[],
            type_resolver: Resolver::new(),
            rename_index: 1,
        }
    }
    pub fn make_unique_name(&mut self, s: &str) -> String {
        let res = format!("{}_{}", s, self.rename_index);
        self.rename_index += 1;
        res
    }
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn felis_rename(package: &mut FelisPackage, dependencies: &[&FelisPackage]) -> Result<(), ()> {
    let mut context = Context {
        dependencies,
        type_resolver: Resolver::new(),
        rename_index: 1,
    };
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
        let unique_name = context.make_unique_name(type_def.name.ident.as_str());
        type_def.name.unique_name = Some(unique_name);
    }
    Ok(())
}

fn felis_rename_defs_fn_def(context: &mut Context, fn_def: &mut SynFnDef) -> Result<(), ()> {
    Ok(())
}

fn felis_rename_defs_theorem_def(
    context: &mut Context,
    theorem_def: &mut SynTheoremDef,
) -> Result<(), ()> {
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
        assert_eq!(type_def.name.unique_name, Some("A_1".to_string()));
    }

    #[test]
    fn felis_rename_defs_file_test_1() {
        let mut context = Context::new();
        let s = "#type A : Prop { hoge : A, }";
        let mut file = parse_from_str::<SynFile>(&s).unwrap().unwrap();
        felis_rename_defs_file(&mut context, &mut file).unwrap();
        if let SynFileItem::TypeDef(type_def) = &file.items[0] {
            assert_eq!(type_def.name.unique_name, Some("A_1".to_string()));
        }
    }
}
