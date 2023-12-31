use felis_syn::{
    decoration::{Decoration, UD},
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecoration;
impl Decoration for DefDecoration {
    type Entrypoint = ();
    type ExprApp = ();
    type ExprIdentWithPath = ();
    type ExprNumber = DefDecorationExprNumber;
    type ExprMatch = ();
    type ExprParen = ();
    type ExprString = DefDecorationExprString;
    type Variant = DefVariant;
    type TypeDef = DefDecorationTypeDef;
    type TypeApp = ();
    type TypeAtom = ();
    type TypeMap = ();
    type TypeParen = ();
    type TypeDependentMap = ();
    type TypeUnit = ();
    type File = DefDecorationFile;
    type FnDef = DefDecorationFnDef;
    type ProcDef = DefDecorationProcDef;
    type TypedArg = DefTypedArg;
    type ExprMatchPattern = DefExprMatchPattern;
    type StatementLet = DefStatementLet;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct DefId(usize);

impl DefId {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecorationFile {
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecorationTypeDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefFormulaForall {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecorationFnDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecorationProcDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefTheoremDef {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefVariant {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefTypeDependentMap {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefExprMatchPattern {
    pub ids: Vec<(String, DefId)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefStatementLet {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefTypedArg {
    pub name: String,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecorationExprNumber {
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecorationExprString {
    pub id: DefId,
}

pub struct RenameDefContext {
    pub next_id: DefId,
}

impl RenameDefContext {
    pub fn new() -> Self {
        Self { next_id: DefId(1) }
    }

    pub fn new_id(&mut self) -> DefId {
        let res = self.next_id;
        self.next_id = DefId(self.next_id.0 + 1);
        res
    }

    pub fn def_count(&self) -> usize {
        self.next_id.0 - 1
    }
}

impl Default for RenameDefContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn rename_defs_file(
    context: &mut RenameDefContext,
    file: &SynFile<UD>,
) -> Result<SynFile<DefDecoration>, ()> {
    let mut items = vec![];
    for item in &file.items {
        let item = rename_defs_file_item(context, item)?;
        items.push(item);
    }
    let ext = DefDecorationFile {
        id: context.new_id(),
    };
    Ok(SynFile { items, ext })
}

fn rename_defs_file_item(
    context: &mut RenameDefContext,
    item: &SynFileItem<UD>,
) -> Result<SynFileItem<DefDecoration>, ()> {
    match item {
        SynFileItem::TypeDef(type_def) => {
            let t = rename_defs_type_def(context, type_def)?;
            Ok(SynFileItem::TypeDef(t))
        }
        SynFileItem::Entrypoint(entrypoint) => {
            let t = rename_defs_entrypoint(context, entrypoint)?;
            Ok(SynFileItem::Entrypoint(t))
        }
        SynFileItem::ProcDef(proc_def) => {
            let t = rename_defs_proc_def(context, proc_def)?;
            Ok(SynFileItem::ProcDef(t))
        }
    }
}

fn rename_defs_type_def(
    context: &mut RenameDefContext,
    type_def: &SynTypeDef<UD>,
) -> Result<SynTypeDef<DefDecoration>, ()> {
    let mut variants = vec![];
    for variant in &type_def.variants {
        let variant = rename_defs_variant(context, variant)?;
        variants.push(variant);
    }
    let ty_ty = rename_defs_type(context, &type_def.ty_ty)?;
    let ext = DefDecorationTypeDef {
        name: type_def.name.as_str().to_string(),
        id: context.new_id(),
    };
    Ok(SynTypeDef {
        keyword_type: type_def.keyword_type.clone(),
        name: type_def.name.clone(),
        colon: type_def.colon.clone(),
        ty_ty: Box::new(ty_ty),
        lbrace: type_def.lbrace.clone(),
        variants,
        rbrace: type_def.rbrace.clone(),
        ext,
    })
}

fn rename_defs_proc_def(
    context: &mut RenameDefContext,
    proc_def: &SynProcDef<UD>,
) -> Result<SynProcDef<DefDecoration>, ()> {
    let ty = rename_defs_type(context, &proc_def.ty)?;
    let proc_block = rename_defs_proc_block(context, &proc_def.proc_block)?;
    let ext = DefDecorationProcDef {
        name: proc_def.name.as_str().to_string(),
        id: context.new_id(),
    };
    Ok(SynProcDef {
        keyword_proc: proc_def.keyword_proc.clone(),
        name: proc_def.name.clone(),
        colon: proc_def.colon.clone(),
        ty,
        proc_block,
        ext,
    })
}

fn rename_defs_proc_block(
    context: &mut RenameDefContext,
    proc_block: &SynProcBlock<UD>,
) -> Result<SynProcBlock<DefDecoration>, ()> {
    let mut statements = vec![];
    for statement in &proc_block.statements {
        let statement = rename_defs_statement(context, statement)?;
        statements.push(statement);
    }
    Ok(SynProcBlock {
        lbrace: proc_block.lbrace.clone(),
        statements,
        rbrace: proc_block.rbrace.clone(),
    })
}

fn rename_defs_statement(
    context: &mut RenameDefContext,
    statement: &SynStatement<UD>,
) -> Result<SynStatement<DefDecoration>, ()> {
    match statement {
        SynStatement::Let(statement_let) => {
            let statement_let = rename_defs_statement_let(context, statement_let)?;
            Ok(SynStatement::Let(statement_let))
        }
        SynStatement::ExprSemi(expr_semi) => {
            let expr = rename_defs_expr(context, &expr_semi.expr)?;
            Ok(SynStatement::ExprSemi(SynStatementExprSemi {
                expr,
                semi: expr_semi.semi.clone(),
            }))
        }
        SynStatement::Assign(statement_assign) => {
            let lhs = rename_defs_expr(context, &statement_assign.lhs)?;
            let rhs = rename_defs_expr(context, &statement_assign.rhs)?;
            Ok(SynStatement::Assign(SynStatementAssign {
                lhs,
                eq: statement_assign.eq.clone(),
                rhs,
                semi: statement_assign.semi.clone(),
            }))
        }
    }
}

fn rename_defs_statement_let(
    context: &mut RenameDefContext,
    statement_let: &SynStatementLet<UD>,
) -> Result<SynStatementLet<DefDecoration>, ()> {
    let expr = rename_defs_expr(context, &statement_let.expr)?;
    let ext = DefStatementLet {
        name: statement_let.name.as_str().to_string(),
        id: context.new_id(),
    };
    Ok(SynStatementLet {
        keyword_let: statement_let.keyword_let.clone(),
        keyword_mut: statement_let.keyword_mut.clone(),
        name: statement_let.name.clone(),
        eq: statement_let.eq.clone(),
        expr,
        semi: statement_let.semi.clone(),
        ext,
    })
}

fn rename_defs_expr(
    context: &mut RenameDefContext,
    expr: &SynExpr<UD>,
) -> Result<SynExpr<DefDecoration>, ()> {
    match expr {
        SynExpr::App(app) => {
            let t = rename_defs_expr_app(context, app)?;
            Ok(SynExpr::App(t))
        }
        SynExpr::Match(expr_match) => {
            let t = rename_defs_expr_match(context, expr_match)?;
            Ok(SynExpr::Match(t))
        }
        SynExpr::Paren(paren) => {
            let t = rename_defs_expr_paren(context, paren)?;
            Ok(SynExpr::Paren(t))
        }
        SynExpr::IdentWithPath(ident_with_path) => {
            let t = rename_defs_expr_ident_with_path(context, ident_with_path)?;
            Ok(SynExpr::IdentWithPath(t))
        }
        SynExpr::String(expr_string) => {
            let t = rename_defs_expr_string(context, expr_string)?;
            Ok(SynExpr::String(t))
        }
        SynExpr::Number(expr_number) => {
            let t = rename_defs_expr_number(context, expr_number)?;
            Ok(SynExpr::Number(t))
        }
    }
}

fn rename_defs_expr_string(
    context: &mut RenameDefContext,
    expr_string: &SynExprString<UD>,
) -> Result<SynExprString<DefDecoration>, ()> {
    let id = context.new_id();
    let ext = DefDecorationExprString { id };
    Ok(SynExprString {
        token_string: expr_string.token_string.clone(),
        ext,
    })
}

fn rename_defs_expr_number(
    context: &mut RenameDefContext,
    expr_number: &SynExprNumber<UD>,
) -> Result<SynExprNumber<DefDecoration>, ()> {
    let id = context.new_id();
    let ext = DefDecorationExprNumber { id };
    Ok(SynExprNumber {
        number: expr_number.number.clone(),
        ext,
    })
}

fn rename_defs_expr_app(
    context: &mut RenameDefContext,
    app: &SynExprApp<UD>,
) -> Result<SynExprApp<DefDecoration>, ()> {
    let mut exprs = vec![];
    for expr in &app.exprs {
        let expr = rename_defs_expr(context, expr)?;
        exprs.push(expr);
    }
    #[allow(clippy::let_unit_value)]
    let ext = ();
    Ok(SynExprApp { exprs, ext })
}

fn rename_defs_expr_paren(
    context: &mut RenameDefContext,
    paren: &SynExprParen<UD>,
) -> Result<SynExprParen<DefDecoration>, ()> {
    #[allow(clippy::let_unit_value)]
    let ext = ();
    Ok(SynExprParen {
        lparen: paren.lparen.clone(),
        expr: Box::new(rename_defs_expr(context, &paren.expr)?),
        rparen: paren.rparen.clone(),
        ext,
    })
}

fn rename_defs_expr_match(
    context: &mut RenameDefContext,
    expr_match: &SynExprMatch<UD>,
) -> Result<SynExprMatch<DefDecoration>, ()> {
    let mut arms = vec![];
    for arm in &expr_match.arms {
        let arm = rename_defs_expr_arm(context, arm)?;
        arms.push(arm);
    }
    #[allow(clippy::let_unit_value)]
    let ext = ();
    Ok(SynExprMatch {
        keyword_match: expr_match.keyword_match.clone(),
        expr: Box::new(rename_defs_expr(context, &expr_match.expr)?),
        lbrace: expr_match.lbrace.clone(),
        arms,
        rbrace: expr_match.rbrace.clone(),
        ext,
    })
}

fn rename_defs_expr_arm(
    context: &mut RenameDefContext,
    arm: &SynExprMatchArm<UD>,
) -> Result<SynExprMatchArm<DefDecoration>, ()> {
    let pattern = rename_defs_expr_match_pattern(context, &arm.pattern)?;
    let expr = rename_defs_expr(context, &arm.expr)?;
    Ok(SynExprMatchArm {
        pattern,
        arrow2: arm.arrow2.clone(),
        expr,
        camma: arm.camma.clone(),
    })
}

fn rename_defs_expr_match_pattern(
    context: &mut RenameDefContext,
    pattern: &SynExprMatchPattern<UD>,
) -> Result<SynExprMatchPattern<DefDecoration>, ()> {
    let type_constructor = rename_defs_expr_ident_with_path(context, &pattern.type_constructor)?;
    let mut ids = vec![];
    for ident in &pattern.idents {
        let id = context.new_id();
        ids.push((ident.as_str().to_string(), id));
    }
    let ext = DefExprMatchPattern { ids };
    Ok(SynExprMatchPattern {
        type_constructor,
        idents: pattern.idents.clone(),
        ext,
    })
}

fn rename_defs_expr_ident_with_path(
    _context: &mut RenameDefContext,
    ident_with_path: &SynExprIdentWithPath<UD>,
) -> Result<SynExprIdentWithPath<DefDecoration>, ()> {
    #[allow(clippy::let_unit_value)]
    let ext = ();
    Ok(SynExprIdentWithPath {
        path: ident_with_path.path.clone(),
        ident: ident_with_path.ident.clone(),
        ext,
    })
}

fn rename_defs_entrypoint(
    _context: &mut RenameDefContext,
    entrypoint: &SynEntrypoint<UD>,
) -> Result<SynEntrypoint<DefDecoration>, ()> {
    Ok(SynEntrypoint {
        token_entrypoint: entrypoint.token_entrypoint.clone(),
        ident: entrypoint.ident.clone(),
        ext: (),
    })
}

fn rename_defs_variant(
    context: &mut RenameDefContext,
    variant: &SynVariant<UD>,
) -> Result<SynVariant<DefDecoration>, ()> {
    let ty = rename_defs_type(context, &variant.ty)?;
    let ext = DefVariant {
        name: variant.name.as_str().to_string(),
        id: context.new_id(),
    };
    Ok(SynVariant {
        name: variant.name.clone(),
        colon: variant.colon.clone(),
        ty,
        camma: variant.camma.clone(),
        ext,
    })
}

fn rename_defs_type(
    context: &mut RenameDefContext,
    ty: &SynType<UD>,
) -> Result<SynType<DefDecoration>, ()> {
    match ty {
        SynType::App(app) => {
            let left = rename_defs_type(context, &app.left)?;
            let right = rename_defs_type(context, &app.right)?;
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynType::App(SynTypeApp {
                left: Box::new(left),
                right: Box::new(right),
                ext,
            }))
        }
        SynType::Atom(atom) => {
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynType::Atom(SynTypeAtom {
                ident: atom.ident.clone(),
                ext,
            }))
        }
        SynType::Paren(paren) => {
            let ty = rename_defs_type(context, &paren.ty)?;
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynType::Paren(SynTypeParen {
                lparen: paren.lparen.clone(),
                ty: Box::new(ty),
                rparen: paren.rparen.clone(),
                ext,
            }))
        }
        SynType::Map(map) => {
            let from = rename_defs_type(context, &map.from)?;
            let to = rename_defs_type(context, &map.to)?;
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynType::Map(SynTypeMap {
                from: Box::new(from),
                arrow: map.arrow.clone(),
                to: Box::new(to),
                ext,
            }))
        }
        SynType::Unit(unit) => {
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynType::Unit(SynTypeUnit {
                lparen: unit.lparen.clone(),
                rparen: unit.rparen.clone(),
                ext,
            }))
        }
        SynType::DependentMap(dependent_map) => {
            let from = rename_defs_type_arg(context, &dependent_map.from)?;
            let to = rename_defs_type(context, &dependent_map.to)?;
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynType::DependentMap(SynTypeDependentMap {
                from: Box::new(from),
                arrow: dependent_map.arrow.clone(),
                to: Box::new(to),
                ext,
            }))
        }
    }
}

fn rename_defs_type_arg(
    context: &mut RenameDefContext,
    typed_arg: &SynTypedArg<UD>,
) -> Result<SynTypedArg<DefDecoration>, ()> {
    let ty = rename_defs_type(context, &typed_arg.ty)?;

    let ext = DefTypedArg {
        name: typed_arg.name.as_str().to_string(),
        id: context.new_id(),
    };

    Ok(SynTypedArg {
        lparen: typed_arg.lparen.clone(),
        name: typed_arg.name.clone(),
        colon: typed_arg.colon.clone(),
        ty,
        rparen: typed_arg.rparen.clone(),
        ext,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use felis_syn::test_utils::parse_from_str;

    #[test]
    fn felis_rename_defs_type_def_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let type_def = parse_from_str::<SynTypeDef<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let _type_def_2 = rename_defs_type_def(&mut context, &type_def).unwrap();
        // A, hoge
        assert_eq!(context.def_count(), 2);
    }

    #[test]
    fn felis_rename_defs_file_test_1() {
        let s = "#type A : Prop { hoge : A, }";
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let _file_2 = rename_defs_file(&mut context, &file).unwrap();
        // [file], A, hoge
        assert_eq!(context.def_count(), 3);
    }
}
