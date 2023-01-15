use std::rc::Rc;

use neco_core_expr::CoreExpr;

#[derive(Debug, Clone)]
pub struct TypeDef {
    name: String,
    args: Vec<TypedIdent>,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct TypedIdent {
    name: String,
    ty: Type,
}

#[derive(Debug, Clone)]
pub struct Variant {
    name: String,
    ty: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
    Leaf(String),
    App(Rc<Type>, Vec<Type>),
    Map(Rc<Type>, Rc<Type>),
}

#[derive(Debug, Clone)]
pub struct Context {
    type_defs: Vec<TypeDef>,
    props: Vec<Prop>,
    proofs: Vec<Proof>,
}

#[derive(Debug, Clone)]
pub struct Prop {}

#[derive(Debug, Clone)]
pub struct Proof {}

impl Context {
    pub fn new() -> Context {
        Context {
            type_defs: vec![],
            props: vec![],
            proofs: vec![],
        }
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
}

fn generate_context_proof(expr: &CoreExpr, context: &mut Context) {
    assert!(expr.node_tag() == Some("proof"));
}

pub fn generate_context(expr: &CoreExpr) -> Context {
    let mut context = Context::new();
    if expr.node_tag() == Some("module") {
        generate_context_module(&expr, &mut context);
    }
    context
}
