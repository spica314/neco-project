use std::collections::HashMap;

use felis_rename::{rename_defs::DefId, rename_uses::RenameDecoration};
use felis_syn::{
    syn_expr::SynExpr,
    syn_file::{SynFile, SynFileItem},
    syn_proc::{SynProcBlock, SynProcDef},
    syn_statement::{
        syn_statement_assign::SynStatementAssign, syn_statement_if::SynStatementIf,
        syn_statement_loop::SynStatementLoop, SynStatement, SynStatementLet,
    },
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

    assert_eq!(prelude_map.len(), 4);

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

    // __eq_i64
    let var_id = type_checker.add_var(TypeTerm::Arrow(
        Box::new(TypeTerm::Base(ty_i64)),
        Box::new(TypeTerm::Arrow(
            Box::new(TypeTerm::Base(ty_i64)),
            Box::new(TypeTerm::Base(ty_i64)),
        )),
    ));
    let def_id = prelude_map["__eq_i64"];
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

fn retrieve_proc_block(context: &mut RetrieveContext, proc_block: &SynProcBlock<RenameDecoration>) {
    for statement in &proc_block.statements {
        retrieve_statement(context, statement);
    }
}

fn retrieve_proc_def(context: &mut RetrieveContext, proc_def: &SynProcDef<RenameDecoration>) {
    retrieve_proc_block(context, &proc_def.proc_block);
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
        SynStatement::If(statement_if) => {
            retrieve_statement_if(context, statement_if);
        }
        SynStatement::Loop(statement_loop) => {
            retrieve_statement_loop(context, statement_loop);
        }
        SynStatement::Break(_statement_break) => {}
        SynStatement::Continue(_statement_continue) => {}
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

fn retrieve_statement_if(
    context: &mut RetrieveContext,
    statement_if: &SynStatementIf<RenameDecoration>,
) {
    retrieve_expr(context, &statement_if.cond);
    retrieve_proc_block(context, &statement_if.t_branch);
    retrieve_proc_block(context, &statement_if.f_branch);
}

fn retrieve_statement_loop(
    context: &mut RetrieveContext,
    statement_loop: &SynStatementLoop<RenameDecoration>,
) {
    retrieve_proc_block(context, &statement_loop.block);
}

fn retrieve_statement_let(
    context: &mut RetrieveContext,
    statement_let: &SynStatementLet<RenameDecoration>,
) {
    let var_id = *context
        .def_to_var
        .entry(statement_let.ext.id)
        .or_insert_with(|| context.type_checker.add_var(TypeTerm::Unknown));
    if let Some(initial) = &statement_let.initial {
        let ty = retrieve_expr(context, &initial.expr);
        context.type_checker.add_relation(TypeTerm::Var(var_id), ty);
    }
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
            let var_id = *context
                .def_to_var
                .entry(app.ext.id)
                .or_insert_with(|| context.type_checker.add_var(TypeTerm::Unknown));

            let mut ty = retrieve_expr(context, &app.exprs[0]);
            for expr in &app.exprs[1..] {
                let s = retrieve_expr(context, expr);
                ty = TypeTerm::App(Box::new(ty), Box::new(s));
            }

            context
                .type_checker
                .add_relation(TypeTerm::Var(var_id), ty.clone());
            ty
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
