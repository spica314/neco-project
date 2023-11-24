use felis_syn::{
    decoration::{Decoration, UD},
    syn_entrypoint::SynEntrypoint,
    syn_expr::{
        SynExpr, SynExprApp, SynExprIdent, SynExprIdentWithPath, SynExprMatch, SynExprMatchArm,
        SynExprMatchPattern, SynExprParen,
    },
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::{SynFnBlock, SynFnDef},
    syn_formula::{
        SynFormula, SynFormulaApp, SynFormulaAtom, SynFormulaForall, SynFormulaImplies,
        SynFormulaParen,
    },
    syn_statement::SynStatement,
    syn_theorem_def::SynTheoremDef,
    syn_type::{
        SynType, SynTypeApp, SynTypeAtom, SynTypeDependentMap, SynTypeMap, SynTypeParen,
        SynTypeUnit,
    },
    syn_type_def::{SynTypeDef, SynVariant},
    syn_typed_arg::SynTypedArg,
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefDecoration;
impl Decoration for DefDecoration {
    // ok
    type Entrypoint = ();
    // ok
    type ExprApp = ();
    // ok
    type ExprBlock = ();
    // ok
    type ExprIdentWithPath = ();
    // ok
    type ExprIdent = ();
    // ok
    type ExprMatch = ();
    // ok
    type ExprParen = ();
    // ok
    type ExprString = ();
    // ?
    type FormulaForall = DefFormulaForall;
    // ok?
    type FormulaImplies = ();
    // ok
    type FormulaAtom = ();
    // ok
    type FormulaApp = ();
    // ok
    type FormulaParen = ();
    // ok
    type Variant = DefVariant;
    // ok
    type TypeDef = DefDecorationTypeDef;
    // ok
    type TypeApp = ();
    // ok
    type TypeAtom = ();
    // ok
    type TypeMap = ();
    // ok
    type TypeParen = ();
    // ok
    type TypeDependentMap = DefTypeDependentMap;
    // ok
    type TypeUnit = ();
    // ok
    type File = DefDecorationFile;
    // ok
    type FnDef = DefDecorationFndef;
    // ok
    type TheoremDef = DefTheoremDef;
    // ok
    type TypedArg = ();
    // ok
    type ExprMatchPattern = DefExprMatchPattern;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct DefId(usize);

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
pub struct DefDecorationFndef {
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
    Ok(SynFile {
        id: SynTreeId::new(),
        items,
        ext,
    })
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
        SynFileItem::FnDef(fn_def) => {
            let t = rename_defs_fn_def(context, fn_def)?;
            Ok(SynFileItem::FnDef(t))
        }
        SynFileItem::TheoremDef(theorem_def) => {
            let t = rename_defs_theorem_def(context, theorem_def)?;
            Ok(SynFileItem::TheoremDef(t))
        }
        SynFileItem::Entrypoint(entrypoint) => {
            let t = rename_defs_entrypoint(context, entrypoint)?;
            Ok(SynFileItem::Entrypoint(t))
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

fn rename_defs_fn_def(
    context: &mut RenameDefContext,
    fn_def: &SynFnDef<UD>,
) -> Result<SynFnDef<DefDecoration>, ()> {
    let ty = rename_defs_type(context, &fn_def.ty)?;
    let fn_block = rename_defs_fn_block(context, &fn_def.fn_block)?;
    let ext = DefDecorationFndef {
        name: fn_def.name.as_str().to_string(),
        id: context.new_id(),
    };
    Ok(SynFnDef {
        keyword_fn: fn_def.keyword_fn.clone(),
        name: fn_def.name.clone(),
        colon: fn_def.colon.clone(),
        ty,
        fn_block,
        ext,
    })
}

fn rename_defs_fn_block(
    context: &mut RenameDefContext,
    fn_block: &SynFnBlock<UD>,
) -> Result<SynFnBlock<DefDecoration>, ()> {
    let mut statements = vec![];
    for statement in &fn_block.statements {
        let statement = rename_defs_statement(context, statement)?;
        statements.push(statement);
    }
    Ok(SynFnBlock {
        lbrace: fn_block.lbrace.clone(),
        statements,
        rbrace: fn_block.rbrace.clone(),
    })
}

fn rename_defs_statement(
    context: &mut RenameDefContext,
    statement: &SynStatement<UD>,
) -> Result<SynStatement<DefDecoration>, ()> {
    match statement {
        SynStatement::Expr(expr) => {
            let expr = rename_defs_expr(context, expr)?;
            Ok(SynStatement::Expr(expr))
        }
        SynStatement::Let(_) => todo!(),
    }
}

fn rename_defs_expr(
    context: &mut RenameDefContext,
    expr: &SynExpr<UD>,
) -> Result<SynExpr<DefDecoration>, ()> {
    match expr {
        SynExpr::Ident(ident) => {
            let t = rename_defs_expr_ident(context, ident)?;
            Ok(SynExpr::Ident(t))
        }
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
        SynExpr::String(_) => todo!(),
        SynExpr::Block(_) => todo!(),
    }
}

fn rename_defs_expr_ident(
    _context: &mut RenameDefContext,
    ident: &SynExprIdent<UD>,
) -> Result<SynExprIdent<DefDecoration>, ()> {
    #[allow(clippy::let_unit_value)]
    let ext = ();
    Ok(SynExprIdent {
        id: SynTreeId::new(),
        ident: ident.ident.clone(),
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
    Ok(SynExprApp {
        id: SynTreeId::new(),
        exprs,
        ext,
    })
}

fn rename_defs_expr_paren(
    context: &mut RenameDefContext,
    paren: &SynExprParen<UD>,
) -> Result<SynExprParen<DefDecoration>, ()> {
    #[allow(clippy::let_unit_value)]
    let ext = ();
    Ok(SynExprParen {
        id: SynTreeId::new(),
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
        id: SynTreeId::new(),
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
        id: SynTreeId::new(),
        path: ident_with_path.path.clone(),
        ident: ident_with_path.ident.clone(),
        ext,
    })
}

fn rename_defs_theorem_def(
    context: &mut RenameDefContext,
    theorem_def: &SynTheoremDef<UD>,
) -> Result<SynTheoremDef<DefDecoration>, ()> {
    let formula = rename_defs_formula(context, &theorem_def.formula)?;
    let fn_def = rename_defs_fn_def(context, &theorem_def.fn_def)?;
    let ext = DefTheoremDef {
        name: theorem_def.name.as_str().to_string(),
        id: context.new_id(),
    };
    Ok(SynTheoremDef {
        keyword_theorem: theorem_def.keyword_theorem.clone(),
        name: theorem_def.name.clone(),
        eq: theorem_def.eq.clone(),
        formula,
        lbrace: theorem_def.lbrace.clone(),
        fn_def,
        rbrace: theorem_def.rbrace.clone(),
        ext,
    })
}

fn rename_defs_formula(
    context: &mut RenameDefContext,
    formula: &SynFormula<UD>,
) -> Result<SynFormula<DefDecoration>, ()> {
    match formula {
        SynFormula::Forall(forall) => {
            let child = rename_defs_formula(context, &forall.child)?;
            let ext = DefFormulaForall {
                name: forall.name.as_str().to_string(),
                id: context.new_id(),
            };
            Ok(SynFormula::Forall(SynFormulaForall {
                keyword_forall: forall.keyword_forall.clone(),
                lparen: forall.lparen.clone(),
                name: forall.name.clone(),
                colon: forall.colon.clone(),
                ty: Box::new(rename_defs_formula(context, &forall.ty)?),
                rparen: forall.rparen.clone(),
                camma: forall.camma.clone(),
                child: Box::new(child),
                ext,
            }))
        }
        SynFormula::App(app) => {
            let fun = rename_defs_formula(context, &app.fun)?;
            let arg = rename_defs_formula(context, &app.arg)?;
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynFormula::App(SynFormulaApp {
                fun: Box::new(fun),
                arg: Box::new(arg),
                ext,
            }))
        }
        SynFormula::Atom(atom) => Ok(SynFormula::Atom(SynFormulaAtom {
            ident: atom.ident.clone(),
            ext: (),
        })),
        SynFormula::Paren(paren) => {
            let child = rename_defs_formula(context, &paren.child)?;
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynFormula::Paren(SynFormulaParen {
                lparen: paren.lparen.clone(),
                child: Box::new(child),
                rparen: paren.rparen.clone(),
                ext,
            }))
        }
        SynFormula::Implies(implies) => {
            let lhs = rename_defs_formula(context, &implies.lhs)?;
            let rhs = rename_defs_formula(context, &implies.rhs)?;
            #[allow(clippy::let_unit_value)]
            let ext = ();
            Ok(SynFormula::Implies(SynFormulaImplies {
                lhs: Box::new(lhs),
                arrow: implies.arrow.clone(),
                rhs: Box::new(rhs),
                ext,
            }))
        }
    }
}

fn rename_defs_entrypoint(
    _context: &mut RenameDefContext,
    entrypoint: &SynEntrypoint<UD>,
) -> Result<SynEntrypoint<DefDecoration>, ()> {
    Ok(SynEntrypoint {
        id: SynTreeId::new(),
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
                id: SynTreeId::new(),
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
                id: SynTreeId::new(),
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
                id: SynTreeId::new(),
                lparen: unit.lparen.clone(),
                rparen: unit.rparen.clone(),
                ext,
            }))
        }
        SynType::DependentMap(dependent_map) => {
            let from = rename_defs_type_arg(context, &dependent_map.from)?;
            let to = rename_defs_type(context, &dependent_map.to)?;
            let ext = DefTypeDependentMap {
                name: from.name.ident.as_str().to_string(),
                id: context.new_id(),
            };
            Ok(SynType::DependentMap(SynTypeDependentMap {
                id: SynTreeId::new(),
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
    #[allow(clippy::let_unit_value)]
    let ext = ();
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

    #[test]
    fn felis_rename_defs_file_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fn_def.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let _file_2 = rename_defs_file(&mut context, &file).unwrap();
        // [file], proof, A, B, x, l, r
        assert_eq!(context.def_count(), 7);
    }

    #[test]
    fn felis_rename_defs_file_test_3() {
        let s = std::fs::read_to_string("../../library/wip/fn_def_simple.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let _file_2 = rename_defs_file(&mut context, &file).unwrap();
        // [file], proof, A, x
        assert_eq!(context.def_count(), 4);
    }

    #[test]
    fn felis_rename_defs_file_test_4() {
        let s = std::fs::read_to_string("../../library/wip/prop4.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let _file_2 = rename_defs_file(&mut context, &file).unwrap();
        // (1) [file]
        // (4) And, conj, A, B
        // (7) Or or_introl, A, B, or_intror, A, B
        // (11) theorem1, A, B, proof, A, B, x, _, _, l, r
        assert_eq!(context.def_count(), 23);
    }
}
