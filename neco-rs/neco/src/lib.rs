use std::{rc::Rc, str::FromStr};

use neco_core_expr::core_expr::CoreExpr;
use neco_resolver::Resolver;

#[derive(Debug, Clone)]
pub struct TypeDef {
    name: String,
    args: Vec<TypedIdent>,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedIdent {
    name: String,
    ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    name: String,
    ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Leaf(String),
    App(Rc<Type>, Vec<Type>),
    Map(Rc<Type>, Rc<Type>),
    Forall(Rc<TypedIdent>, Rc<Type>),
}

impl Type {
    pub fn is_leaf_of(&self, s: &str) -> bool {
        match self {
            Type::Leaf(t) => s == t,
            _ => false,
        }
    }
    pub fn is_head_of(&self, s: &str) -> bool {
        match self {
            Type::Leaf(t) => s == t,
            Type::App(t, _) => match t.as_ref() {
                Type::Leaf(t) => s == t,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    type_defs: Vec<TypeDef>,
    props: Vec<Prop>,
    proofs: Vec<Proof>,
}

#[derive(Debug, Clone)]
pub struct Prop {
    name: String,
    ty: Type,
}

#[derive(Debug, Clone)]
pub struct Proof {
    name: String,
    function: Function,
}

#[derive(Debug, Clone)]
pub struct Function {
    args: Vec<TypedIdent>,
    expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Match(ExprMatch),
    Atom(String),
    App(ExprApp),
}

#[derive(Debug, Clone)]
pub struct ExprApp {
    exprs: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprMatch {
    expr: Rc<Expr>,
    arms: Vec<Arm>,
}

#[derive(Debug, Clone)]
pub struct Arm {
    left: Vec<String>,
    right: Expr,
}

impl Context {
    pub fn new() -> Context {
        Context {
            type_defs: vec![],
            props: vec![],
            proofs: vec![],
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_context_module(expr: &CoreExpr, context: &mut Context) {
    assert!(expr.node_tag() == Some("module"));
    let tail = expr.tail().unwrap();
    for child in tail {
        let node_tag = child.node_tag();
        if node_tag == Some("type") {
            generate_context_type(child, context);
        } else if node_tag == Some("prop") {
            generate_context_prop(child, context);
        } else if node_tag == Some("proof") {
            generate_context_proof(child, context);
        }
    }
}

fn expr_to_typed_ident(expr: &CoreExpr) -> TypedIdent {
    let list = expr.list().unwrap();
    assert!(list[1].is_atom_of(":"));
    let name = list[0].to_string();
    let ty = expr_to_type(&list[2..]);
    TypedIdent { name, ty }
}

fn expr_to_type(exprs: &[CoreExpr]) -> Type {
    // forall
    if exprs.len() >= 2 && exprs[0].is_atom_of("forall") {
        let typed = expr_to_typed_ident(&exprs[1]);
        let inner = expr_to_type(&exprs[2..]);
        return Type::Forall(Rc::new(typed), Rc::new(inner));
    }

    let mut types = vec![];
    let mut ts = vec![];
    for expr in exprs {
        if expr.is_atom_of("->") {
            let ty = expr_to_type_sub(&ts);
            types.push(ty);
            ts.clear();
        } else {
            ts.push(expr.clone());
        }
    }
    let ty = expr_to_type_sub(&ts);
    types.push(ty);
    let mut res = types[types.len() - 1].clone();
    for ty in types.iter().rev().skip(1) {
        res = Type::Map(Rc::new(ty.clone()), Rc::new(res));
    }
    res
}

fn expr_to_type_sub(exprs: &[CoreExpr]) -> Type {
    assert!(exprs.iter().all(|expr| expr.is_atom()));
    let head = Type::Leaf(exprs[0].to_string());
    let tail: Vec<_> = exprs[1..]
        .iter()
        .map(|expr| Type::Leaf(expr.to_string()))
        .collect();
    if tail.is_empty() {
        head
    } else {
        Type::App(Rc::new(head), tail)
    }
}

fn generate_context_type(expr: &CoreExpr, context: &mut Context) {
    assert!(expr.node_tag() == Some("type"));
    let list = expr.list().unwrap();
    let name = list[1].to_string();
    let args: Vec<_> = list[2..list.len() - 1]
        .iter()
        .map(expr_to_typed_ident)
        .collect();
    let variants = expr_to_variants(&list[list.len() - 1]);
    let type_def = TypeDef {
        name,
        args,
        variants,
    };
    context.type_defs.push(type_def);
}

fn expr_to_variants(expr: &CoreExpr) -> Vec<Variant> {
    let list = expr.list().unwrap();
    list.iter().map(expr_to_variant).collect()
}

fn expr_to_variant(expr: &CoreExpr) -> Variant {
    let ty = expr_to_typed_ident(expr);
    Variant {
        name: ty.name,
        ty: ty.ty,
    }
}

fn generate_context_prop(expr: &CoreExpr, context: &mut Context) {
    assert!(expr.node_tag() == Some("prop"));
    let list = expr.list().unwrap();
    let name = list[1].to_string();
    let ty = expr_to_type(list[2].list().unwrap());
    let prop = Prop { name, ty };
    context.props.push(prop);
}

fn generate_context_proof(expr: &CoreExpr, context: &mut Context) {
    assert!(expr.node_tag() == Some("proof"));
    let list = expr.list().unwrap();
    let name = list[1].to_string();
    let function = expr_to_function_definition(&list[2]);
    let proof = Proof { name, function };
    context.proofs.push(proof);
}

fn expr_to_function_definition(expr: &CoreExpr) -> Function {
    assert!(expr.node_tag() == Some("fun"));
    let list = expr.list().unwrap();
    let args: Vec<_> = list[1..list.len() - 1]
        .iter()
        .map(expr_to_typed_ident)
        .collect();
    let expr = expr_to_expr(&list[list.len() - 1]);
    Function { args, expr }
}

fn expr_to_expr(expr: &CoreExpr) -> Expr {
    if expr.node_tag() == Some("match") {
        let expr_match = expr_to_expr_match(expr);
        return Expr::Match(expr_match);
    }
    if expr.is_atom() {
        return Expr::Atom(expr.to_string());
    }
    let list = expr.list().unwrap();
    let exprs: Vec<_> = list.iter().map(expr_to_expr).collect();
    Expr::App(ExprApp { exprs })
}

fn expr_to_expr_match(expr: &CoreExpr) -> ExprMatch {
    assert!(expr.node_tag() == Some("match"));
    let list = expr.list().unwrap();
    let expr = expr_to_expr(&list[1]);
    let arms: Vec<_> = list[2..].iter().map(expr_to_arm).collect();
    ExprMatch {
        expr: Rc::new(expr),
        arms,
    }
}

fn expr_to_arm(expr: &CoreExpr) -> Arm {
    let list = expr.list().unwrap();
    let left_expr = &list[0];
    let left: Vec<_> = left_expr
        .list()
        .unwrap()
        .iter()
        .map(|x| x.to_string())
        .collect();
    let right_expr = &list[1];
    let right = expr_to_expr(right_expr);
    Arm { left, right }
}

pub fn generate_context(expr: &CoreExpr) -> Context {
    let mut context = Context::new();
    if expr.node_tag() == Some("module") {
        generate_context_module(expr, &mut context);
    }
    context
}

impl Context {
    pub fn type_check(&self) -> bool {
        let mut var_type = Resolver::<Type>::new();
        for type_def in &self.type_defs {
            for variant in &type_def.variants {
                eprintln!("{} : {:?}", variant.name.clone(), variant.ty.clone());
                var_type
                    .record(variant.name.clone(), variant.ty.clone())
                    .unwrap();
            }
        }

        for proof in &self.proofs {
            for prop in &self.props {
                if proof.name != prop.name {
                    continue;
                }
                eprintln!("-- {}", prop.name);
                var_type.enter_scope();

                let mut delete_after_eval = vec![];
                let mut prop_ty = Rc::new(prop.ty.clone());
                for typed_ident in &proof.function.args {
                    delete_after_eval.push(typed_ident.name.clone());
                    eprintln!("{}: {:?}", typed_ident.name.clone(), typed_ident.ty.clone());
                    var_type
                        .record(typed_ident.name.clone(), typed_ident.ty.clone())
                        .unwrap();

                    let (ty_ll, ty_lr, ty_r) = match prop_ty.as_ref() {
                        Type::Forall(l, r) => {
                            (Some(l.as_ref().name.clone()), l.as_ref().ty.clone(), r)
                        }
                        Type::Map(l, r) => (None, l.as_ref().clone(), r),
                        _ => panic!(),
                    };
                    prop_ty = ty_r.clone();
                    if let Some(ty_ll) = ty_ll {
                        if ty_ll != typed_ident.name {
                            panic!();
                        }
                    }
                    if typed_ident.ty != ty_lr {
                        panic!();
                    }
                }

                let res_ty = self.expr_ty(&mut var_type, &proof.function.expr);

                var_type.leave_scope();
                if &res_ty != prop_ty.as_ref() {
                    panic!();
                }
            }
        }
        true
    }
    pub fn expr_ty(&self, var_type: &mut Resolver<Type>, expr: &Expr) -> Type {
        match expr {
            Expr::Match(expr_match) => {
                // eprintln!("expr_match = {expr_match:?}");
                let expr_ty = self.expr_ty(var_type, &expr_match.expr);
                // eprintln!("expr_ty = {expr_ty:?}");
                let mut right_types = vec![];
                // 各Typeについて
                for type_def in &self.type_defs {
                    // expr_tyが単体の型で，type_defの型の場合
                    if expr_ty.is_head_of(&type_def.name) {
                        // matchの各armについて
                        for arm in &expr_match.arms {
                            let left = &arm.left;
                            let right = &arm.right;
                            let mut delete_after_eval = vec![];
                            var_type.enter_scope();
                            // 型定義の各variantについて
                            for variant in &type_def.variants {
                                // arm左の先頭がvariant名と同じならば
                                if left[0] == variant.name {
                                    // eprintln!("variant = {variant:?}");
                                    let mut ty = Rc::new(variant.ty.clone());
                                    for v in left.iter().skip(1) {
                                        let v = v.to_string();
                                        let (ty_l, ty_r) = match ty.as_ref() {
                                            Type::Map(l, r) => (l, r),
                                            _ => panic!(),
                                        };
                                        delete_after_eval.push(v.clone());
                                        eprintln!("{} : {:?}", v.clone(), ty_l.as_ref().clone());
                                        var_type.record(v.clone(), ty_l.as_ref().clone()).unwrap();
                                        ty = ty_r.clone();
                                    }
                                    break;
                                }
                            }
                            let right = self.expr_ty(var_type, right);
                            right_types.push(right);
                            var_type.leave_scope();
                        }
                    }
                }
                eprintln!("match right-arm types: {right_types:?}");
                for i in 1..right_types.len() {
                    if right_types[0] != right_types[i] {
                        panic!();
                    }
                }
                right_types[0].clone()
            }
            Expr::Atom(expr_atom) => var_type
                .get(expr_atom)
                .unwrap_or_else(|| panic!("{expr_atom}"))
                .clone(),
            Expr::App(expr_app) => {
                let mut ty = Rc::new(self.expr_ty(var_type, &expr_app.exprs[0]));
                for arg in &expr_app.exprs[1..] {
                    let arg_ty = self.expr_ty(var_type, arg);
                    let new_ty = match ty.as_ref() {
                        Type::Leaf(_) => panic!(),
                        Type::App(_, _) => panic!(),
                        Type::Map(left, right) => {
                            if &arg_ty == left.as_ref() {
                                right
                            } else {
                                panic!()
                            }
                        }
                        Type::Forall(_, _) => todo!(),
                    };
                    ty = new_ty.clone();
                }
                ty.as_ref().clone()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CliContext {
    Compile(String),
}

pub fn run_cli(context: CliContext) {
    match context {
        CliContext::Compile(filename) => {
            let s = std::fs::read_to_string(filename).unwrap();
            let expr = CoreExpr::from_str(&s).unwrap();
            // eprintln!("expr = {expr:?}");
            let context = generate_context(&expr);
            // eprintln!("context = {context:?}");
            assert!(context.type_check());
            eprintln!("type check passed!");
        }
    }
}
