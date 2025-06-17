use std::rc::Rc;

use crate::{
    global_environment::GlobalEnvironment,
    local_context::LocalContext,
    reduction::{normalize, whnf},
    term::{
        Sort, Term, TermApplication, TermConstant, TermLambda, TermLetIn, TermProduct, TermSort,
        TermVariable,
    },
};

/// Type checking errors
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnboundVariable(String),
    UnboundConstant(String),
    NotAFunction(String),
    TypeMismatch { expected: String, found: String },
    NotAType(String),
    InvalidApplication(String),
    InvalidProductSort(String),
    UniverseInconsistency(String),
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::UnboundVariable(msg) => write!(f, "Unbound variable: {}", msg),
            TypeError::UnboundConstant(msg) => write!(f, "Unbound constant: {}", msg),
            TypeError::NotAFunction(msg) => write!(f, "Not a function: {}", msg),
            TypeError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            TypeError::NotAType(msg) => write!(f, "Not a type: {}", msg),
            TypeError::InvalidApplication(msg) => write!(f, "Invalid application: {}", msg),
            TypeError::InvalidProductSort(msg) => write!(f, "Invalid product sort: {}", msg),
            TypeError::UniverseInconsistency(msg) => write!(f, "Universe inconsistency: {}", msg),
        }
    }
}

impl std::error::Error for TypeError {}

type TypeResult = Result<Rc<Term>, TypeError>;

/// Infers the type of a term in the given context
/// Implements the typing rules of CIC
pub fn infer_type(ctx: &LocalContext, env: &GlobalEnvironment, term: &Term) -> TypeResult {
    match term {
        Term::Sort(sort) => infer_sort_type(sort),
        Term::Variable(var) => infer_variable_type(ctx, var),
        Term::Constant(const_) => infer_constant_type(env, const_),
        Term::Product(product) => infer_product_type(ctx, env, product),
        Term::Lambda(lambda) => infer_lambda_type(ctx, env, lambda),
        Term::Application(app) => infer_application_type(ctx, env, app),
        Term::LetIn(let_in) => infer_let_in_type(ctx, env, let_in),
    }
}

/// Type checking: verify that a term has a given type
pub fn check_type(
    ctx: &LocalContext,
    env: &GlobalEnvironment,
    term: &Term,
    expected_type: &Term,
) -> Result<(), TypeError> {
    let inferred_type = infer_type(ctx, env, term)?;
    if !is_convertible(ctx, env, &inferred_type, expected_type) {
        return Err(TypeError::TypeMismatch {
            expected: format!("{:?}", expected_type),
            found: format!("{:?}", inferred_type),
        });
    }
    Ok(())
}

/// Checks if two terms are convertible (definitionally equal)
pub fn is_convertible(
    _ctx: &LocalContext,
    _env: &GlobalEnvironment,
    term1: &Term,
    term2: &Term,
) -> bool {
    // For now, we use syntactic equality after normalization
    // TODO: Implement proper conversion checking with η-conversion
    let norm1 = normalize(term1);
    let norm2 = normalize(term2);
    norm1 == norm2
}

/// Infers the type of a sort
/// Set : Type(0), Prop : Type(0), Type(i) : Type(i+1)
fn infer_sort_type(sort: &TermSort) -> TypeResult {
    match &sort.sort {
        Sort::Set => Ok(Rc::new(Term::Sort(TermSort {
            sort: Sort::Type(0),
        }))),
        Sort::Prop => Ok(Rc::new(Term::Sort(TermSort {
            sort: Sort::Type(0),
        }))),
        Sort::Type(level) => Ok(Rc::new(Term::Sort(TermSort {
            sort: Sort::Type(level + 1),
        }))),
    }
}

/// Infers the type of a variable by looking it up in the context
fn infer_variable_type(ctx: &LocalContext, var: &TermVariable) -> TypeResult {
    ctx.lookup(var.id)
        .ok_or_else(|| TypeError::UnboundVariable(format!("{:?}", var.id)))
}

/// Infers the type of a constant by looking it up in the global environment
fn infer_constant_type(_env: &GlobalEnvironment, _const: &TermConstant) -> TypeResult {
    // TODO: Implement when GlobalEnvironment is properly defined
    Err(TypeError::UnboundConstant(format!(
        "Global environment not yet implemented"
    )))
}

/// Infers the type of a product (Π-type)
/// Γ ⊢ A : s₁  Γ, x:A ⊢ B : s₂
/// --------------------------------
/// Γ ⊢ Πx:A.B : sort_rule(s₁, s₂)
fn infer_product_type(
    ctx: &LocalContext,
    env: &GlobalEnvironment,
    product: &TermProduct,
) -> TypeResult {
    // Check that the source type is a valid type
    let source_type = infer_type(ctx, env, &product.source)?;
    let source_sort = ensure_sort(&source_type)?;

    // Check the target in the extended context
    let extended_ctx = ctx.with(product.var, product.source.clone());
    let target_type = infer_type(&extended_ctx, env, &product.target)?;
    let target_sort = ensure_sort(&target_type)?;

    // Apply sort rule
    let result_sort = sort_rule(&source_sort, &target_sort)?;
    Ok(Rc::new(Term::Sort(TermSort { sort: result_sort })))
}

