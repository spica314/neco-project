use std::rc::Rc;

use crate::{
    substitution::{Substitution, substitute},
    term::{Term, TermApplication, TermFix, TermLambda, TermLetIn, TermMatch, TermProduct},
};

/// Performs one step of reduction on a term.
/// Returns None if the term is already in normal form.
pub fn reduce_step(term: &Term) -> Option<Term> {
    match term {
        Term::Sort(_) => None,
        Term::Variable(_) => None,
        Term::Constant(_) => None,
        Term::Product(product) => reduce_product(product),
        Term::Lambda(lambda) => reduce_lambda(lambda),
        Term::Application(app) => reduce_application(app),
        Term::LetIn(let_in) => reduce_let_in(let_in),
        Term::Match(case) => reduce_case(case),
        Term::Fix(fix) => reduce_fix(fix),
    }
}

/// Reduces a term to its normal form.
pub fn normalize(term: &Term) -> Term {
    let mut current = term.clone();
    while let Some(reduced) = reduce_step(&current) {
        current = reduced;
    }
    current
}

fn reduce_product(product: &TermProduct) -> Option<Term> {
    // Try to reduce the source type
    if let Some(source) = reduce_step(&product.source) {
        return Some(Term::Product(TermProduct {
            var: product.var,
            source: Rc::new(source),
            target: product.target.clone(),
        }));
    }

    // Try to reduce the target type
    if let Some(target) = reduce_step(&product.target) {
        return Some(Term::Product(TermProduct {
            var: product.var,
            source: product.source.clone(),
            target: Rc::new(target),
        }));
    }

    None
}

fn reduce_lambda(lambda: &TermLambda) -> Option<Term> {
    // Try to reduce the source type
    if let Some(source_ty) = reduce_step(&lambda.source_ty) {
        return Some(Term::Lambda(TermLambda {
            var: lambda.var,
            source_ty: Rc::new(source_ty),
            target: lambda.target.clone(),
        }));
    }

    // Try to reduce the body
    if let Some(target) = reduce_step(&lambda.target) {
        return Some(Term::Lambda(TermLambda {
            var: lambda.var,
            source_ty: lambda.source_ty.clone(),
            target: Rc::new(target),
        }));
    }

    None
}

fn reduce_application(app: &TermApplication) -> Option<Term> {
    // β-reduction: (λx. t) u → t[x := u]
    if let Term::Lambda(lambda) = app.f.as_ref() {
        if !app.args.is_empty() {
            // Apply the first argument
            let mut subst = Substitution::new();
            subst.add(lambda.var, Rc::new(app.args[0].clone()));
            let reduced_body = substitute(&lambda.target, &subst);

            // If there are more arguments, create a new application
            if app.args.len() > 1 {
                return Some(Term::Application(TermApplication {
                    f: Rc::new(reduced_body),
                    args: app.args[1..].to_vec(),
                }));
            } else {
                return Some(reduced_body);
            }
        }
    }

    // Try to reduce the function
    if let Some(f) = reduce_step(&app.f) {
        return Some(Term::Application(TermApplication {
            f: Rc::new(f),
            args: app.args.clone(),
        }));
    }

    // Try to reduce the arguments (left to right)
    for (i, arg) in app.args.iter().enumerate() {
        if let Some(reduced_arg) = reduce_step(arg) {
            let mut new_args = app.args.clone();
            new_args[i] = reduced_arg;
            return Some(Term::Application(TermApplication {
                f: app.f.clone(),
                args: new_args,
            }));
        }
    }

    None
}

fn reduce_let_in(let_in: &TermLetIn) -> Option<Term> {
    // ζ-reduction: let x = t : T in u → u[x := t]
    let mut subst = Substitution::new();
    subst.add(let_in.var, let_in.term.clone());
    Some(substitute(&let_in.body, &subst))
}

