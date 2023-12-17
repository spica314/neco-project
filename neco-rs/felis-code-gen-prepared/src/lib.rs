use felis_rename::rename_defs::DefId;
use felis_syn::{
    decoration::Decoration,
    syn_entrypoint::SynEntrypoint,
    syn_expr::{
        SynExpr, SynExprApp, SynExprBlock, SynExprIdentWithPath, SynExprMatch, SynExprMatchArm,
        SynExprMatchPattern, SynExprParen, SynExprString,
    },
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::{SynFnBlock, SynFnDef},
    syn_proc::{SynProcBlock, SynProcDef},
    syn_statement::{syn_statement_expr_semi::SynStatementExprSemi, SynStatement, SynStatementLet},
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
    type ExprApp = ();
    type ExprBlock = ();
    type ExprIdentWithPath = CodeGenPreparedDecorationExprIdentWithPath;
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
pub struct CodeGenPreparedDecorationExprString {
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
        SynFileItem::FnDef(fn_def) => {
            let fn_def = prepare_code_gen_fn_def(fn_def);
            SynFileItem::FnDef(fn_def)
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

fn prepare_code_gen_fn_def(
    fn_def: &SynFnDef<TypedDecoration>,
) -> SynFnDef<CodeGenPreparedDecoration> {
    let ty = prepare_code_gen_type(&fn_def.ty);
    let fn_block = prepare_code_gen_fn_block(&fn_def.fn_block);
    SynFnDef {
        keyword_fn: fn_def.keyword_fn.clone(),
        name: fn_def.name.clone(),
        colon: fn_def.colon.clone(),
        ty,
        fn_block,
        ext: CodeGenPreparedDecorationFnDef {
            name: fn_def.ext.name.clone(),
            id: fn_def.ext.id,
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

fn prepare_code_gen_fn_block(
    fn_block: &SynFnBlock<TypedDecoration>,
) -> SynFnBlock<CodeGenPreparedDecoration> {
    let mut statements = vec![];
    for statement in &fn_block.statements {
        let (statement, _lets) = prepare_code_gen_statement(statement);
        statements.push(statement);
    }
    SynFnBlock {
        lbrace: fn_block.lbrace.clone(),
        statements,
        rbrace: fn_block.rbrace.clone(),
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
        SynStatement::Expr(expr) => {
            let (expr, lets) = prepare_code_gen_expr(expr);
            (SynStatement::Expr(expr), lets)
        }
        SynStatement::ExprSemi(expr_semi) => {
            let (expr, lets) = prepare_code_gen_statement_expr_semi(expr_semi);
            (SynStatement::ExprSemi(expr), lets)
        }
        SynStatement::Let(statement_let) => {
            let (statement_let, lets) = prepare_code_gen_statement_let(statement_let);
            (SynStatement::Let(statement_let), lets)
        }
    }
}

fn prepare_code_gen_statement_let(
    statement_let: &SynStatementLet<TypedDecoration>,
) -> (
    SynStatementLet<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let (expr, mut lets) = prepare_code_gen_expr(&statement_let.expr);
    lets.push((statement_let.ext.id, statement_let.ext.ty.clone()));
    (
        SynStatementLet {
            keyword_let: statement_let.keyword_let.clone(),
            keyword_mut: statement_let.keyword_mut.clone(),
            name: statement_let.name.clone(),
            eq: statement_let.eq.clone(),
            expr,
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
        SynExpr::Block(expr_block) => {
            let (expr_block, lets) = prepare_code_gen_expr_block(expr_block);
            (SynExpr::Block(expr_block), lets)
        }
        SynExpr::Match(expr_match) => {
            let (expr_match, lets) = prepare_code_gen_expr_match(expr_match);
            (SynExpr::Match(expr_match), lets)
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
    (SynExprApp { exprs, ext: () }, lets)
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

fn prepare_code_gen_expr_block(
    expr_block: &SynExprBlock<TypedDecoration>,
) -> (
    SynExprBlock<CodeGenPreparedDecoration>,
    Vec<(DefId, TypeTerm)>,
) {
    let mut lets = vec![];
    let mut statements = vec![];
    for statement in &expr_block.statements {
        let (statement, lets2) = prepare_code_gen_statement(statement);
        statements.push(statement);
        lets.extend(lets2);
    }
    (
        SynExprBlock {
            lbrace: expr_block.lbrace.clone(),
            statements,
            rbrace: expr_block.rbrace.clone(),
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

    use felis_type_checker::{
        retrieve::{retrieve_file, setup_retrieve_context},
        typing::typing_file,
    };

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
        assert_eq!(var_types.len(), 4);
        let file_4 = typing_file(&mut context, &file_3);

        let file_5 = prepare_code_gen_file(&file_4);
        eprintln!();
        for item in &file_5.items {
            if let SynFileItem::ProcDef(proc_def) = item {
                for (id, ty) in &proc_def.ext.lets {
                    eprintln!("{:?}: {:?}", id, ty);
                }
                assert_eq!(proc_def.ext.lets.len(), 1);
            }
        }
    }
}