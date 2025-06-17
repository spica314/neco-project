use std::rc::Rc;

use crate::{
    substitution::{substitute, Substitution},
    term::{Term, TermApplication, TermLambda, TermLetIn, TermProduct},
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
        // Other terms are already in WHNF
        _ => term.clone(),
    }
}
