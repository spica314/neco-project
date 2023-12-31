use std::collections::HashMap;

use felis_rename::{rename_defs::DefId, rename_uses::RenameDecoration};
use felis_syn::{
    syn_expr::SynExpr,
    syn_file::{SynFile, SynFileItem},
    syn_proc::SynProcDef,
    syn_statement::{syn_statement_assign::SynStatementAssign, SynStatement, SynStatementLet},
};
use neco_type_checker::{
    type_checker::TypeChecker,
    type_term::{TypeLevel, TypeTerm},
    TypeId, VarId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveContext {
    pub type_checker: TypeChecker,
    pub def_to_var: HashMap<DefId, VarId>,
    pub ty_str: TypeId,
    pub ty_unit: TypeId,
    pub ty_i64: TypeId,
}

pub fn setup_retrieve_context(prelude_map: &HashMap<String, DefId>) -> RetrieveContext {
    let mut type_checker = TypeChecker::new();
    let ty_str = type_checker.add_type(TypeTerm::Star(TypeLevel::new(2)));
    let ty_unit = type_checker.add_type(TypeTerm::Star(TypeLevel::new(2)));
    let ty_i64 = type_checker.add_type(TypeTerm::Star(TypeLevel::new(2)));

    let mut def_to_var = HashMap::new();

    assert_eq!(prelude_map.len(), 3);

    // __write_to_stdout
    let var_id = type_checker.add_var(TypeTerm::Arrow(
        Box::new(TypeTerm::Base(ty_str)),
        Box::new(TypeTerm::Base(ty_unit)),
    ));
    let def_id = prelude_map["__write_to_stdout"];
    def_to_var.insert(def_id, var_id);

    // __exit
    let var_id = type_checker.add_var(TypeTerm::Arrow(
        Box::new(TypeTerm::Base(ty_i64)),
        Box::new(TypeTerm::Base(ty_unit)),
    ));
    let def_id = prelude_map["__exit"];
    def_to_var.insert(def_id, var_id);

    // __add_i64
    let var_id = type_checker.add_var(TypeTerm::Arrow(
        Box::new(TypeTerm::Base(ty_i64)),
        Box::new(TypeTerm::Arrow(
            Box::new(TypeTerm::Base(ty_i64)),
            Box::new(TypeTerm::Base(ty_i64)),
        )),
    ));
    let def_id = prelude_map["__add_i64"];
    def_to_var.insert(def_id, var_id);

    RetrieveContext {
        type_checker,
        def_to_var,
        ty_str,
        ty_unit,
        ty_i64,
    }
}

pub fn retrieve_file(context: &mut RetrieveContext, file: &SynFile<RenameDecoration>) {
    for item in &file.items {
        retrieve_file_item(context, item);
    }
}

fn retrieve_file_item(context: &mut RetrieveContext, item: &SynFileItem<RenameDecoration>) {
    match item {
        SynFileItem::TypeDef(_type_def) => {
            todo!()
        }
        SynFileItem::Entrypoint(_entrypoint) => {}
        SynFileItem::ProcDef(proc_def) => {
            retrieve_proc_def(context, proc_def);
        }
    }
}

fn retrieve_proc_def(context: &mut RetrieveContext, proc_def: &SynProcDef<RenameDecoration>) {
    for statement in &proc_def.proc_block.statements {
        retrieve_statement(context, statement);
    }
}

fn retrieve_statement(context: &mut RetrieveContext, statement: &SynStatement<RenameDecoration>) {
    match statement {
        SynStatement::ExprSemi(expr_semi) => {
            let var_id = context.type_checker.add_var(TypeTerm::Unknown);
            let ty = retrieve_expr(context, &expr_semi.expr);
            context.type_checker.add_relation(TypeTerm::Var(var_id), ty);
        }
        SynStatement::Let(statement_let) => {
            retrieve_statement_let(context, statement_let);
        }
        SynStatement::Assign(statement_assign) => {
            retrieve_statement_assign(context, statement_assign);
        }
    }
}

fn retrieve_statement_assign(
    context: &mut RetrieveContext,
    statement_assign: &SynStatementAssign<RenameDecoration>,
) {
    retrieve_expr(context, &statement_assign.lhs);
    retrieve_expr(context, &statement_assign.rhs);
    // todo: add relation
}

fn retrieve_statement_let(
    context: &mut RetrieveContext,
    statement_let: &SynStatementLet<RenameDecoration>,
) {
    let var_id = *context
        .def_to_var
        .entry(statement_let.ext.id)
        .or_insert_with(|| context.type_checker.add_var(TypeTerm::Unknown));
    let ty = retrieve_expr(context, &statement_let.expr);
    context.type_checker.add_relation(TypeTerm::Var(var_id), ty);
}

fn retrieve_expr(context: &mut RetrieveContext, expr: &SynExpr<RenameDecoration>) -> TypeTerm {
    match expr {
        SynExpr::IdentWithPath(ident_with_path) => {
            let def_id = ident_with_path.ext.use_id;
            let var_id = context
                .def_to_var
                .entry(def_id)
                .or_insert_with(|| context.type_checker.add_var(TypeTerm::Unknown));
            TypeTerm::Var(*var_id)
        }
        SynExpr::App(app) => {
            let mut t = retrieve_expr(context, &app.exprs[0]);
            for expr in &app.exprs[1..] {
                let s = retrieve_expr(context, expr);
                t = TypeTerm::App(Box::new(t), Box::new(s));
            }
            t
        }
        SynExpr::Match(_) => todo!(),
        SynExpr::Paren(_) => todo!(),
        SynExpr::String(expr_string) => {
            let var_id = context
                .def_to_var
                .entry(expr_string.ext.id)
                .or_insert_with(|| context.type_checker.add_var(TypeTerm::Unknown));
            let ty = TypeTerm::Base(context.ty_str);
            context
                .type_checker
                .add_relation(TypeTerm::Var(*var_id), ty.clone());
            ty
        }
        SynExpr::Number(expr_number) => {
            let var_id = context
                .def_to_var
                .entry(expr_number.ext.id)
                .or_insert_with(|| context.type_checker.add_var(TypeTerm::Unknown));
            let ty = TypeTerm::Base(context.ty_i64);
            context
                .type_checker
                .add_relation(TypeTerm::Var(*var_id), ty.clone());
            ty
        }
    }
}

#[cfg(test)]
mod test {
    use felis_rename::{
        path_table::construct_path_table_syn_file,
        rename_defs::{rename_defs_file, RenameDefContext},
        rename_uses::{rename_uses_file, RenameUseContext},
        setup_resolver_for_prelude,
    };
    use felis_syn::{decoration::UD, test_utils::parse_from_str};
    use neco_resolver::Resolver;

    use super::*;

    #[test]
    fn test_1() {
        let s = std::fs::read_to_string("../../examples/let-string/main.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        let mut resolver = Resolver::new();
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        path_table.setup_resolver(file_2.ext.id, &mut resolver);
        let prelude_map = setup_resolver_for_prelude(&mut context, &mut resolver);
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        let mut context = setup_retrieve_context(&prelude_map);
        retrieve_file(&mut context, &file_3);
        context.type_checker.resolve();
        let var_types = context.type_checker.get_all();
        for (var_id, ty) in &var_types {
            eprintln!("{:?}: {:?}", var_id, ty);
        }
    }
}
