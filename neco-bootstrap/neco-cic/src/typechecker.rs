use std::rc::Rc;

use crate::{
    global_environment::GlobalEnvironment,
    local_context::LocalContext,
    reduction::{normalize_with_env, whnf_with_env},
    term::{
        Sort, Term, TermApplication, TermConstant, TermLambda, TermLetIn, TermMatch, TermProduct,
        TermSort, TermVariable,
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
    UnknownInductive(String),
    UnknownConstructor(String),
    InvalidConstructor(String),
    InvalidCase(String),
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::UnboundVariable(msg) => write!(f, "Unbound variable: {msg}"),
            TypeError::UnboundConstant(msg) => write!(f, "Unbound constant: {msg}"),
            TypeError::NotAFunction(msg) => write!(f, "Not a function: {msg}"),
            TypeError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {expected}, found {found}")
            }
            TypeError::NotAType(msg) => write!(f, "Not a type: {msg}"),
            TypeError::InvalidApplication(msg) => write!(f, "Invalid application: {msg}"),
            TypeError::InvalidProductSort(msg) => write!(f, "Invalid product sort: {msg}"),
            TypeError::UniverseInconsistency(msg) => write!(f, "Universe inconsistency: {msg}"),
            TypeError::UnknownInductive(msg) => write!(f, "Unknown inductive type: {msg}"),
            TypeError::UnknownConstructor(msg) => write!(f, "Unknown constructor: {msg}"),
            TypeError::InvalidConstructor(msg) => write!(f, "Invalid constructor: {msg}"),
            TypeError::InvalidCase(msg) => write!(f, "Invalid case expression: {msg}"),
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
        Term::Match(case) => infer_case_type(ctx, env, case),
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
            expected: format!("{expected_type:?}"),
            found: format!("{inferred_type:?}"),
        });
    }
    Ok(())
}