/// Infers the type of a lambda abstraction
/// Γ, x:A ⊢ t : B  Γ ⊢ Πx:A.B : s
/// --------------------------------
/// Γ ⊢ λx:A.t : Πx:A.B
fn infer_lambda_type(
    ctx: &LocalContext,
    env: &GlobalEnvironment,
    lambda: &TermLambda,
) -> TypeResult {
    // Check that the source type is valid
    let source_type = infer_type(ctx, env, &lambda.source_ty)?;
    ensure_sort(&source_type)?;

    // Infer the type of the body in the extended context
    let extended_ctx = ctx.with(lambda.var, lambda.source_ty.clone());
    let body_type = infer_type(&extended_ctx, env, &lambda.target)?;

    // The type of the lambda is a product type
    let product_type = Term::Product(TermProduct {
        var: lambda.var,
        source: lambda.source_ty.clone(),
        target: body_type,
    });

    // Check that the product type is well-formed
    let _ = infer_type(ctx, env, &product_type)?;

    Ok(Rc::new(product_type))
}

/// Infers the type of an application
/// Γ ⊢ f : Πx:A.B  Γ ⊢ a : A
/// --------------------------
/// Γ ⊢ f a : B[x := a]
fn infer_application_type(
    ctx: &LocalContext,
    env: &GlobalEnvironment,
    app: &TermApplication,
) -> TypeResult {
    // Infer the type of the function
    let fun_type = infer_type(ctx, env, &app.f)?;

    // Apply arguments one by one
    let mut result_type = fun_type;
    for arg in &app.args {
        // Reduce to WHNF to expose the product type
        let fun_whnf = whnf(&result_type);

        match &fun_whnf {
            Term::Product(product) => {
                // Check that the argument has the expected type
                check_type(ctx, env, arg, &product.source)?;

                // Substitute the argument in the return type
                let mut subst = crate::substitution::Substitution::new();
                subst.add(product.var, Rc::new(arg.clone()));
                result_type = Rc::new(crate::substitution::substitute(&product.target, &subst));
            }
            _ => {
                return Err(TypeError::NotAFunction(format!(
                    "Expected function type, got {:?}",
                    fun_whnf
                )));
            }
        }
    }

    Ok(result_type)
}

/// Infers the type of a let-in expression
/// Γ ⊢ t : A  Γ, x:A ⊢ u : B
/// --------------------------
/// Γ ⊢ let x:A = t in u : B[x := t]
fn infer_let_in_type(
    ctx: &LocalContext,
    env: &GlobalEnvironment,
    let_in: &TermLetIn,
) -> TypeResult {
    // Check that the term has the declared type
    check_type(ctx, env, &let_in.term, &let_in.ty)?;

    // Check that the type is valid
    let ty_type = infer_type(ctx, env, &let_in.ty)?;
    ensure_sort(&ty_type)?;

    // Infer the type of the body in the extended context
    let extended_ctx = ctx.with(let_in.var, let_in.ty.clone());
    let body_type = infer_type(&extended_ctx, env, &let_in.body)?;

    // Substitute the term in the body type
    let mut subst = crate::substitution::Substitution::new();
    subst.add(let_in.var, let_in.term.clone());
    Ok(Rc::new(crate::substitution::substitute(&body_type, &subst)))
}

/// Ensures that a term is a sort and returns it
fn ensure_sort(term: &Term) -> Result<Sort, TypeError> {
    let whnf_term = whnf(term);
    match &whnf_term {
        Term::Sort(sort) => Ok(sort.sort.clone()),
        _ => Err(TypeError::NotAType(format!("{:?}", term))),
    }
}

/// Sort rule for CIC
/// This implements the allowed product formations
fn sort_rule(s1: &Sort, s2: &Sort) -> Result<Sort, TypeError> {
    match (s1, s2) {
        // (Set, Set) → Set
        (Sort::Set, Sort::Set) => Ok(Sort::Set),
        // (Set, Prop) → Prop
        (Sort::Set, Sort::Prop) => Ok(Sort::Prop),
        // (Prop, Prop) → Prop
        (Sort::Prop, Sort::Prop) => Ok(Sort::Prop),
        // (Prop, Set) → Set
        (Sort::Prop, Sort::Set) => Ok(Sort::Set),

        // Rules involving Type
        (Sort::Type(i), Sort::Type(j)) => Ok(Sort::Type(*i.max(j))),
        (Sort::Type(_), Sort::Set) => Ok(Sort::Set),
        (Sort::Type(_), Sort::Prop) => Ok(Sort::Prop),
        (Sort::Set, Sort::Type(j)) => Ok(Sort::Type(*j)),
        (Sort::Prop, Sort::Type(j)) => Ok(Sort::Type(*j)),
    }
}
