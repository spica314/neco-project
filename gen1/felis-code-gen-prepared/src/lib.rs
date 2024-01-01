use std::marker::PhantomData;

use felis_rename::rename_defs::DefId;
use felis_syn::{
    decoration::Decoration,
    syn_entrypoint::SynEntrypoint,
    syn_expr::{
        SynExpr, SynExprApp, SynExprIdentWithPath, SynExprMatch, SynExprMatchArm,
        SynExprMatchPattern, SynExprNumber, SynExprParen, SynExprString,
    },
    syn_file::{SynFile, SynFileItem},
    syn_proc::{SynProcBlock, SynProcDef},
    syn_statement::{
        syn_statement_assign::SynStatementAssign, syn_statement_break::SynStatementBreak,
        syn_statement_continue::SynStatementContinue,
        syn_statement_expr_semi::SynStatementExprSemi, syn_statement_if::SynStatementIf,
        syn_statement_loop::SynStatementLoop, SynStatement, SynStatementLet,
        SynStatementLetInitial,
    },
    syn_type::{
        SynType, SynTypeApp, SynTypeAtom, SynTypeDependentMap, SynTypeMap, SynTypeParen,
        SynTypeUnit,
    },
    syn_type_def::{SynTypeDef, SynVariant},
    syn_typed_arg::SynTypedArg,
};
use felis_type_checker::typing::TypedDecoration;
use neco_type_checker::type_term::TypeTerm;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecoration;

