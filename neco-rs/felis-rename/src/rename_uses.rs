use felis_syn::{
    decoration::Decoration,
    syn_entrypoint::SynEntrypoint,
    syn_expr::{
        SynExpr, SynExprApp, SynExprBlock, SynExprIdent, SynExprIdentWithPath, SynExprMatch,
        SynExprMatchArm, SynExprMatchPattern, SynExprParen, SynExprString,
    },
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::{SynFnBlock, SynFnDef},
    syn_formula::{
        SynFormula, SynFormulaApp, SynFormulaAtom, SynFormulaForall, SynFormulaImplies,
        SynFormulaParen,
    },
    syn_statement::{SynStatement, SynStatementLet},
    syn_theorem_def::SynTheoremDef,
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
    type ExprIdent = RenameDecorationExprIdent;
    type ExprMatch = ();
    type ExprParen = ();
    type ExprString = ();
    type FormulaForall = RenameDecorationFormulaForall;
    type FormulaImplies = ();
    type FormulaAtom = RenameDecorationFormulaAtom;
    type FormulaApp = ();
    type FormulaParen = ();
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
    type TheoremDef = RenameDecorationTheoremDef;
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
pub struct RenameDecorationExprIdent {
    pub use_id: DefId,
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
        SynFileItem::FnDef(fn_def) => {
            let fn_def2 = rename_uses_fn_def(context, fn_def)?;
            Ok(SynFileItem::FnDef(fn_def2))
        }
        SynFileItem::TheoremDef(theorem_def) => {
            let theorem_def2 = rename_uses_theorem_def(context, theorem_def)?;
            Ok(SynFileItem::TheoremDef(theorem_def2))
        }
        SynFileItem::Entrypoint(entrypoint) => {
            let entrypoint2 = rename_uses_entrypoint(context, entrypoint)?;
            Ok(SynFileItem::Entrypoint(entrypoint2))
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

pub fn rename_uses_fn_def(
    context: &mut RenameUseContext,
    fn_def: &SynFnDef<DefDecoration>,
) -> Result<SynFnDef<RenameDecoration>, RenameError> {
    context.resolver.enter_scope();

    let ty = rename_uses_type(context, &fn_def.ty)?;

    let fn_block = rename_uses_fn_block(context, &fn_def.fn_block)?;

    let ext = RenameDecorationFnDef {
        name: fn_def.ext.name.clone(),
        id: fn_def.ext.id,
    };

    context.resolver.leave_scope();

    let fn_def2 = SynFnDef {
        keyword_fn: fn_def.keyword_fn.clone(),
        name: fn_def.name.clone(),
        colon: fn_def.colon.clone(),
        ty,
        fn_block,
        ext,
    };
    Ok(fn_def2)
}

pub fn rename_uses_theorem_def(
    context: &mut RenameUseContext,
    theorem_def: &SynTheoremDef<DefDecoration>,
) -> Result<SynTheoremDef<RenameDecoration>, RenameError> {
    context.resolver.enter_scope();

    let formula = rename_uses_formula(context, &theorem_def.formula)?;

    context.resolver.leave_scope();

    let fn_def = rename_uses_fn_def(context, &theorem_def.fn_def)?;

    let ext = RenameDecorationTheoremDef {
        name: theorem_def.ext.name.clone(),
        id: theorem_def.ext.id,
    };

    let theorem_def2 = SynTheoremDef {
        keyword_theorem: theorem_def.keyword_theorem.clone(),
        name: theorem_def.name.clone(),
        eq: theorem_def.eq.clone(),
        formula,
        lbrace: theorem_def.lbrace.clone(),
        fn_def,
        rbrace: theorem_def.rbrace.clone(),
        ext,
    };
    Ok(theorem_def2)
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

pub fn rename_uses_fn_block(
    context: &mut RenameUseContext,
    fn_block: &SynFnBlock<DefDecoration>,
) -> Result<SynFnBlock<RenameDecoration>, RenameError> {
    let mut statements = vec![];
    for statement in &fn_block.statements {
        let statement2 = rename_uses_statement(context, statement)?;
        statements.push(statement2);
    }

    let fn_block2 = SynFnBlock {
        lbrace: fn_block.lbrace.clone(),
        statements,
        rbrace: fn_block.rbrace.clone(),
    };
    Ok(fn_block2)
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
    }
}

pub fn rename_uses_let(
    context: &mut RenameUseContext,
    let_: &SynStatementLet<DefDecoration>,
) -> Result<SynStatementLet<RenameDecoration>, RenameError> {
    let expr = rename_uses_expr(context, &let_.expr)?;

    let ext = RenameDecorationStatementLet {
        name: let_.ext.name.clone(),
        id: let_.ext.id,
    };

    let let_2 = SynStatementLet {
        keyword_let: let_.keyword_let.clone(),
        name: let_.name.clone(),
        eq: let_.eq.clone(),
        expr,
        semi: let_.semi.clone(),
        ext,
    };
    Ok(let_2)
}

pub fn rename_uses_expr(
    context: &mut RenameUseContext,
    expr: &SynExpr<DefDecoration>,
) -> Result<SynExpr<RenameDecoration>, RenameError> {
    match expr {
        SynExpr::Ident(ident) => {
            let ident2 = rename_uses_ident(context, ident)?;
            Ok(SynExpr::Ident(ident2))
        }
        SynExpr::App(app) => {
            let app2 = rename_uses_app(context, app)?;
            Ok(SynExpr::App(app2))
        }
        SynExpr::Paren(paren) => {
            let paren2 = rename_uses_paren(context, paren)?;
            Ok(SynExpr::Paren(paren2))
        }
        SynExpr::Block(block) => {
            let block2 = rename_uses_block(context, block)?;
            Ok(SynExpr::Block(block2))
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
    }
}

pub fn rename_uses_ident(
    context: &mut RenameUseContext,
    ident: &SynExprIdent<DefDecoration>,
) -> Result<SynExprIdent<RenameDecoration>, RenameError> {
    let id = context.resolver.get(ident.ident.as_str()).unwrap();
    let ext = RenameDecorationExprIdent { use_id: *id };
    context.use_count += 1;

    let ident2 = SynExprIdent {
        ident: ident.ident.clone(),
        ext,
    };
    Ok(ident2)
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

pub fn rename_uses_block(
    context: &mut RenameUseContext,
    block: &SynExprBlock<DefDecoration>,
) -> Result<SynExprBlock<RenameDecoration>, RenameError> {
    let mut statements = vec![];
    for statement in &block.statements {
        let statement2 = rename_uses_statement(context, statement)?;
        statements.push(statement2);
    }

    let block2 = SynExprBlock {
        lbrace: block.lbrace.clone(),
        statements,
        rbrace: block.rbrace.clone(),
        ext: (),
    };
    Ok(block2)
}

pub fn rename_uses_string(
    _context: &mut RenameUseContext,
    string: &SynExprString<DefDecoration>,
) -> Result<SynExprString<RenameDecoration>, RenameError> {
    let string2 = SynExprString {
        token_string: string.token_string.clone(),
        ext: (),
    };
    Ok(string2)
}

pub fn rename_uses_formula(
    context: &mut RenameUseContext,
    formula: &SynFormula<DefDecoration>,
) -> Result<SynFormula<RenameDecoration>, RenameError> {
    match formula {
        SynFormula::Forall(formula_forall) => {
            let formula_forall2 = rename_uses_formula_forall(context, formula_forall)?;
            Ok(SynFormula::Forall(formula_forall2))
        }
        SynFormula::Implies(formula_implies) => {
            let formula_implies2 = rename_uses_formula_implies(context, formula_implies)?;
            Ok(SynFormula::Implies(formula_implies2))
        }
        SynFormula::Atom(formula_atom) => {
            let formula_atom2 = rename_uses_formula_atom(context, formula_atom)?;
            Ok(SynFormula::Atom(formula_atom2))
        }
        SynFormula::App(formula_app) => {
            let formula_app2 = rename_uses_formula_app(context, formula_app)?;
            Ok(SynFormula::App(formula_app2))
        }
        SynFormula::Paren(formula_paren) => {
            let formula_paren2 = rename_uses_formula_paren(context, formula_paren)?;
            Ok(SynFormula::Paren(formula_paren2))
        }
    }
}

pub fn rename_uses_formula_forall(
    context: &mut RenameUseContext,
    formula_forall: &SynFormulaForall<DefDecoration>,
) -> Result<SynFormulaForall<RenameDecoration>, RenameError> {
    context.resolver.set(
        formula_forall.name.as_str().to_string(),
        formula_forall.ext.id,
    );

    let ty = rename_uses_formula(context, &formula_forall.ty)?;
    let child = rename_uses_formula(context, &formula_forall.child)?;

    let ext = RenameDecorationFormulaForall {
        name: formula_forall.ext.name.clone(),
        id: formula_forall.ext.id,
    };

    let formula_forall2 = SynFormulaForall {
        keyword_forall: formula_forall.keyword_forall.clone(),
        lparen: formula_forall.lparen.clone(),
        name: formula_forall.name.clone(),
        colon: formula_forall.colon.clone(),
        ty: Box::new(ty),
        rparen: formula_forall.rparen.clone(),
        camma: formula_forall.camma.clone(),
        child: Box::new(child),
        ext,
    };
    Ok(formula_forall2)
}

pub fn rename_uses_formula_implies(
    context: &mut RenameUseContext,
    formula_implies: &SynFormulaImplies<DefDecoration>,
) -> Result<SynFormulaImplies<RenameDecoration>, RenameError> {
    let lhs = rename_uses_formula(context, &formula_implies.lhs)?;
    let rhs = rename_uses_formula(context, &formula_implies.rhs)?;

    let formula_implies2 = SynFormulaImplies {
        lhs: Box::new(lhs),
        arrow: formula_implies.arrow.clone(),
        rhs: Box::new(rhs),
        ext: (),
    };
    Ok(formula_implies2)
}

pub fn rename_uses_formula_atom(
    context: &mut RenameUseContext,
    formula_atom: &SynFormulaAtom<DefDecoration>,
) -> Result<SynFormulaAtom<RenameDecoration>, RenameError> {
    let id = context.resolver.get(formula_atom.ident.as_str()).unwrap();
    let ext = RenameDecorationFormulaAtom { use_id: *id };
    context.use_count += 1;

    let formula_atom2 = SynFormulaAtom {
        ident: formula_atom.ident.clone(),
        ext,
    };

    Ok(formula_atom2)
}

pub fn rename_uses_formula_app(
    context: &mut RenameUseContext,
    formula_app: &SynFormulaApp<DefDecoration>,
) -> Result<SynFormulaApp<RenameDecoration>, RenameError> {
    let fun = rename_uses_formula(context, &formula_app.fun)?;
    let arg = rename_uses_formula(context, &formula_app.arg)?;

    #[allow(clippy::let_unit_value)]
    let ext = ();

    let formula_app2 = SynFormulaApp {
        fun: Box::new(fun),
        arg: Box::new(arg),
        ext,
    };

    Ok(formula_app2)
}

pub fn rename_uses_formula_paren(
    _context: &mut RenameUseContext,
    _formula_paren: &SynFormulaParen<DefDecoration>,
) -> Result<SynFormulaParen<RenameDecoration>, RenameError> {
    todo!()
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
    fn felis_rename_uses_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        // [file], proof, A, B, x, l, r
        assert_eq!(context.def_count(), 7);

        let mut resolver = Resolver::new();
        let prop_def_id = context.new_id();
        resolver.set("Prop".to_string(), prop_def_id);
        let and_def_id = context.new_id();
        resolver.set("And".to_string(), and_def_id);
        let or_def_id = context.new_id();
        resolver.set("Or".to_string(), or_def_id);
        let conj_def_id = context.new_id();
        resolver.set("conj".to_string(), conj_def_id);
        let or_introl_def_id = context.new_id();
        resolver.set("or_introl".to_string(), or_introl_def_id);
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        let SynFileItem::FnDef(fn_def) = &file_3.items[0] else {
            panic!()
        };
        let _proof_def_id = fn_def.ext.id;
        let SynType::DependentMap(dep_map1) = &fn_def.ty else {
            panic!()
        };
        let a_def_id = dep_map1.from.ext.id;
        let SynType::Atom(atom) = &dep_map1.from.ty else {
            panic!()
        };
        let prop_use_id = atom.ext.use_id;
        assert_eq!(prop_use_id, prop_def_id);
        let SynType::DependentMap(dep_map2) = &dep_map1.to.as_ref() else {
            panic!()
        };
        let b_def_id = dep_map2.from.ext.id;
        let SynType::Atom(atom) = &dep_map2.from.ty else {
            panic!()
        };
        let prop_use_id = atom.ext.use_id;
        assert_eq!(prop_use_id, prop_def_id);
        let SynType::DependentMap(dep_map3) = &dep_map2.to.as_ref() else {
            panic!()
        };
        let x_def_id = dep_map3.from.ext.id;
        let SynType::App(app1) = &dep_map3.from.ty else {
            panic!()
        };
        let SynType::Atom(atom) = app1.right.as_ref() else {
            panic!()
        };
        let b_use_id = atom.ext.use_id;
        assert_eq!(b_use_id, b_def_id);
        let SynType::App(app2) = app1.left.as_ref() else {
            panic!()
        };
        let SynType::Atom(atom) = app2.right.as_ref() else {
            panic!()
        };
        let a_use_id = atom.ext.use_id;
        assert_eq!(a_use_id, a_def_id);
        let SynType::Atom(atom) = app2.left.as_ref() else {
            panic!()
        };
        let and_use_id = atom.ext.use_id;
        assert_eq!(and_use_id, and_def_id);
        let SynType::App(app1) = dep_map3.to.as_ref() else {
            panic!()
        };
        let SynType::Atom(atom) = app1.right.as_ref() else {
            panic!()
        };
        let b_use_id = atom.ext.use_id;
        assert_eq!(b_use_id, b_def_id);
        let SynType::App(app2) = app1.left.as_ref() else {
            panic!()
        };
        let SynType::Atom(atom) = app2.right.as_ref() else {
            panic!()
        };
        let a_use_id = atom.ext.use_id;
        assert_eq!(a_use_id, a_def_id);
        let SynType::Atom(atom) = app2.left.as_ref() else {
            panic!()
        };
        let or_use_id = atom.ext.use_id;
        assert_eq!(or_use_id, or_def_id);
        let SynStatement::Expr(expr) = &fn_def.fn_block.statements[0] else {
            panic!()
        };
        let SynExpr::Match(expr_match) = &expr else {
            panic!()
        };
        let SynExpr::IdentWithPath(ident_with_path) = expr_match.expr.as_ref() else {
            panic!()
        };
        let x_use_id = ident_with_path.ext.use_id;
        assert_eq!(x_use_id, x_def_id);
        let expr_match_arm = &expr_match.arms[0];
        let conj_use_id = expr_match_arm.pattern.type_constructor.ext.use_id;
        assert_eq!(conj_use_id, conj_def_id);
        let l_def_id = expr_match_arm.pattern.ext.ids[0].1;
        let _r_def_id = expr_match_arm.pattern.ext.ids[1].1;
        let SynExpr::App(app) = &expr_match_arm.expr else {
            panic!()
        };
        let SynExpr::IdentWithPath(ident_with_path) = &app.exprs[0] else {
            panic!()
        };
        let or_introl_use_id = ident_with_path.ext.use_id;
        assert_eq!(or_introl_use_id, or_introl_def_id);
        let SynExpr::IdentWithPath(ident_with_path) = &app.exprs[1] else {
            panic!()
        };
        let l_use_id = ident_with_path.ext.use_id;
        assert_eq!(l_use_id, l_def_id);
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

    #[test]
    fn felis_rename_uses_test_5() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or, or_introl, A, B, or_intror, A, B
        // (11) theorem1, A, B, proof, A, B, x, _, _, l, r
        assert_eq!(context.def_count(), 23);

        let mut resolver = Resolver::new();
        let prop_def_id = context.new_id();
        resolver.set("Prop".to_string(), prop_def_id);
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        path_table.setup_resolver(file_2.ext.id, &mut resolver);
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let _file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        // (3) Prop, Prop, Prop
        // (7) Prop, Prop, A, B, And, A, B
        // (3) Prop, Prop, Prop
        // (6) Prop, Prop, A, Or, A, B
        // (6) Prop, Prop, B, Or, A, B
        // (8) Prop, Prop, And A, B, Or, A, B
        // (8) Prop, Prop, And, A, B, Or, A, B
        // (1) x
        // (5) And::conj, Or::or_introl, A, B, l
        assert_eq!(context2.use_count, 47);
    }

    #[test]
    fn felis_rename_uses_test_6() {
        let s = std::fs::read_to_string("../../library/wip/entrypoint.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        // (1) [file]
        // (1) main
        assert_eq!(context.def_count(), 2);

        let mut resolver = Resolver::new();
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        path_table.setup_resolver(file_2.ext.id, &mut resolver);
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let _file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        // (1) main
        assert_eq!(context2.use_count, 1);
    }
}
