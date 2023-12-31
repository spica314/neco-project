use felis_rename::{rename_defs::DefId, rename_uses::RenameDecoration};
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
        syn_statement_assign::SynStatementAssign, syn_statement_expr_semi::SynStatementExprSemi,
        SynStatement, SynStatementLet,
    },
    syn_type::{
        SynType, SynTypeApp, SynTypeAtom, SynTypeDependentMap, SynTypeMap, SynTypeParen,
        SynTypeUnit,
    },
    syn_type_def::{SynTypeDef, SynVariant},
    syn_typed_arg::SynTypedArg,
};
use neco_type_checker::type_term::TypeTerm;

use crate::retrieve::RetrieveContext;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecoration;

impl Decoration for TypedDecoration {
    type Entrypoint = TypedDecorationEntrypoint;
    type ExprApp = TypedDecorationExprApp;
    type ExprIdentWithPath = TypedDecorationExprIdentWithPath;
    type ExprNumber = TypedDecorationExprNumber;
    type ExprMatch = ();
    type ExprParen = ();
    type ExprString = TypedDecorationExprString;
    type Variant = TypedDecorationVariant;
    type TypeDef = TypedDecorationTypeDef;
    type TypeApp = ();
    type TypeAtom = TypedDecorationTypeAtom;
    type TypeMap = ();
    type TypeParen = ();
    type TypeDependentMap = ();
    type TypeUnit = ();
    type File = TypedDecorationFile;
    type FnDef = TypedDecorationFnDef;
    type ProcDef = TypedDecorationProcDef;
    type TypedArg = TypedDecorationTypedArg;
    type ExprMatchPattern = TypedDecorationExprMatchPattern;
    type StatementLet = TypedDecorationStatementLet;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationFile {
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationTypeDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationFnDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationProcDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationEntrypoint {
    pub use_id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationStatementLet {
    pub name: String,
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationExprIdentWithPath {
    pub path_ids: Vec<DefId>,
    pub use_id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationTypeAtom {
    pub use_id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationVariant {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationTypedArg {
    pub name: String,
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationExprMatchPattern {
    pub ids: Vec<(String, DefId)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationExprNumber {
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationExprString {
    pub id: DefId,
    pub ty: TypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedDecorationExprApp {
    pub id: DefId,
    pub ty: TypeTerm,
}

pub fn typing_file(
    context: &mut RetrieveContext,
    file: &SynFile<RenameDecoration>,
) -> SynFile<TypedDecoration> {
    let mut items = vec![];
    for item in &file.items {
        let item2 = typing_file_item(context, item);
        items.push(item2);
    }

    SynFile {
        items,
        ext: TypedDecorationFile { id: file.ext.id },
    }
}

pub fn typing_file_item(
    context: &mut RetrieveContext,
    item: &SynFileItem<RenameDecoration>,
) -> SynFileItem<TypedDecoration> {
    match item {
        SynFileItem::TypeDef(type_def) => {
            let type_def2 = typing_type_def(context, type_def);
            SynFileItem::TypeDef(type_def2)
        }
        SynFileItem::Entrypoint(entrypoint) => {
            let entrypoint2 = typing_entrypoint(context, entrypoint);
            SynFileItem::Entrypoint(entrypoint2)
        }
        SynFileItem::ProcDef(proc_def) => {
            let proc_def2 = typing_proc_def(context, proc_def);
            SynFileItem::ProcDef(proc_def2)
        }
    }
}

pub fn typing_type_def(
    context: &mut RetrieveContext,
    type_def: &SynTypeDef<RenameDecoration>,
) -> SynTypeDef<TypedDecoration> {
    // let var_id = context.def_to_var.get(&type_def.ext.id).unwrap().clone();
    // let ty = context.type_checker.get(var_id).unwrap().clone();
    let ty_ty = typing_type(context, &type_def.ty_ty);
    let mut variants = vec![];
    for variant in &type_def.variants {
        let variant2 = typing_variant(context, variant);
        variants.push(variant2);
    }
    SynTypeDef {
        keyword_type: type_def.keyword_type.clone(),
        name: type_def.name.clone(),
        colon: type_def.colon.clone(),
        ty_ty: Box::new(ty_ty),
        lbrace: type_def.lbrace.clone(),
        variants,
        rbrace: type_def.rbrace.clone(),
        ext: TypedDecorationTypeDef {
            name: type_def.ext.name.clone(),
            id: type_def.ext.id,
        },
    }
}

pub fn typing_type(
    context: &mut RetrieveContext,
    ty: &SynType<RenameDecoration>,
) -> SynType<TypedDecoration> {
    match ty {
        SynType::Atom(type_atom) => {
            let type_atom2 = typing_type_atom(context, type_atom);
            SynType::Atom(type_atom2)
        }
        SynType::App(type_app) => {
            let type_app2 = typing_type_app(context, type_app);
            SynType::App(type_app2)
        }
        SynType::Paren(type_paren) => {
            let type_paren2 = typing_type_paren(context, type_paren);
            SynType::Paren(type_paren2)
        }
        SynType::Map(type_map) => {
            let type_map2 = typing_type_map(context, type_map);
            SynType::Map(type_map2)
        }
        SynType::DependentMap(type_dependent_map) => {
            let type_dependent_map2 = typing_type_dependent_map(context, type_dependent_map);
            SynType::DependentMap(type_dependent_map2)
        }
        SynType::Unit(type_unit) => {
            let type_unit2 = typing_type_unit(context, type_unit);
            SynType::Unit(type_unit2)
        }
    }
}

pub fn typing_type_atom(
    context: &mut RetrieveContext,
    type_atom: &SynTypeAtom<RenameDecoration>,
) -> SynTypeAtom<TypedDecoration> {
    let var_id = *context.def_to_var.get(&type_atom.ext.use_id).unwrap();
    let ty = context.type_checker.get(var_id).unwrap().clone();
    SynTypeAtom {
        ident: type_atom.ident.clone(),
        ext: TypedDecorationTypeAtom {
            use_id: type_atom.ext.use_id,
            ty,
        },
    }
}

pub fn typing_type_app(
    context: &mut RetrieveContext,
    type_app: &SynTypeApp<RenameDecoration>,
) -> SynTypeApp<TypedDecoration> {
    let left = typing_type(context, &type_app.left);
    let right = typing_type(context, &type_app.right);
    SynTypeApp {
        left: Box::new(left),
        right: Box::new(right),
        ext: (),
    }
}

pub fn typing_type_paren(
    context: &mut RetrieveContext,
    type_paren: &SynTypeParen<RenameDecoration>,
) -> SynTypeParen<TypedDecoration> {
    let ty = typing_type(context, &type_paren.ty);
    SynTypeParen {
        lparen: type_paren.lparen.clone(),
        ty: Box::new(ty),
        rparen: type_paren.rparen.clone(),
        ext: (),
    }
}

pub fn typing_type_map(
    context: &mut RetrieveContext,
    type_map: &SynTypeMap<RenameDecoration>,
) -> SynTypeMap<TypedDecoration> {
    let from = typing_type(context, &type_map.from);
    let to = typing_type(context, &type_map.to);
    SynTypeMap {
        from: Box::new(from),
        arrow: type_map.arrow.clone(),
        to: Box::new(to),
        ext: (),
    }
}

pub fn typing_type_dependent_map(
    context: &mut RetrieveContext,
    type_dependent_map: &SynTypeDependentMap<RenameDecoration>,
) -> SynTypeDependentMap<TypedDecoration> {
    let from = typing_typed_arg(context, &type_dependent_map.from);
    let to = typing_type(context, &type_dependent_map.to);
    SynTypeDependentMap {
        from: Box::new(from),
        arrow: type_dependent_map.arrow.clone(),
        to: Box::new(to),
        ext: (),
    }
}

pub fn typing_type_unit(
    _context: &mut RetrieveContext,
    type_unit: &SynTypeUnit<RenameDecoration>,
) -> SynTypeUnit<TypedDecoration> {
    SynTypeUnit {
        lparen: type_unit.lparen.clone(),
        rparen: type_unit.rparen.clone(),
        ext: (),
    }
}

pub fn typing_variant(
    context: &mut RetrieveContext,
    variant: &SynVariant<RenameDecoration>,
) -> SynVariant<TypedDecoration> {
    let ty = typing_type(context, &variant.ty);
    SynVariant {
        name: variant.name.clone(),
        colon: variant.colon.clone(),
        ty,
        camma: variant.camma.clone(),
        ext: TypedDecorationVariant {
            name: variant.ext.name.clone(),
            id: variant.ext.id,
        },
    }
}

pub fn typing_entrypoint(
    _context: &mut RetrieveContext,
    entrypoint: &SynEntrypoint<RenameDecoration>,
) -> SynEntrypoint<TypedDecoration> {
    SynEntrypoint {
        token_entrypoint: entrypoint.token_entrypoint.clone(),
        ident: entrypoint.ident.clone(),
        ext: TypedDecorationEntrypoint {
            use_id: entrypoint.ext.use_id,
        },
    }
}

pub fn typing_statement(
    context: &mut RetrieveContext,
    statement: &SynStatement<RenameDecoration>,
) -> SynStatement<TypedDecoration> {
    match statement {
        SynStatement::ExprSemi(expr_semi) => {
            let expr_semi2 = typing_statement_expr_semi(context, expr_semi);
            SynStatement::ExprSemi(expr_semi2)
        }
        SynStatement::Let(statement_let) => {
            let statement_let2 = typing_statement_let(context, statement_let);
            SynStatement::Let(statement_let2)
        }
        SynStatement::Assign(statement_assign) => {
            let statement_assign2 = typing_statement_assign(context, statement_assign);
            SynStatement::Assign(statement_assign2)
        }
    }
}

pub fn typing_statement_assign(
    context: &mut RetrieveContext,
    statement_assign: &SynStatementAssign<RenameDecoration>,
) -> SynStatementAssign<TypedDecoration> {
    let lhs = typing_expr(context, &statement_assign.lhs);
    let rhs = typing_expr(context, &statement_assign.rhs);
    SynStatementAssign {
        lhs,
        eq: statement_assign.eq.clone(),
        rhs,
        semi: statement_assign.semi.clone(),
    }
}

pub fn typing_expr(
    context: &mut RetrieveContext,
    expr: &SynExpr<RenameDecoration>,
) -> SynExpr<TypedDecoration> {
    match expr {
        SynExpr::App(expr_app) => {
            let expr_app2 = typing_expr_app(context, expr_app);
            SynExpr::App(expr_app2)
        }
        SynExpr::IdentWithPath(expr_ident_with_path) => {
            let expr_ident_with_path2 = typing_expr_ident_with_path(context, expr_ident_with_path);
            SynExpr::IdentWithPath(expr_ident_with_path2)
        }
        SynExpr::Match(expr_match) => {
            let expr_match2 = typing_expr_match(context, expr_match);
            SynExpr::Match(expr_match2)
        }
        SynExpr::Paren(expr_paren) => {
            let expr_paren2 = typing_expr_paren(context, expr_paren);
            SynExpr::Paren(expr_paren2)
        }
        SynExpr::String(expr_string) => {
            let expr_string2 = typing_expr_string(context, expr_string);
            SynExpr::String(expr_string2)
        }
        SynExpr::Number(expr_number) => {
            let expr_number2 = typing_expr_number(context, expr_number);
            SynExpr::Number(expr_number2)
        }
    }
}

pub fn typing_expr_app(
    context: &mut RetrieveContext,
    expr_app: &SynExprApp<RenameDecoration>,
) -> SynExprApp<TypedDecoration> {
    let mut exprs = vec![];
    for expr in &expr_app.exprs {
        let expr2 = typing_expr(context, expr);
        exprs.push(expr2);
    }

    let var_id = *context.def_to_var.get(&expr_app.ext.id).unwrap();
    let ty_term = context.type_checker.get(var_id).unwrap().clone();
    let ext = TypedDecorationExprApp {
        id: expr_app.ext.id,
        ty: ty_term,
    };

    SynExprApp { exprs, ext }
}

pub fn typing_expr_ident_with_path(
    context: &mut RetrieveContext,
    expr_ident_with_path: &SynExprIdentWithPath<RenameDecoration>,
) -> SynExprIdentWithPath<TypedDecoration> {
    let mut path_ids = vec![];
    for path_id in &expr_ident_with_path.ext.path_ids {
        path_ids.push(*path_id);
    }
    let var_id = *context
        .def_to_var
        .get(&expr_ident_with_path.ext.use_id)
        .unwrap();
    let ty = context.type_checker.get(var_id).unwrap().clone();
    SynExprIdentWithPath {
        path: expr_ident_with_path.path.clone(),
        ident: expr_ident_with_path.ident.clone(),
        ext: TypedDecorationExprIdentWithPath {
            path_ids,
            use_id: expr_ident_with_path.ext.use_id,
            ty,
        },
    }
}

pub fn typing_expr_match(
    context: &mut RetrieveContext,
    expr_match: &SynExprMatch<RenameDecoration>,
) -> SynExprMatch<TypedDecoration> {
    let expr = typing_expr(context, &expr_match.expr);
    let mut arms = vec![];
    for arm in &expr_match.arms {
        let arm2 = typing_expr_match_arm(context, arm);
        arms.push(arm2);
    }
    SynExprMatch {
        keyword_match: expr_match.keyword_match.clone(),
        expr: Box::new(expr),
        lbrace: expr_match.lbrace.clone(),
        arms,
        rbrace: expr_match.rbrace.clone(),
        ext: (),
    }
}

pub fn typing_expr_match_arm(
    context: &mut RetrieveContext,
    expr_match_arm: &SynExprMatchArm<RenameDecoration>,
) -> SynExprMatchArm<TypedDecoration> {
    let pattern = typing_expr_match_pattern(context, &expr_match_arm.pattern);
    let expr = typing_expr(context, &expr_match_arm.expr);
    SynExprMatchArm {
        pattern,
        arrow2: expr_match_arm.arrow2.clone(),
        expr,
        camma: expr_match_arm.camma.clone(),
    }
}

pub fn typing_expr_match_pattern(
    context: &mut RetrieveContext,
    expr_match_pattern: &SynExprMatchPattern<RenameDecoration>,
) -> SynExprMatchPattern<TypedDecoration> {
    let type_constructor =
        typing_expr_ident_with_path(context, &expr_match_pattern.type_constructor);
    SynExprMatchPattern {
        type_constructor,
        idents: expr_match_pattern.idents.clone(),
        ext: TypedDecorationExprMatchPattern {
            ids: expr_match_pattern.ext.ids.clone(),
        },
    }
}

pub fn typing_expr_paren(
    context: &mut RetrieveContext,
    expr_paren: &SynExprParen<RenameDecoration>,
) -> SynExprParen<TypedDecoration> {
    let expr = typing_expr(context, &expr_paren.expr);
    SynExprParen {
        lparen: expr_paren.lparen.clone(),
        expr: Box::new(expr),
        rparen: expr_paren.rparen.clone(),
        ext: (),
    }
}

pub fn typing_proc_def(
    context: &mut RetrieveContext,
    proc_def: &SynProcDef<RenameDecoration>,
) -> SynProcDef<TypedDecoration> {
    let ty = typing_type(context, &proc_def.ty);
    let proc_block = typing_proc_block(context, &proc_def.proc_block);
    SynProcDef {
        keyword_proc: proc_def.keyword_proc.clone(),
        name: proc_def.name.clone(),
        colon: proc_def.colon.clone(),
        ty,
        proc_block,
        ext: TypedDecorationProcDef {
            name: proc_def.ext.name.clone(),
            id: proc_def.ext.id,
        },
    }
}

pub fn typing_proc_block(
    context: &mut RetrieveContext,
    proc_block: &SynProcBlock<RenameDecoration>,
) -> SynProcBlock<TypedDecoration> {
    let mut statements = vec![];
    for statement in &proc_block.statements {
        let statement2 = typing_statement(context, statement);
        statements.push(statement2);
    }
    SynProcBlock {
        lbrace: proc_block.lbrace.clone(),
        statements,
        rbrace: proc_block.rbrace.clone(),
    }
}

pub fn typing_typed_arg(
    context: &mut RetrieveContext,
    typed_arg: &SynTypedArg<RenameDecoration>,
) -> SynTypedArg<TypedDecoration> {
    let var_id = *context.def_to_var.get(&typed_arg.ext.id).unwrap();
    let ty_term = context.type_checker.get(var_id).unwrap().clone();
    let ty = typing_type(context, &typed_arg.ty);
    SynTypedArg {
        lparen: typed_arg.lparen.clone(),
        name: typed_arg.name.clone(),
        colon: typed_arg.colon.clone(),
        ty,
        rparen: typed_arg.rparen.clone(),
        ext: TypedDecorationTypedArg {
            name: typed_arg.ext.name.clone(),
            id: typed_arg.ext.id,
            ty: ty_term,
        },
    }
}

pub fn typing_statement_expr_semi(
    context: &mut RetrieveContext,
    expr_semi: &SynStatementExprSemi<RenameDecoration>,
) -> SynStatementExprSemi<TypedDecoration> {
    let expr = typing_expr(context, &expr_semi.expr);
    SynStatementExprSemi {
        expr,
        semi: expr_semi.semi.clone(),
    }
}

pub fn typing_statement_let(
    context: &mut RetrieveContext,
    statement_let: &SynStatementLet<RenameDecoration>,
) -> SynStatementLet<TypedDecoration> {
    let var_id = *context.def_to_var.get(&statement_let.ext.id).unwrap();
    let ty_term = context.type_checker.get(var_id).unwrap().clone();
    let expr = typing_expr(context, &statement_let.expr);
    SynStatementLet {
        keyword_let: statement_let.keyword_let.clone(),
        keyword_mut: statement_let.keyword_mut.clone(),
        name: statement_let.name.clone(),
        eq: statement_let.eq.clone(),
        expr,
        semi: statement_let.semi.clone(),
        ext: TypedDecorationStatementLet {
            name: statement_let.ext.name.clone(),
            id: statement_let.ext.id,
            ty: ty_term,
        },
    }
}

pub fn typing_expr_string(
    context: &mut RetrieveContext,
    expr_string: &SynExprString<RenameDecoration>,
) -> SynExprString<TypedDecoration> {
    let var_id = *context.def_to_var.get(&expr_string.ext.id).unwrap();
    let ty_term = context.type_checker.get(var_id).unwrap().clone();
    SynExprString {
        token_string: expr_string.token_string.clone(),
        ext: TypedDecorationExprString {
            id: expr_string.ext.id,
            ty: ty_term,
        },
    }
}

pub fn typing_expr_number(
    context: &mut RetrieveContext,
    expr_string: &SynExprNumber<RenameDecoration>,
) -> SynExprNumber<TypedDecoration> {
    let var_id = *context.def_to_var.get(&expr_string.ext.id).unwrap();
    let ty_term = context.type_checker.get(var_id).unwrap().clone();
    SynExprNumber {
        number: expr_string.number.clone(),
        ext: TypedDecorationExprNumber {
            id: expr_string.ext.id,
            ty: ty_term,
        },
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

    use crate::retrieve::{retrieve_file, setup_retrieve_context};

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
        let _file_4 = typing_file(&mut context, &file_3);
    }
}
