use std::collections::HashMap;

use felis_syn::{
    syn_expr::{SynExpr, SynExprMatch},
    syn_file::{SynFile, SynFileItem},
    syn_fn_def::{SynFnDef, SynStatement},
    syn_theorem_def::SynTheoremDef,
    syn_type::SynType,
    syn_type_def::SynTypeDef,
    to_felis_string::ToFelisString,
};
use neco_resolver::Resolver;

#[derive(Debug, Clone)]
struct Context<'a> {
    types: HashMap<String, &'a SynTypeDef>,
    theorems: HashMap<String, &'a SynTheoremDef>,
    type_resolver: Resolver<SynType>,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context::<'a> {
            types: HashMap::new(),
            theorems: HashMap::new(),
            type_resolver: Resolver::new(),
        }
    }
    pub fn get_type_def(&self, s: &str) -> Option<&'a SynTypeDef> {
        self.types.get(s).copied()
    }
}

pub fn felis_type_check(file: &SynFile) -> Result<SynFile, ()> {
    let mut context = Context::new();

    for item in &file.items {
        match item {
            SynFileItem::TypeDef(type_def) => {
                context
                    .types
                    .insert(type_def.name.ident.as_str().to_string(), type_def);
            }
            SynFileItem::FnDef(_) => todo!(),
            SynFileItem::TheoremDef(theorem_def) => {
                context
                    .theorems
                    .insert(theorem_def.name.ident.as_str().to_string(), theorem_def);
            }
        }
    }

    for type_def in context.types.values() {
        for variant in &type_def.variants {
            context
                .type_resolver
                .set(variant.name.ident.as_str().to_string(), variant.ty.clone())
                .ok();
        }
    }

    let mut items = vec![];
    for item in &file.items {
        match item {
            SynFileItem::TypeDef(type_def) => {
                items.push(SynFileItem::TypeDef(type_def.clone()));
            }
            SynFileItem::FnDef(fn_def) => {
                let t = felis_fn_type_check(&mut context, fn_def)?;
                items.push(SynFileItem::FnDef(t));
            }
            SynFileItem::TheoremDef(theorem_def) => {
                let t = felis_theorem_def_type_check(&mut context, theorem_def)?;
                items.push(SynFileItem::TheoremDef(t));
            }
        }
    }
    Ok(SynFile { items })
}

fn felis_theorem_def_type_check(
    context: &mut Context,
    theorem_def: &SynTheoremDef,
) -> Result<SynTheoremDef, ()> {
    let mut res = theorem_def.clone();
    let fn_def = felis_fn_type_check(context, &theorem_def.fn_def)?;
    res.fn_def = fn_def;
    Ok(res)
}

fn felis_fn_type_check(context: &mut Context, fn_def: &SynFnDef) -> Result<SynFnDef, ()> {
    context.type_resolver.enter_scope();
    for typed_arg in &fn_def.args {
        context
            .type_resolver
            .set(
                typed_arg.name.ident.as_str().to_string(),
                typed_arg.ty.clone(),
            )
            .ok();
    }

    assert_eq!(fn_def.fn_block.statements.len(), 1);
    let SynStatement::Expr(expr) = &fn_def.fn_block.statements[0];
    let t = felis_expr_type_check(context, expr, &fn_def.res_ty)?;
    let mut res = fn_def.clone();
    res.fn_block.statements[0] = SynStatement::Expr(t);

    context.type_resolver.leave_scope();
    Ok(res)
}

fn felis_expr_type_check(
    context: &mut Context,
    expr: &SynExpr,
    expected_ret_ty: &SynType,
) -> Result<SynExpr, ()> {
    match expr {
        SynExpr::Ident(_) => todo!(),
        SynExpr::App(app) => {
            let exprs = &app.exprs;
            let v_name = expr_to_ident_name(&exprs[0]).unwrap();
            let mut ty = context.type_resolver.get(&v_name).unwrap();
            let mut types = vec![];
            for expr in exprs {
                let v_name = expr_to_ident_name(expr).unwrap();
                let ty = context.type_resolver.get(&v_name).unwrap();
                types.push(ty);
            }
            eprintln!("types = {types:?}");
            for ty2 in types.iter().skip(1) {
                match ty {
                    SynType::Map(map) => {
                        let from = &map.from;
                        let to = &map.to;
                        eprintln!("from = {from:?}");
                        eprintln!("to   = {to:?}");
                        eprintln!("ty2  = {ty2:?}");
                        if from.as_ref().to_felis_string() == ty2.to_felis_string() {
                            ty = to.as_ref();
                        } else {
                            panic!();
                        }
                    }
                    _ => panic!(),
                }
            }
            eprintln!("ty = {ty:?}");
            if ty.to_felis_string() == expected_ret_ty.to_felis_string() {
                Ok(expr.clone())
            } else {
                panic!()
            }
        }
        SynExpr::Match(expr_match) => {
            let t = felis_expr_match_type_check(context, expr_match, expected_ret_ty);
            t.map(SynExpr::Match)
        }
        SynExpr::Paren(_) => todo!(),
    }
}