/// Performs weak head normal form reduction (WHNF).
/// This reduces only the outermost redex and stops.
/// For applications, we reduce the function position to WHNF first,
/// then apply β-reduction if possible, but we do NOT reduce inside lambda bodies.
pub fn whnf(term: &Term) -> Term {
    match term {
        Term::Application(app) => {
            // First reduce the function to WHNF
            let f_whnf = whnf(&app.f);

            // If it's a lambda, perform ONE β-reduction step
            if let Term::Lambda(lambda) = &f_whnf {
                if !app.args.is_empty() {
                    let mut subst = Substitution::new();
                    subst.add(lambda.var, Rc::new(app.args[0].clone()));
                    let reduced_body = substitute(&lambda.target, &subst);

                    if app.args.len() > 1 {
                        // Create new application with remaining arguments
                        Term::Application(TermApplication {
                            f: Rc::new(reduced_body),
                            args: app.args[1..].to_vec(),
                        })
                    } else {
                        // Return the substituted body as-is (don't reduce further)
                        reduced_body
                    }
                } else {
                    Term::Application(TermApplication {
                        f: Rc::new(f_whnf),
                        args: app.args.clone(),
                    })
                }
            } else {
                // Function is not a lambda after WHNF
                Term::Application(TermApplication {
                    f: Rc::new(f_whnf),
                    args: app.args.clone(),
                })
            }
        }
        Term::LetIn(let_in) => {
            // ζ-reduction: immediately substitute
            let mut subst = Substitution::new();
            subst.add(let_in.var, let_in.term.clone());
            substitute(&let_in.body, &subst)
        }
        Term::Match(case) => {
            // Try to reduce the case expression
            if let Some(reduced_case) = reduce_case(case) {
                whnf(&reduced_case)
            } else {
                term.clone()
            }
        }
        Term::Fix(fix) => {
            // Try to reduce the fix expression
            if let Some(reduced_fix) = reduce_fix(fix) {
                whnf(&reduced_fix)
            } else {
                term.clone()
            }
        }
        // Other terms are already in WHNF
        _ => term.clone(),
    }
}

/// Reduces a case expression (ι-reduction)
/// match (C a₁ ... aₙ) with | C x₁ ... xₙ => t | ... => t[x₁ := a₁, ..., xₙ := aₙ]
fn reduce_case(case: &TermMatch) -> Option<Term> {
    // First reduce the scrutinee to WHNF
    let scrutinee_whnf = whnf(&case.scrutinee);

    // Check if the scrutinee is a constructor application
    // Constructor applications are represented as Application(Constant(constructor_id), args)
    match &scrutinee_whnf {
        Term::Constant(const_) => {
            // Zero-argument constructor
            for branch in &case.branches {
                if branch.constructor_id == const_.id {
                    if !branch.bound_vars.is_empty() {
                        // This should not happen if type checking is correct
                        return None;
                    }
                    return Some(branch.body.as_ref().clone());
                }
            }
        }
        Term::Application(app) => {
            if let Term::Constant(const_) = app.f.as_ref() {
                // Constructor with arguments
                for branch in &case.branches {
                    if branch.constructor_id == const_.id {
                        // Check that the number of bound variables matches the constructor arguments
                        if branch.bound_vars.len() != app.args.len() {
                            // This should not happen if type checking is correct
                            return None;
                        }

                        // Apply substitutions: substitute constructor arguments for bound variables
                        let mut subst = Substitution::new();
                        for (bound_var, arg) in branch.bound_vars.iter().zip(&app.args) {
                            subst.add(*bound_var, Rc::new(arg.clone()));
                        }

                        return Some(substitute(&branch.body, &subst));
                    }
                }
            }
        }
        _ => {}
    }

    // If the scrutinee is not a constructor, try to reduce it
    if let Some(reduced_scrutinee) = reduce_step(&case.scrutinee) {
        return Some(Term::Match(TermMatch {
            scrutinee: Rc::new(reduced_scrutinee),
            return_type: case.return_type.clone(),
            branches: case.branches.clone(),
        }));
    }

    // Try to reduce the return type
    if let Some(reduced_return_type) = reduce_step(&case.return_type) {
        return Some(Term::Match(TermMatch {
            scrutinee: case.scrutinee.clone(),
            return_type: Rc::new(reduced_return_type),
            branches: case.branches.clone(),
        }));
    }

    // Try to reduce branches
    for (i, branch) in case.branches.iter().enumerate() {
        if let Some(reduced_body) = reduce_step(&branch.body) {
            let mut new_branches = case.branches.clone();
            new_branches[i] = crate::term::MatchBranch {
                constructor_id: branch.constructor_id,
                bound_vars: branch.bound_vars.clone(),
                body: Rc::new(reduced_body),
            };
            return Some(Term::Match(TermMatch {
                scrutinee: case.scrutinee.clone(),
                return_type: case.return_type.clone(),
                branches: new_branches,
            }));
        }
    }

    None
}

/// Reduces a fixpoint expression (unfolding)
/// fix f : T := body → body[f := fix f : T := body]
fn reduce_fix(fix: &TermFix) -> Option<Term> {
    // Unfold the fixpoint: substitute the entire fix expression for the fix variable
    let mut subst = Substitution::new();
    subst.add(fix.fix_var, Rc::new(Term::Fix(fix.clone())));
    Some(substitute(&fix.body, &subst))
}