/// Checks if two terms are convertible (definitionally equal)
pub fn is_convertible(
    _ctx: &LocalContext,
    env: &GlobalEnvironment,
    term1: &Term,
    term2: &Term,
) -> bool {
    // Use normalization with environment to handle δ-reduction
    let norm1 = normalize_with_env(term1, env);
    let norm2 = normalize_with_env(term2, env);
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
fn infer_constant_type(env: &GlobalEnvironment, const_: &TermConstant) -> TypeResult {
    // First check if it's a regular constant
    if let Some(const_def) = env.get_constant(const_.id) {
        return Ok(const_def.ty.clone());
    }

    // Then check if it's an inductive type
    if let Some(inductive_def) = env.inductives.get_inductive(const_.id) {
        return Ok(inductive_def.get_type());
    }

    // Finally check if it's a constructor
    if let Some(constructor_def) = env.inductives.get_constructor(const_.id) {
        return Ok(constructor_def.ty.clone());
    }

    Err(TypeError::UnboundConstant(format!("{:?}", const_.id)))
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
    let source_sort = ensure_sort_with_env(&source_type, env)?;

    // Check the target in the extended context
    let extended_ctx = ctx.with(product.var, product.source.clone());
    let target_type = infer_type(&extended_ctx, env, &product.target)?;
    let target_sort = ensure_sort_with_env(&target_type, env)?;

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
    ensure_sort_with_env(&source_type, env)?;

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
        let fun_whnf = whnf_with_env(&result_type, env);

        match &fun_whnf {
            Term::Product(product) => {
                // Check that the argument has the expected type
                check_type(ctx, env, arg, &product.source)?;

                // Substitute the argument in the return type
                let mut subst = crate::substitution::Substitution::new();
                subst.add(product.var, Rc::new(arg.clone()));
                let substituted_target = crate::substitution::substitute(&product.target, &subst);
                result_type = Rc::new(substituted_target);
            }
            _ => {
                return Err(TypeError::NotAFunction(format!(
                    "Expected function type, got {fun_whnf:?}"
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
    ensure_sort_with_env(&ty_type, env)?;

    // Infer the type of the body in the extended context
    let extended_ctx = ctx.with(let_in.var, let_in.ty.clone());
    let body_type = infer_type(&extended_ctx, env, &let_in.body)?;

    // Substitute the term in the body type
    let mut subst = crate::substitution::Substitution::new();
    subst.add(let_in.var, let_in.term.clone());
    Ok(Rc::new(crate::substitution::substitute(&body_type, &subst)))
}

/// Ensures that a term is a sort and returns it (with environment)
fn ensure_sort_with_env(term: &Term, env: &GlobalEnvironment) -> Result<Sort, TypeError> {
    let whnf_term = whnf_with_env(term, env);
    match &whnf_term {
        Term::Sort(sort) => Ok(sort.sort.clone()),
        _ => Err(TypeError::NotAType(format!("{term:?}"))),
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

/// Infers the type of a case expression
/// This implements the dependent case analysis rule
fn infer_case_type(ctx: &LocalContext, env: &GlobalEnvironment, case: &TermMatch) -> TypeResult {
    // Infer the type of the scrutinee
    let scrutinee_type = infer_type(ctx, env, &case.scrutinee)?;

    // The scrutinee should have an inductive type
    let inductive_id = match scrutinee_type.as_ref() {
        Term::Constant(const_) => {
            // Check that this is indeed an inductive type
            if env.inductives.get_inductive(const_.id).is_some() {
                const_.id
            } else {
                return Err(TypeError::InvalidCase(format!(
                    "Case scrutinee type {:?} is not an inductive type",
                    const_.id
                )));
            }
        }
        _ => {
            return Err(TypeError::InvalidCase(format!(
                "Case scrutinee must have inductive type, got {scrutinee_type:?}"
            )));
        }
    };

    let inductive_def = env.inductives.get_inductive(inductive_id).unwrap();

    // Check that we have the right number of branches
    if case.branches.len() != inductive_def.constructor_count() {
        return Err(TypeError::InvalidCase(format!(
            "Case has {} branches but inductive type has {} constructors",
            case.branches.len(),
            inductive_def.constructor_count()
        )));
    }

    // Type check each branch
    for branch in &case.branches {
        let constructor_def = inductive_def
            .find_constructor(branch.constructor_id)
            .ok_or_else(|| TypeError::UnknownConstructor(format!("{:?}", branch.constructor_id)))?;

        // Check that the number of bound variables matches the constructor arity
        if branch.bound_vars.len() != constructor_def.arity {
            return Err(TypeError::InvalidCase(format!(
                "Branch for constructor {:?} binds {} variables but constructor has arity {}",
                branch.constructor_id,
                branch.bound_vars.len(),
                constructor_def.arity
            )));
        }

        // TODO: Add the bound variables to the context with their proper types
        // and type check the branch body
        let _branch_type = infer_type(ctx, env, &branch.body)?;

        // TODO: Check that all branches have compatible types with the return type
    }

    // For now, return the declared return type
    // TODO: Apply proper substitutions
    Ok(case.return_type.clone())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        global_environment::GlobalEnvironment,
        id::Id,
        local_context::LocalContext,
        term::{Sort, Term, TermApplication, TermLambda, TermProduct, TermSort, TermVariable},
    };

    use super::{TypeError, check_type, infer_type};

    #[test]
    fn test_set_has_type_type0() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();

        let set = Term::Sort(TermSort { sort: Sort::Set });
        let ty = infer_type(&ctx, &env, &set).unwrap();
        assert_eq!(
            *ty,
            Term::Sort(TermSort {
                sort: Sort::Type(0)
            })
        );
    }

    #[test]
    fn test_type0_has_type_type1() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();

        let type0 = Term::Sort(TermSort {
            sort: Sort::Type(0),
        });
        let ty = infer_type(&ctx, &env, &type0).unwrap();
        assert_eq!(
            *ty,
            Term::Sort(TermSort {
                sort: Sort::Type(1)
            })
        );
    }

    #[test]
    fn test_prop_has_type_type0() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();

        let prop = Term::Sort(TermSort { sort: Sort::Prop });
        let ty = infer_type(&ctx, &env, &prop).unwrap();
        assert_eq!(
            *ty,
            Term::Sort(TermSort {
                sort: Sort::Type(0)
            })
        );
    }

    #[test]
    fn test_unbound_variable_fails() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let x = Id::new();

        let var = Term::Variable(TermVariable { id: x });
        assert!(matches!(
            infer_type(&ctx, &env, &var),
            Err(TypeError::UnboundVariable(_))
        ));
    }

    #[test]
    fn test_bound_variable_succeeds() {
        let mut ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let x = Id::new();

        let set = Rc::new(Term::Sort(TermSort { sort: Sort::Set }));
        ctx.extend(x, set.clone()).unwrap();

        let var = Term::Variable(TermVariable { id: x });
        let ty = infer_type(&ctx, &env, &var).unwrap();
        assert_eq!(ty, set);
    }

    #[test]
    fn test_lambda_identity_function_typing() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let x = Id::new();

        let set = Term::Sort(TermSort { sort: Sort::Set });
        let lambda = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(set.clone()),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        let ty = infer_type(&ctx, &env, &lambda).unwrap();
        let expected = Term::Product(TermProduct {
            var: x,
            source: Rc::new(set.clone()),
            target: Rc::new(set),
        });
        assert_eq!(*ty, expected);
    }

    #[test]
    fn test_application_typing() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let x = Id::new();
        let y = Id::new();

        // Create: (λx:Set. x) : Set → Set
        let set = Term::Sort(TermSort { sort: Sort::Set });
        let lambda = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(set.clone()),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        // Apply to a variable of type Set
        let mut ctx_with_y = ctx.clone();
        ctx_with_y.extend(y, Rc::new(set.clone())).unwrap();

        let app = Term::Application(TermApplication {
            f: Rc::new(lambda),
            args: vec![Term::Variable(TermVariable { id: y })],
        });

        let ty = infer_type(&ctx_with_y, &env, &app).unwrap();
        assert_eq!(*ty, set);
    }

    #[test]
    fn test_simple_product_typing() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let x = Id::new();

        let set = Term::Sort(TermSort { sort: Sort::Set });
        let product = Term::Product(TermProduct {
            var: x,
            source: Rc::new(set.clone()),
            target: Rc::new(set),
        });

        let ty = infer_type(&ctx, &env, &product).unwrap();
        assert_eq!(
            *ty,
            Term::Sort(TermSort {
                sort: Sort::Type(0)
            })
        );
    }

    #[test]
    fn test_type_checking_succeeds_with_correct_type() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let x = Id::new();

        // Create identity function: λx:Set. x
        let set = Term::Sort(TermSort { sort: Sort::Set });
        let id_fun = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(set.clone()),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        // Expected type: Set → Set
        let expected_type = Term::Product(TermProduct {
            var: x,
            source: Rc::new(set.clone()),
            target: Rc::new(set.clone()),
        });

        assert!(check_type(&ctx, &env, &id_fun, &expected_type).is_ok());
    }

    #[test]
    fn test_type_checking_fails_with_wrong_type() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let x = Id::new();

        // Create identity function: λx:Set. x
        let set = Term::Sort(TermSort { sort: Sort::Set });
        let id_fun = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(set.clone()),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        // Wrong type should fail
        assert!(check_type(&ctx, &env, &id_fun, &set).is_err());
    }

    #[test]
    fn test_dependent_product_type_inference() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment::new();
        let a = Id::new();
        let x = Id::new();

        // Test: Πa:Type(0). Πx:a. a : Type(1)
        let type0 = Term::Sort(TermSort {
            sort: Sort::Type(0),
        });
        let var_a = Term::Variable(TermVariable { id: a });

        let inner_product = Term::Product(TermProduct {
            var: x,
            source: Rc::new(var_a.clone()),
            target: Rc::new(var_a),
        });

        let outer_product = Term::Product(TermProduct {
            var: a,
            source: Rc::new(type0.clone()),
            target: Rc::new(inner_product),
        });

        let ty = infer_type(&ctx, &env, &outer_product).unwrap();
        assert_eq!(
            *ty,
            Term::Sort(TermSort {
                sort: Sort::Type(1)
            })
        );
    }
}
