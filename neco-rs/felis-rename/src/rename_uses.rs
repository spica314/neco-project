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
use neco_resolver::Resolver;

use crate::{
    path_table::PathTable,
    rename_defs::{DefDecoration, DefId},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecoration;

impl Decoration for RenameDecoration {
    type Entrypoint = RenameDecorationEntrypoint;
    type ExprApp = ();
    type ExprBlock = ();
    type ExprIdentWithPath = RenameDecorationExprIdentWithPath;
    type ExprNumber = RenameDecorationExprNumber;
    type ExprMatch = ();
    type ExprParen = ();
    type ExprString = RenameDecorationExprString;
    type Variant = RenameDecorationVariant;
    type TypeDef = RenameDecorationTypeDef;
    type TypeApp = ();
    type TypeAtom = RenameDecorationTypeAtom;
    type TypeMap = ();
    type TypeParen = ();
    type TypeDependentMap = ();
    type TypeUnit = ();
    type File = RenameDecorationFile;
    type FnDef = RenameDecorationFnDef;
    type ProcDef = RenameDecorationProcDef;
    type TypedArg = RenameDecorationTypedArg;
    type ExprMatchPattern = RenameDecorationExprMatchPattern;
    type StatementLet = RenameDecorationStatementLet;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationFile {
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationTypeDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationFnDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationProcDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationTheoremDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationEntrypoint {
    pub use_id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationStatementLet {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationExprIdentWithPath {
    pub path_ids: Vec<DefId>,
    pub use_id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationTypeAtom {
    pub use_id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationVariant {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationTypedArg {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationExprMatchPattern {
    pub ids: Vec<(String, DefId)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationFormulaForall {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationFormulaAtom {
    pub use_id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationExprNumber {
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenameDecorationExprString {
    pub id: DefId,
}

#[derive(Debug, Clone)]
pub struct RenameUseContext {
    resolver: Resolver<DefId>,
    path_table: PathTable,
    use_count: usize,
}

impl RenameUseContext {
    pub fn new(resolver: Resolver<DefId>, path_table: PathTable) -> Self {
        Self {
            resolver,
            path_table,
            use_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenameError {
    Error,
    NotFound,
    AlreadyExists,
}

pub fn rename_uses_file(
    context: &mut RenameUseContext,
    file: &SynFile<DefDecoration>,
) -> Result<SynFile<RenameDecoration>, RenameError> {
    let mut items = vec![];
    for item in &file.items {
        let item2 = rename_uses_file_item(context, item)?;
        items.push(item2);
    }
    let ext = RenameDecorationFile { id: file.ext.id };
    Ok(SynFile { items, ext })
}

pub fn rename_uses_file_item(
    context: &mut RenameUseContext,
    item: &SynFileItem<DefDecoration>,
) -> Result<SynFileItem<RenameDecoration>, RenameError> {
    match item {
        SynFileItem::TypeDef(type_def) => {
            let type_def2 = rename_uses_type_def(context, type_def)?;
            Ok(SynFileItem::TypeDef(type_def2))
        }
        SynFileItem::Entrypoint(entrypoint) => {
            let entrypoint2 = rename_uses_entrypoint(context, entrypoint)?;
            Ok(SynFileItem::Entrypoint(entrypoint2))
        }
        SynFileItem::ProcDef(proc_def) => {
            let proc_def2 = rename_uses_proc_def(context, proc_def)?;
            Ok(SynFileItem::ProcDef(proc_def2))
        }
    }
}

pub fn rename_uses_type_def(
    context: &mut RenameUseContext,
    type_def: &SynTypeDef<DefDecoration>,
) -> Result<SynTypeDef<RenameDecoration>, RenameError> {
    context.resolver.enter_scope();
    context
        .resolver
        .set(type_def.name.as_str().to_string(), type_def.ext.id);

    let ty_ty = rename_uses_type(context, &type_def.ty_ty)?;

    let mut variants = vec![];
    for variant in &type_def.variants {
        let variant2 = rename_uses_variant(context, variant)?;
        variants.push(variant2);
    }

    let ext = RenameDecorationTypeDef {
        name: type_def.ext.name.clone(),
        id: type_def.ext.id,
    };

    context.resolver.leave_scope();

    let type_def2 = SynTypeDef {
        keyword_type: type_def.keyword_type.clone(),
        name: type_def.name.clone(),
        colon: type_def.colon.clone(),
        ty_ty: Box::new(ty_ty),
        lbrace: type_def.lbrace.clone(),
        variants,
        rbrace: type_def.rbrace.clone(),
        ext,
    };
    Ok(type_def2)
}

pub fn rename_uses_proc_def(
    context: &mut RenameUseContext,
    proc_def: &SynProcDef<DefDecoration>,
) -> Result<SynProcDef<RenameDecoration>, RenameError> {
    context.resolver.enter_scope();

    let ty = rename_uses_type(context, &proc_def.ty)?;

    let proc_block = rename_uses_proc_block(context, &proc_def.proc_block)?;

    let ext = RenameDecorationProcDef {
        name: proc_def.ext.name.clone(),
        id: proc_def.ext.id,
    };

    context.resolver.leave_scope();

    let proc_def2 = SynProcDef {
        keyword_proc: proc_def.keyword_proc.clone(),
        name: proc_def.name.clone(),
        colon: proc_def.colon.clone(),
        ty,
        proc_block,
        ext,
    };
    Ok(proc_def2)
}

pub fn rename_uses_entrypoint(
    context: &mut RenameUseContext,
    entrypoint: &SynEntrypoint<DefDecoration>,
) -> Result<SynEntrypoint<RenameDecoration>, RenameError> {
    let id = context.resolver.get(entrypoint.ident.as_str()).unwrap();
    let ext = RenameDecorationEntrypoint { use_id: *id };
    context.use_count += 1;

    let entrypoint2 = SynEntrypoint {
        token_entrypoint: entrypoint.token_entrypoint.clone(),
        ident: entrypoint.ident.clone(),
        ext,
    };
    Ok(entrypoint2)
}

pub fn rename_uses_proc_block(
    context: &mut RenameUseContext,
    proc_block: &SynProcBlock<DefDecoration>,
) -> Result<SynProcBlock<RenameDecoration>, RenameError> {
    let mut statements = vec![];
    for statement in &proc_block.statements {
        let statement2 = rename_uses_statement(context, statement)?;
        statements.push(statement2);
    }

    let proc_block2 = SynProcBlock {
        lbrace: proc_block.lbrace.clone(),
        statements,
        rbrace: proc_block.rbrace.clone(),
    };
    Ok(proc_block2)
}

pub fn rename_uses_statement(
    context: &mut RenameUseContext,
    statement: &SynStatement<DefDecoration>,
) -> Result<SynStatement<RenameDecoration>, RenameError> {
    match statement {
        SynStatement::Expr(expr) => {
            let expr2 = rename_uses_expr(context, expr)?;
            Ok(SynStatement::Expr(expr2))
        }
        SynStatement::Let(let_) => {
            let let_2 = rename_uses_let(context, let_)?;
            Ok(SynStatement::Let(let_2))
        }
        SynStatement::ExprSemi(expr_semi) => {
            let expr_semi2 = rename_uses_statement_expr_semi(context, expr_semi)?;
            Ok(SynStatement::ExprSemi(expr_semi2))
        }
        SynStatement::Assign(assign) => {
            let assign2 = rename_uses_statement_assign(context, assign)?;
            Ok(SynStatement::Assign(assign2))
        }
    }
}

pub fn rename_uses_statement_assign(
    context: &mut RenameUseContext,
    assign: &SynStatementAssign<DefDecoration>,
) -> Result<SynStatementAssign<RenameDecoration>, RenameError> {
    let lhs = rename_uses_expr(context, &assign.lhs)?;
    let rhs = rename_uses_expr(context, &assign.rhs)?;

    let assign2 = SynStatementAssign {
        lhs,
        eq: assign.eq.clone(),
        rhs,
        semi: assign.semi.clone(),
    };
    Ok(assign2)
}

pub fn rename_uses_let(
    context: &mut RenameUseContext,
    let_: &SynStatementLet<DefDecoration>,
) -> Result<SynStatementLet<RenameDecoration>, RenameError> {
    context
        .resolver
        .set(let_.name.as_str().to_string(), let_.ext.id);

    let expr = rename_uses_expr(context, &let_.expr)?;

    let ext = RenameDecorationStatementLet {
        name: let_.ext.name.clone(),
        id: let_.ext.id,
    };

    let let_2 = SynStatementLet {
        keyword_let: let_.keyword_let.clone(),
        keyword_mut: let_.keyword_mut.clone(),
        name: let_.name.clone(),
        eq: let_.eq.clone(),
        expr,
        semi: let_.semi.clone(),
        ext,
    };
    Ok(let_2)
}

pub fn rename_uses_statement_expr_semi(
    context: &mut RenameUseContext,
    expr_semi: &SynStatementExprSemi<DefDecoration>,
) -> Result<SynStatementExprSemi<RenameDecoration>, RenameError> {
    let expr = rename_uses_expr(context, &expr_semi.expr)?;

    let expr_semi2 = SynStatementExprSemi {
        expr,
        semi: expr_semi.semi.clone(),
    };
    Ok(expr_semi2)
}

pub fn rename_uses_expr(
    context: &mut RenameUseContext,
    expr: &SynExpr<DefDecoration>,
) -> Result<SynExpr<RenameDecoration>, RenameError> {
    match expr {
        SynExpr::App(app) => {
            let app2 = rename_uses_app(context, app)?;
            Ok(SynExpr::App(app2))
        }
        SynExpr::Paren(paren) => {
            let paren2 = rename_uses_paren(context, paren)?;
            Ok(SynExpr::Paren(paren2))
        }
        SynExpr::String(string) => {
            let string2 = rename_uses_string(context, string)?;
            Ok(SynExpr::String(string2))
        }
        SynExpr::IdentWithPath(ident_with_path) => {
            let ident_with_path2 = rename_uses_ident_with_path(context, ident_with_path)?;
            Ok(SynExpr::IdentWithPath(ident_with_path2))
        }
        SynExpr::Match(expr_match) => {
            let expr_match2 = rename_uses_match(context, expr_match)?;
            Ok(SynExpr::Match(expr_match2))
        }
        SynExpr::Number(number) => {
            let number2 = rename_uses_number(context, number)?;
            Ok(SynExpr::Number(number2))
        }
    }
}

pub fn rename_uses_ident_with_path(
    context: &mut RenameUseContext,
    ident_with_path: &SynExprIdentWithPath<DefDecoration>,
) -> Result<SynExprIdentWithPath<RenameDecoration>, RenameError> {
    // todo: fix
    let mut path_ids = vec![];

    let mut id = None;
    for (ident, _) in &ident_with_path.path {
        if id.is_none() {
            let id2 = context.resolver.get(ident.as_str()).unwrap();
            path_ids.push(*id2);
            id = Some(*id2);
        } else {
            let id2 = context
                .path_table
                .get(id.unwrap())
                .unwrap()
                .children
                .get(ident.as_str())
                .unwrap();
            path_ids.push(*id2);
            id = Some(*id2);
        }
    }
    let id = if id.is_none() {
        context
            .resolver
            .get(ident_with_path.ident.as_str())
            .unwrap()
    } else {
        let id2 = context
            .path_table
            .get(id.unwrap())
            .unwrap()
            .children
            .get(ident_with_path.ident.as_str())
            .unwrap();
        id2
    };
    let ext = RenameDecorationExprIdentWithPath {
        path_ids,
        use_id: *id,
    };
    context.use_count += 1;

    let ident_with_path2 = SynExprIdentWithPath {
        path: ident_with_path.path.clone(),
        ident: ident_with_path.ident.clone(),
        ext,
    };
    Ok(ident_with_path2)
}

pub fn rename_uses_app(
    context: &mut RenameUseContext,
    app: &SynExprApp<DefDecoration>,
) -> Result<SynExprApp<RenameDecoration>, RenameError> {
    let mut exprs = vec![];
    for expr in &app.exprs {
        let expr2 = rename_uses_expr(context, expr)?;
        exprs.push(expr2);
    }

    let app2 = SynExprApp { exprs, ext: () };
    Ok(app2)
}

pub fn rename_uses_paren(
    context: &mut RenameUseContext,
    paren: &SynExprParen<DefDecoration>,
) -> Result<SynExprParen<RenameDecoration>, RenameError> {
    let expr = rename_uses_expr(context, &paren.expr)?;

    let paren2 = SynExprParen {
        lparen: paren.lparen.clone(),
        expr: Box::new(expr),
        rparen: paren.rparen.clone(),
        ext: (),
    };
    Ok(paren2)
}

pub fn rename_uses_string(
    _context: &mut RenameUseContext,
    string: &SynExprString<DefDecoration>,
) -> Result<SynExprString<RenameDecoration>, RenameError> {
    let ext = RenameDecorationExprString { id: string.ext.id };
    let string2 = SynExprString {
        token_string: string.token_string.clone(),
        ext,
    };
    Ok(string2)
}

pub fn rename_uses_number(
    _context: &mut RenameUseContext,
    number: &SynExprNumber<DefDecoration>,
) -> Result<SynExprNumber<RenameDecoration>, RenameError> {
    let ext = RenameDecorationExprNumber { id: number.ext.id };
    let number2 = SynExprNumber {
        number: number.number.clone(),
        ext,
    };
    Ok(number2)
}

pub fn rename_uses_type(
    context: &mut RenameUseContext,
    ty: &SynType<DefDecoration>,
) -> Result<SynType<RenameDecoration>, RenameError> {
    match ty {
        SynType::App(ty_app) => {
            let ty_app2 = rename_uses_type_app(context, ty_app)?;
            Ok(SynType::App(ty_app2))
        }
        SynType::Atom(ty_atom) => {
            let ty_atom2 = rename_uses_type_atom(context, ty_atom)?;
            Ok(SynType::Atom(ty_atom2))
        }
        SynType::Map(ty_map) => {
            let ty_map2 = rename_uses_type_map(context, ty_map)?;
            Ok(SynType::Map(ty_map2))
        }
        SynType::Paren(ty_paren) => {
            let ty_paren2 = rename_uses_type_paren(context, ty_paren)?;
            Ok(SynType::Paren(ty_paren2))
        }
        SynType::DependentMap(ty_dependent_map) => {
            let ty_dependent_map2 = rename_uses_type_dependent_map(context, ty_dependent_map)?;
            Ok(SynType::DependentMap(ty_dependent_map2))
        }
        SynType::Unit(ty_unit) => {
            let ty_unit2 = rename_uses_type_unit(context, ty_unit)?;
            Ok(SynType::Unit(ty_unit2))
        }
    }
}

pub fn rename_uses_type_app(
    context: &mut RenameUseContext,
    ty_app: &SynTypeApp<DefDecoration>,
) -> Result<SynTypeApp<RenameDecoration>, RenameError> {
    let left = rename_uses_type(context, &ty_app.left)?;
    let right = rename_uses_type(context, &ty_app.right)?;

    #[allow(clippy::let_unit_value)]
    let ext = ();

    let ty_app2 = SynTypeApp {
        left: Box::new(left),
        right: Box::new(right),
        ext,
    };

    Ok(ty_app2)
}

pub fn rename_uses_type_atom(
    context: &mut RenameUseContext,
    ty_atom: &SynTypeAtom<DefDecoration>,
) -> Result<SynTypeAtom<RenameDecoration>, RenameError> {
    let id = context.resolver.get(ty_atom.ident.as_str()).unwrap();
    let ext = RenameDecorationTypeAtom { use_id: *id };
    context.use_count += 1;

    let ty_atom2 = SynTypeAtom {
        ident: ty_atom.ident.clone(),
        ext,
    };

    Ok(ty_atom2)
}

pub fn rename_uses_type_map(
    context: &mut RenameUseContext,
    ty_map: &SynTypeMap<DefDecoration>,
) -> Result<SynTypeMap<RenameDecoration>, RenameError> {
    let from = rename_uses_type(context, &ty_map.from)?;
    let to = rename_uses_type(context, &ty_map.to)?;

    #[allow(clippy::let_unit_value)]
    let ext = ();

    let ty_map2 = SynTypeMap {
        from: Box::new(from),
        arrow: ty_map.arrow.clone(),
        to: Box::new(to),
        ext,
    };

    Ok(ty_map2)
}

pub fn rename_uses_type_paren(
    context: &mut RenameUseContext,
    ty_paren: &SynTypeParen<DefDecoration>,
) -> Result<SynTypeParen<RenameDecoration>, RenameError> {
    let ty = rename_uses_type(context, &ty_paren.ty)?;

    #[allow(clippy::let_unit_value)]
    let ext = ();

    let ty_paren2 = SynTypeParen {
        lparen: ty_paren.lparen.clone(),
        ty: Box::new(ty),
        rparen: ty_paren.rparen.clone(),
        ext,
    };

    Ok(ty_paren2)
}

pub fn rename_uses_type_dependent_map(
    context: &mut RenameUseContext,
    ty_dependent_map: &SynTypeDependentMap<DefDecoration>,
) -> Result<SynTypeDependentMap<RenameDecoration>, RenameError> {
    let from = rename_uses_typed_arg(context, &ty_dependent_map.from)?;
    let to = rename_uses_type(context, &ty_dependent_map.to)?;

    #[allow(clippy::let_unit_value)]
    let ext = ();

    let ty_dependent_map_2 = SynTypeDependentMap {
        from: Box::new(from),
        arrow: ty_dependent_map.arrow.clone(),
        to: Box::new(to),
        ext,
    };

    Ok(ty_dependent_map_2)
}

pub fn rename_uses_type_unit(
    _context: &mut RenameUseContext,
    ty_unit: &SynTypeUnit<DefDecoration>,
) -> Result<SynTypeUnit<RenameDecoration>, RenameError> {
    #[allow(clippy::let_unit_value)]
    let ext = ();

    let ty_unit2 = SynTypeUnit {
        lparen: ty_unit.lparen.clone(),
        rparen: ty_unit.rparen.clone(),
        ext,
    };

    Ok(ty_unit2)
}

pub fn rename_uses_variant(
    context: &mut RenameUseContext,
    variant: &SynVariant<DefDecoration>,
) -> Result<SynVariant<RenameDecoration>, RenameError> {
    let ty = rename_uses_type(context, &variant.ty)?;

    let ext = RenameDecorationVariant {
        name: variant.ext.name.clone(),
        id: variant.ext.id,
    };

    let variant2 = SynVariant {
        name: variant.name.clone(),
        colon: variant.colon.clone(),
        ty,
        camma: variant.camma.clone(),
        ext,
    };

    Ok(variant2)
}

pub fn rename_uses_match(
    context: &mut RenameUseContext,
    expr_match: &SynExprMatch<DefDecoration>,
) -> Result<SynExprMatch<RenameDecoration>, RenameError> {
    let expr = rename_uses_expr(context, &expr_match.expr)?;

    let mut arms = vec![];
    for arm in &expr_match.arms {
        let arm2 = rename_uses_expr_match_arm(context, arm)?;
        arms.push(arm2);
    }

    let expr_match2 = SynExprMatch {
        keyword_match: expr_match.keyword_match.clone(),
        expr: Box::new(expr),
        lbrace: expr_match.lbrace.clone(),
        arms,
        rbrace: expr_match.rbrace.clone(),
        ext: (),
    };

    Ok(expr_match2)
}

pub fn rename_uses_expr_match_arm(
    context: &mut RenameUseContext,
    expr_match_arm: &SynExprMatchArm<DefDecoration>,
) -> Result<SynExprMatchArm<RenameDecoration>, RenameError> {
    context.resolver.enter_scope();

    let pattern = rename_uses_expr_match_pattern(context, &expr_match_arm.pattern)?;
    let expr = rename_uses_expr(context, &expr_match_arm.expr)?;

    context.resolver.leave_scope();

    let expr_match_arm2 = SynExprMatchArm {
        pattern,
        arrow2: expr_match_arm.arrow2.clone(),
        expr,
        camma: expr_match_arm.camma.clone(),
    };

    Ok(expr_match_arm2)
}

pub fn rename_uses_expr_match_pattern(
    context: &mut RenameUseContext,
    expr_match_pattern: &SynExprMatchPattern<DefDecoration>,
) -> Result<SynExprMatchPattern<RenameDecoration>, RenameError> {
    let type_constructor =
        rename_uses_ident_with_path(context, &expr_match_pattern.type_constructor)?;

    let ext = RenameDecorationExprMatchPattern {
        ids: expr_match_pattern.ext.ids.clone(),
    };
    for (name, id) in &expr_match_pattern.ext.ids {
        context.resolver.set(name.clone(), *id);
    }

    let expr_match_pattern2 = SynExprMatchPattern {
        type_constructor,
        idents: expr_match_pattern.idents.clone(),
        ext,
    };

    Ok(expr_match_pattern2)
}

pub fn rename_uses_match_pattern(
    _context: &mut RenameUseContext,
    _expr_match_pattern: &SynExpr<DefDecoration>,
) -> Result<SynExpr<RenameDecoration>, RenameError> {
    todo!()
}

pub fn rename_uses_typed_arg(
    context: &mut RenameUseContext,
    typed_arg: &SynTypedArg<DefDecoration>,
) -> Result<SynTypedArg<RenameDecoration>, RenameError> {
    context
        .resolver
        .set(typed_arg.name.as_str().to_string(), typed_arg.ext.id);

    let ty = rename_uses_type(context, &typed_arg.ty)?;

    let ext = RenameDecorationTypedArg {
        name: typed_arg.ext.name.clone(),
        id: typed_arg.ext.id,
    };

    let typed_arg2 = SynTypedArg {
        lparen: typed_arg.lparen.clone(),
        name: typed_arg.name.clone(),
        colon: typed_arg.colon.clone(),
        ty,
        rparen: typed_arg.rparen.clone(),
        ext,
    };

    Ok(typed_arg2)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{
        path_table::construct_path_table_syn_file, rename_defs::RenameDefContext, rename_defs::*,
    };
    use felis_syn::{decoration::UD, test_utils::parse_from_str};

    #[test]
    fn felis_rename_uses_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        // [file], A, hoge
        assert_eq!(context.def_count(), 3);

        let mut resolver = Resolver::new();
        let prop_def_id = context.new_id();
        resolver.set("Prop".to_string(), prop_def_id);
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        let SynFileItem::TypeDef(type_def) = &file_3.items[0] else {
            panic!()
        };
        let a_def_id = type_def.ext.id;
        let SynVariant {
            ty: SynType::Atom(ty),
            ..
        } = &type_def.variants[0]
        else {
            panic!()
        };
        let a_use_id = ty.ext.use_id;
        assert_eq!(a_use_id, a_def_id);
        let SynType::Atom(atom) = type_def.ty_ty.as_ref() else {
            panic!()
        };
        let prop_use_id = atom.ext.use_id;
        assert_eq!(prop_use_id, prop_def_id);
    }

    #[test]
    fn felis_rename_uses_test_1_2() {
        let s = "#type A : Prop { hoge : (A), }";
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        // [file], A, hoge
        assert_eq!(context.def_count(), 3);

        let mut resolver = Resolver::new();
        let prop_def_id = context.new_id();
        resolver.set("Prop".to_string(), prop_def_id);
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        let SynFileItem::TypeDef(type_def) = &file_3.items[0] else {
            panic!()
        };
        let a_def_id = type_def.ext.id;
        let SynVariant {
            ty: SynType::Paren(ty),
            ..
        } = &type_def.variants[0]
        else {
            panic!()
        };
        let SynType::Atom(ty) = ty.ty.as_ref() else {
            panic!()
        };
        let a_use_id = ty.ext.use_id;
        assert_eq!(a_use_id, a_def_id);
        let SynType::Atom(atom) = type_def.ty_ty.as_ref() else {
            panic!()
        };
        let prop_use_id = atom.ext.use_id;
        assert_eq!(prop_use_id, prop_def_id);
    }

    #[test]
    fn felis_rename_uses_test_4() {
        let s = std::fs::read_to_string("../../library/wip/and2.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        // [file], And, conj, A, B
        assert_eq!(context.def_count(), 5);

        let mut resolver = Resolver::new();
        let prop_def_id = context.new_id();
        resolver.set("Prop".to_string(), prop_def_id);
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let _file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        // (3) Prop, Prop, Prop
        // (7) Prop, Prop, A, B, And, A, B
        assert_eq!(context2.use_count, 10);
    }
}