impl Decoration for CodeGenPreparedDecoration {
    type Entrypoint = CodeGenPreparedDecorationEntrypoint;
    type ExprApp = CodeGenPreparedDecorationExprApp;
    type ExprIdentWithPath = CodeGenPreparedDecorationExprIdentWithPath;
    type ExprNumber = CodeGenPreparedDecorationExprNumber;
    type ExprMatch = ();
    type ExprParen = ();
    type ExprString = CodeGenPreparedDecorationExprString;
    type Variant = CodeGenPreparedDecorationVariant;
    type TypeDef = CodeGenPreparedDecorationTypeDef;
    type TypeApp = ();
    type TypeAtom = CodeGenPreparedDecorationTypeAtom;
    type TypeMap = ();
    type TypeParen = ();
    type TypeDependentMap = ();
    type TypeUnit = ();
    type File = CodeGenPreparedDecorationFile;
    type FnDef = CodeGenPreparedDecorationFnDef;
    type ProcDef = CodeGenPreparedDecorationProcDef;
    type TypedArg = CodeGenPreparedDecorationTypedArg;
    type ExprMatchPattern = CodeGenPreparedDecorationExprMatchPattern;
    type StatementLet = CodeGenPreparedDecorationStatementLet;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationFile {
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationTypeDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationFnDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationProcDef {
    pub name: String,
    pub id: DefId,
    pub lets: Vec<(DefId, TypeTerm)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationEntrypoint {
    pub use_id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationStatementLet {
    pub name: String,
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationExprIdentWithPath {
    pub path_ids: Vec<DefId>,
    pub use_id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationTypeAtom {
    pub use_id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationVariant {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationTypedArg {
    pub name: String,
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationExprMatchPattern {
    pub ids: Vec<(String, DefId)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationExprNumber {
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationExprString {
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeGenPreparedDecorationExprApp {
    pub id: DefId,
    pub ty: TypeTerm,
}

pub fn prepare_code_gen_file(
    file: &SynFile<TypedDecoration>,
) -> SynFile<CodeGenPreparedDecoration> {
    let mut items = vec![];
    for item in &file.items {
        let item2 = prepare_code_gen_file_item(item);
        items.push(item2);
    }
    SynFile {
        items,
        ext: CodeGenPreparedDecorationFile { id: file.ext.id },
    }
}

fn prepare_code_gen_file_item(
    item: &SynFileItem<TypedDecoration>,
) -> SynFileItem<CodeGenPreparedDecoration> {
    match item {
        SynFileItem::TypeDef(type_def) => {
            let type_def = prepare_code_gen_type_def(type_def);
            SynFileItem::TypeDef(type_def)
        }
        SynFileItem::Entrypoint(entrypoint) => {
            let entrypoint = prepare_code_gen_entrypoint(entrypoint);
            SynFileItem::Entrypoint(entrypoint)
        }
        SynFileItem::ProcDef(proc_def) => {
            let proc_def = prepare_code_gen_proc_def(proc_def);
            SynFileItem::ProcDef(proc_def)
        }
    }
}

fn prepare_code_gen_type_def(
    type_def: &SynTypeDef<TypedDecoration>,
) -> SynTypeDef<CodeGenPreparedDecoration> {
    let ty_ty = prepare_code_gen_type(type_def.ty_ty.as_ref());
    let mut variants = vec![];
    for variant in &type_def.variants {
        let variant = prepare_code_gen_variant(variant);
        variants.push(variant);
    }
    SynTypeDef {
        keyword_type: type_def.keyword_type.clone(),
        name: type_def.name.clone(),
        colon: type_def.colon.clone(),
        ty_ty: Box::new(ty_ty),
        lbrace: type_def.lbrace.clone(),
        variants,
        rbrace: type_def.rbrace.clone(),
        ext: CodeGenPreparedDecorationTypeDef {
            name: type_def.ext.name.clone(),
            id: type_def.ext.id,
        },
    }
}

fn prepare_code_gen_entrypoint(
    entrypoint: &SynEntrypoint<TypedDecoration>,
) -> SynEntrypoint<CodeGenPreparedDecoration> {
    SynEntrypoint {
        token_entrypoint: entrypoint.token_entrypoint.clone(),
        ident: entrypoint.ident.clone(),
        ext: CodeGenPreparedDecorationEntrypoint {
            use_id: entrypoint.ext.use_id,
        },
    }
}

fn prepare_code_gen_proc_def(
    proc_def: &SynProcDef<TypedDecoration>,
) -> SynProcDef<CodeGenPreparedDecoration> {
    let ty = prepare_code_gen_type(&proc_def.ty);
    let (proc_block, lets) = prepare_code_gen_proc_block(&proc_def.proc_block);
    SynProcDef {
        keyword_proc: proc_def.keyword_proc.clone(),
        name: proc_def.name.clone(),
        colon: proc_def.colon.clone(),
        ty,
        proc_block,
        ext: CodeGenPreparedDecorationProcDef {
            name: proc_def.ext.name.clone(),
            id: proc_def.ext.id,
            lets,
        },
    }
}

fn prepare_code_gen_proc_block(
    proc_block: &SynProcBlock<TypedDecoration>,
) -> (
    SynProcBlock<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut statements = vec![];
    let mut lets = vec![];
    for statement in &proc_block.statements {
        let (statement, lets2) = prepare_code_gen_statement(statement);
        statements.push(statement);
        lets.extend(lets2);
    }
    (
        SynProcBlock {
            lbrace: proc_block.lbrace.clone(),
            statements,
            rbrace: proc_block.rbrace.clone(),
        },
        lets,
    )
}

fn prepare_code_gen_statement(
    statement: &SynStatement<TypedDecoration>,
) -> (
    SynStatement<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    match statement {
        SynStatement::ExprSemi(expr_semi) => {
            let (expr, lets) = prepare_code_gen_statement_expr_semi(expr_semi);
            (SynStatement::ExprSemi(expr), lets)
        }
        SynStatement::Let(statement_let) => {
            let (statement_let, lets) = prepare_code_gen_statement_let(statement_let);
            (SynStatement::Let(statement_let), lets)
        }
        SynStatement::Assign(statement_assign) => {
            let (statement_assign, lets) = prepare_code_gen_statement_assign(statement_assign);
            (SynStatement::Assign(statement_assign), lets)
        }
        SynStatement::If(statement_if) => {
            let (statement_if, lets) = prepare_code_gen_statement_if(statement_if);
            (SynStatement::If(statement_if), lets)
        }
        SynStatement::Loop(statement_loop) => {
            let (statement_loop, lets) = prepare_code_gen_statement_loop(statement_loop);
            (SynStatement::Loop(statement_loop), lets)
        }
        SynStatement::Break(statement_break) => {
            let lets = vec![];
            (
                SynStatement::Break(SynStatementBreak {
                    keyword_break: statement_break.keyword_break.clone(),
                    semi: statement_break.semi.clone(),
                    ext: PhantomData,
                }),
                lets,
            )
        }
        SynStatement::Continue(statement_continue) => {
            let lets = vec![];
            (
                SynStatement::Continue(SynStatementContinue {
                    keyword_continue: statement_continue.keyword_continue.clone(),
                    semi: statement_continue.semi.clone(),
                    ext: PhantomData,
                }),
                lets,
            )
        }
    }
}

fn prepare_code_gen_statement_assign(
    statement_assign: &SynStatementAssign<TypedDecoration>,
) -> (
    SynStatementAssign<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let (lhs, mut lets) = prepare_code_gen_expr(&statement_assign.lhs);
    let (rhs, lets2) = prepare_code_gen_expr(&statement_assign.rhs);
    lets.extend(lets2);
    (
        SynStatementAssign {
            lhs,
            eq: statement_assign.eq.clone(),
            rhs,
            semi: statement_assign.semi.clone(),
        },
        lets,
    )
}

fn prepare_code_gen_statement_if(
    statement_if: &SynStatementIf<TypedDecoration>,
) -> (
    SynStatementIf<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut lets = vec![];
    let (cond, lets2) = prepare_code_gen_expr(&statement_if.cond);
    lets.extend(lets2);
    let (t_branch, lets2) = prepare_code_gen_proc_block(&statement_if.t_branch);
    lets.extend(lets2);
    let (f_branch, lets2) = prepare_code_gen_proc_block(&statement_if.f_branch);
    lets.extend(lets2);
    (
        SynStatementIf {
            keyword_if: statement_if.keyword_if.clone(),
            cond,
            t_branch,
            keyword_else: statement_if.keyword_else.clone(),
            f_branch,
        },
        lets,
    )
}

fn prepare_code_gen_statement_loop(
    statement_loop: &SynStatementLoop<TypedDecoration>,
) -> (
    SynStatementLoop<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let (block, lets) = prepare_code_gen_proc_block(&statement_loop.block);
    (
        SynStatementLoop {
            keyword_loop: statement_loop.keyword_loop.clone(),
            block,
        },
        lets,
    )
}

fn prepare_code_gen_statement_let(
    statement_let: &SynStatementLet<TypedDecoration>,
) -> (
    SynStatementLet<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    if let Some(initial) = &statement_let.initial {
        let (expr, mut lets) = prepare_code_gen_expr(&initial.expr);
        lets.push((statement_let.ext.id, statement_let.ext.ty.clone()));
        (
            SynStatementLet {
                keyword_let: statement_let.keyword_let.clone(),
                keyword_mut: statement_let.keyword_mut.clone(),
                name: statement_let.name.clone(),
                initial: Some(SynStatementLetInitial {
                    eq: initial.eq.clone(),
                    expr,
                }),
                semi: statement_let.semi.clone(),
                ext: CodeGenPreparedDecorationStatementLet {
                    name: statement_let.ext.name.clone(),
                    id: statement_let.ext.id,
                    ty: statement_let.ext.ty.clone(),
                },
            },
            lets,
        )
    } else {
        let lets = vec![(statement_let.ext.id, statement_let.ext.ty.clone())];
        (
            SynStatementLet {
                keyword_let: statement_let.keyword_let.clone(),
                keyword_mut: statement_let.keyword_mut.clone(),
                name: statement_let.name.clone(),
                initial: None,
                semi: statement_let.semi.clone(),
                ext: CodeGenPreparedDecorationStatementLet {
                    name: statement_let.ext.name.clone(),
                    id: statement_let.ext.id,
                    ty: statement_let.ext.ty.clone(),
                },
            },
            lets,
        )
    }
}

fn prepare_code_gen_expr(
    expr: &SynExpr<TypedDecoration>,
) -> (SynExpr<CodeGenPreparedDecoration>, Vec<(DefId, TypeTerm)>) {
    match expr {
        SynExpr::IdentWithPath(expr_ident_with_path) => {
            let (expr_ident_with_path, lets) =
                prepare_code_gen_expr_ident_with_path(expr_ident_with_path);
            (SynExpr::IdentWithPath(expr_ident_with_path), lets)
        }
        SynExpr::String(expr_string) => {
            let (expr_string, lets) = prepare_code_gen_expr_string(expr_string);
            (SynExpr::String(expr_string), lets)
        }
        SynExpr::App(expr_app) => {
            let (expr_app, lets) = prepare_code_gen_expr_app(expr_app);
            (SynExpr::App(expr_app), lets)
        }
        SynExpr::Paren(expr_paren) => {
            let (expr_paren, lets) = prepare_code_gen_expr_paren(expr_paren);
            (SynExpr::Paren(expr_paren), lets)
        }
        SynExpr::Match(expr_match) => {
            let (expr_match, lets) = prepare_code_gen_expr_match(expr_match);
            (SynExpr::Match(expr_match), lets)
        }
        SynExpr::Number(expr_number) => {
            let (expr_number, lets) = prepare_code_gen_expr_number(expr_number);
            (SynExpr::Number(expr_number), lets)
        }
    }
}

fn prepare_code_gen_expr_ident_with_path(
    expr_ident_with_path: &SynExprIdentWithPath<TypedDecoration>,
) -> (
    SynExprIdentWithPath<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let lets = vec![];
    (
        SynExprIdentWithPath {
            path: expr_ident_with_path.path.clone(),
            ident: expr_ident_with_path.ident.clone(),
            ext: CodeGenPreparedDecorationExprIdentWithPath {
                path_ids: expr_ident_with_path.ext.path_ids.clone(),
                use_id: expr_ident_with_path.ext.use_id,
                ty: expr_ident_with_path.ext.ty.clone(),
            },
        },
        lets,
    )
}

fn prepare_code_gen_expr_string(
    expr_string: &SynExprString<TypedDecoration>,
) -> (
    SynExprString<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let lets = vec![];
    (
        SynExprString {
            token_string: expr_string.token_string.clone(),
            ext: CodeGenPreparedDecorationExprString {
                id: expr_string.ext.id,
                ty: expr_string.ext.ty.clone(),
            },
        },
        lets,
    )
}

fn prepare_code_gen_expr_number(
    expr_number: &SynExprNumber<TypedDecoration>,
) -> (
    SynExprNumber<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let lets = vec![];
    (
        SynExprNumber {
            number: expr_number.number.clone(),
            ext: CodeGenPreparedDecorationExprNumber {
                id: expr_number.ext.id,
                ty: expr_number.ext.ty.clone(),
            },
        },
        lets,
    )
}

fn prepare_code_gen_expr_app(
    expr_app: &SynExprApp<TypedDecoration>,
) -> (
    SynExprApp<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut exprs = vec![];
    let mut lets = vec![];
    for expr in &expr_app.exprs {
        let (expr, lets2) = prepare_code_gen_expr(expr);
        exprs.push(expr);
        lets.extend(lets2);
    }

    let ext = CodeGenPreparedDecorationExprApp {
        id: expr_app.ext.id,
        ty: expr_app.ext.ty.clone(),
    };

    (SynExprApp { exprs, ext }, lets)
}

fn prepare_code_gen_expr_paren(
    expr_paren: &SynExprParen<TypedDecoration>,
) -> (
    SynExprParen<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut lets = vec![];
    let expr = prepare_code_gen_expr(expr_paren.expr.as_ref());
    lets.extend(expr.1);
    (
        SynExprParen {
            lparen: expr_paren.lparen.clone(),
            expr: Box::new(expr.0),
            rparen: expr_paren.rparen.clone(),
            ext: (),
        },
        lets,
    )
}

fn prepare_code_gen_expr_match(
    expr_match: &SynExprMatch<TypedDecoration>,
) -> (
    SynExprMatch<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut lets = vec![];
    let expr = prepare_code_gen_expr(expr_match.expr.as_ref());
    lets.extend(expr.1);
    let mut arms = vec![];
    for arm in &expr_match.arms {
        let (arm, lets2) = prepare_code_gen_expr_match_arm(arm);
        arms.push(arm);
        lets.extend(lets2);
    }
    (
        SynExprMatch {
            keyword_match: expr_match.keyword_match.clone(),
            expr: Box::new(expr.0),
            lbrace: expr_match.lbrace.clone(),
            arms,
            rbrace: expr_match.rbrace.clone(),
            ext: (),
        },
        lets,
    )
}

fn prepare_code_gen_expr_match_arm(
    arm: &SynExprMatchArm<TypedDecoration>,
) -> (
    SynExprMatchArm<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut lets = vec![];
    let (pattern, lets2) = prepare_code_gen_expr_match_pattern(&arm.pattern);
    lets.extend(lets2);
    let (expr, lets2) = prepare_code_gen_expr(&arm.expr);
    lets.extend(lets2);
    (
        SynExprMatchArm {
            pattern,
            arrow2: arm.arrow2.clone(),
            expr,
            camma: arm.camma.clone(),
        },
        lets,
    )
}

fn prepare_code_gen_expr_match_pattern(
    pattern: &SynExprMatchPattern<TypedDecoration>,
) -> (
    SynExprMatchPattern<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut lets = vec![];
    let (type_constructor, lets2) =
        prepare_code_gen_expr_ident_with_path(&pattern.type_constructor);
    lets.extend(lets2);
    (
        SynExprMatchPattern {
            type_constructor,
            idents: pattern.idents.clone(),
            ext: CodeGenPreparedDecorationExprMatchPattern {
                ids: pattern.ext.ids.clone(),
            },
        },
        lets,
    )
}

fn prepare_code_gen_type(ty: &SynType<TypedDecoration>) -> SynType<CodeGenPreparedDecoration> {
    match ty {
        SynType::App(type_app) => {
            let type_app = prepare_code_gen_type_app(type_app);
            SynType::App(type_app)
        }
        SynType::Atom(type_atom) => {
            let type_atom = prepare_code_gen_type_atom(type_atom);
            SynType::Atom(type_atom)
        }
        SynType::Map(type_map) => {
            let type_map = prepare_code_gen_type_map(type_map);
            SynType::Map(type_map)
        }
        SynType::Paren(type_paren) => {
            let type_paren = prepare_code_gen_type_paren(type_paren);
            SynType::Paren(type_paren)
        }
        SynType::DependentMap(type_dependent_map) => {
            let type_dependent_map = prepare_code_gen_type_dependent_map(type_dependent_map);
            SynType::DependentMap(type_dependent_map)
        }
        SynType::Unit(type_unit) => {
            let type_unit = prepare_code_gen_type_unit(type_unit);
            SynType::Unit(type_unit)
        }
    }
}

fn prepare_code_gen_type_app(
    type_app: &SynTypeApp<TypedDecoration>,
) -> SynTypeApp<CodeGenPreparedDecoration> {
    let left = prepare_code_gen_type(type_app.left.as_ref());
    let right = prepare_code_gen_type(type_app.right.as_ref());
    SynTypeApp {
        left: Box::new(left),
        right: Box::new(right),
        ext: (),
    }
}

fn prepare_code_gen_type_atom(
    type_atom: &SynTypeAtom<TypedDecoration>,
) -> SynTypeAtom<CodeGenPreparedDecoration> {
    SynTypeAtom {
        ident: type_atom.ident.clone(),
        ext: CodeGenPreparedDecorationTypeAtom {
            use_id: type_atom.ext.use_id,
            ty: type_atom.ext.ty.clone(),
        },
    }
}

fn prepare_code_gen_type_map(
    type_map: &SynTypeMap<TypedDecoration>,
) -> SynTypeMap<CodeGenPreparedDecoration> {
    let from = prepare_code_gen_type(type_map.from.as_ref());
    let to = prepare_code_gen_type(type_map.to.as_ref());
    SynTypeMap {
        from: Box::new(from),
        arrow: type_map.arrow.clone(),
        to: Box::new(to),
        ext: (),
    }
}

fn prepare_code_gen_type_paren(
    type_paren: &SynTypeParen<TypedDecoration>,
) -> SynTypeParen<CodeGenPreparedDecoration> {
    let ty = prepare_code_gen_type(type_paren.ty.as_ref());
    SynTypeParen {
        lparen: type_paren.lparen.clone(),
        ty: Box::new(ty),
        rparen: type_paren.rparen.clone(),
        ext: (),
    }
}

fn prepare_code_gen_type_dependent_map(
    type_dependent_map: &SynTypeDependentMap<TypedDecoration>,
) -> SynTypeDependentMap<CodeGenPreparedDecoration> {
    let from = prepare_code_gen_typed_arg(type_dependent_map.from.as_ref());
    let to = prepare_code_gen_type(type_dependent_map.to.as_ref());
    SynTypeDependentMap {
        from: Box::new(from),
        arrow: type_dependent_map.arrow.clone(),
        to: Box::new(to),
        ext: (),
    }
}

fn prepare_code_gen_type_unit(
    type_unit: &SynTypeUnit<TypedDecoration>,
) -> SynTypeUnit<CodeGenPreparedDecoration> {
    SynTypeUnit {
        lparen: type_unit.lparen.clone(),
        rparen: type_unit.rparen.clone(),
        ext: (),
    }
}

fn prepare_code_gen_typed_arg(
    typed_arg: &SynTypedArg<TypedDecoration>,
) -> SynTypedArg<CodeGenPreparedDecoration> {
    let ty = prepare_code_gen_type(&typed_arg.ty);
    SynTypedArg {
        lparen: typed_arg.lparen.clone(),
        name: typed_arg.name.clone(),
        colon: typed_arg.colon.clone(),
        ty,
        rparen: typed_arg.rparen.clone(),
        ext: CodeGenPreparedDecorationTypedArg {
            name: typed_arg.ext.name.clone(),
            id: typed_arg.ext.id,
            ty: typed_arg.ext.ty.clone(),
        },
    }
}

fn prepare_code_gen_variant(
    variant: &SynVariant<TypedDecoration>,
) -> SynVariant<CodeGenPreparedDecoration> {
    let ty = prepare_code_gen_type(&variant.ty);
    SynVariant {
        name: variant.name.clone(),
        colon: variant.colon.clone(),
        ty,
        camma: variant.camma.clone(),
        ext: CodeGenPreparedDecorationVariant {
            name: variant.ext.name.clone(),
            id: variant.ext.id,
        },
    }
}

pub fn prepare_code_gen_statement_expr_semi(
    expr_semi: &SynStatementExprSemi<TypedDecoration>,
) -> (
    SynStatementExprSemi<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let (expr, lets) = prepare_code_gen_expr(&expr_semi.expr);
    (
        SynStatementExprSemi {
            expr,
            semi: expr_semi.semi.clone(),
        },
        lets,
    )
}