// temp
fn expr_to_ident_name(expr: &SynExpr) -> Option<String> {
    match expr {
        SynExpr::Ident(ident) => Some(ident.ident.ident.as_str().to_string()),
        SynExpr::App(_) => todo!(),
        SynExpr::Match(_) => todo!(),
        SynExpr::Paren(paren) => todo!("{:?}", paren),
    }
}

// temp
fn type_most_left_name(ty: &SynType) -> Option<String> {
    match ty {
        SynType::Forall(_) => todo!(),
        SynType::App(app) => type_most_left_name(&app.left),
        SynType::Atom(atom) => Some(atom.ident.ident.as_str().to_string()),
        SynType::Map(_) => todo!(),
        SynType::Paren(_) => todo!(),
    }
}

fn type_flatten_map(ty: &SynType) -> Option<Vec<SynType>> {
    match ty {
        SynType::Forall(_) => todo!(),
        SynType::App(_) | SynType::Atom(_) | SynType::Paren(_) => Some(vec![ty.clone()]),
        SynType::Map(ty_map) => {
            let mut xs = vec![];
            xs.push(ty_map.from.as_ref().clone());
            let ys = type_flatten_map(&ty_map.to).unwrap();
            for y in ys {
                xs.push(y);
            }
            Some(xs)
        }
    }
}

fn felis_expr_match_type_check(
    context: &mut Context,
    expr_match: &SynExprMatch,
    expected_ret_ty: &SynType,
) -> Result<SynExprMatch, ()> {
    eprintln!("check expr_match");
    eprintln!("expr_match = {expr_match:?}");
    eprintln!("expected_res_ty = {expected_ret_ty:?}");
    eprintln!();
    let expr_ident = expr_to_ident_name(&expr_match.expr).unwrap();
    let expr_ty = context.type_resolver.get(&expr_ident).unwrap();
    eprintln!("expr_ty = {expr_ty:?}");
    let left = type_most_left_name(expr_ty).unwrap();
    let type_def = context.get_type_def(&left).unwrap();
    eprintln!("type_def = {type_def:?}");
    {
        let mut variant_names = vec![];
        let mut arm_names = vec![];
        for variant in &type_def.variants {
            let t = variant.name.ident.as_str().to_string();
            variant_names.push(t);
        }
        for arm in &expr_match.arms {
            let t = arm.pattern.idents[0].ident.as_str().to_string();
            arm_names.push(t);
        }
        variant_names.sort();
        arm_names.sort();
        if variant_names != arm_names {
            panic!();
        }
    }
    for arm in &expr_match.arms {
        context.type_resolver.enter_scope();
        let variant = 'b: {
            for variant in &type_def.variants {
                if arm.pattern.idents[0].ident.as_str() == variant.name.ident.as_str() {
                    break 'b variant;
                }
            }
            panic!();
        };
        let variant_types = type_flatten_map(&variant.ty).unwrap();
        for i in 1..arm.pattern.idents.len() {
            let name = arm.pattern.idents[i].ident.as_str().to_string();
            let ty = variant_types[i - 1].clone();
            eprintln!("name = {name}, ty = {ty:?}");
            context.type_resolver.set(name, ty).ok();
        }
        let expr = &arm.expr;
        eprintln!("expr = {expr:?}");
        let _typed = felis_expr_type_check(context, expr, expected_ret_ty).unwrap();
        context.type_resolver.leave_scope();
    }
    Ok(expr_match.clone())
}

#[cfg(test)]
mod test {
    use felis_syn::test_utils::parse_from_str;

    use super::*;

    #[test]
    fn felis_type_check_test_1() {
        let s = std::fs::read_to_string("../../library/wip/prop2.fe").unwrap();
        let res = parse_from_str::<SynFile>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        let _res_typed = felis_type_check(&res).ok();
    }

    #[test]
    #[should_panic]
    fn felis_type_check_test_2() {
        let s = std::fs::read_to_string("../../library/wip/fail_prop.fe").unwrap();
        let res = parse_from_str::<SynFile>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        let _res_typed = felis_type_check(&res).ok();
    }

    #[test]
    #[should_panic]
    fn felis_type_check_test_3() {
        let s = std::fs::read_to_string("../../library/wip/fail_prop2.fe").unwrap();
        let res = parse_from_str::<SynFile>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        let _res_typed = felis_type_check(&res).ok();
    }

    // #[test]
    fn felis_type_check_test_4() {
        let s = std::fs::read_to_string("../../library/wip/prop3.fe").unwrap();
        let res = parse_from_str::<SynFile>(&s);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        let _res_typed = felis_type_check(&res).ok();
    }
}
