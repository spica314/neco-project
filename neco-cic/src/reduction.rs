use std::rc::Rc;

use crate::{
    substitution::{Substitution, substitute},
    term::{Term, TermApplication, TermLambda, TermLetIn, TermMatch, TermProduct},
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
    if let Term::Lambda(lambda) = app.f.as_ref()
        && !app.args.is_empty()
    {
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
            new_branches[i] = crate::term::TermMatchBranch {
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        id::Id,
        term::{Sort, Term, TermApplication, TermLambda, TermLetIn, TermSort, TermVariable},
    };

    use super::{normalize, reduce_step, whnf};

    #[test]
    fn test_beta_reduction_identity_function() {
        // Test: (λx. x) y → y
        let x = Id::new();
        let y = Id::new();

        let lambda = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        let app = Term::Application(TermApplication {
            f: Rc::new(lambda),
            args: vec![Term::Variable(TermVariable { id: y })],
        });

        let reduced = reduce_step(&app);
        assert!(reduced.is_some());
        assert_eq!(reduced.unwrap(), Term::Variable(TermVariable { id: y }));
    }

    #[test]
    fn test_beta_reduction_constant_function() {
        // Test: (λx. λy. x) a b → a
        let x = Id::new();
        let y = Id::new();
        let a = Id::new();
        let b = Id::new();

        let inner_lambda = Term::Lambda(TermLambda {
            var: y,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        let outer_lambda = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(inner_lambda),
        });

        let app = Term::Application(TermApplication {
            f: Rc::new(outer_lambda),
            args: vec![
                Term::Variable(TermVariable { id: a }),
                Term::Variable(TermVariable { id: b }),
            ],
        });

        let normalized = normalize(&app);
        assert_eq!(normalized, Term::Variable(TermVariable { id: a }));
    }

    #[test]
    fn test_beta_reduction_three_arguments() {
        // Test: (λx. λy. λz. x) a b c → a
        let x = Id::new();
        let y = Id::new();
        let z = Id::new();
        let a = Id::new();
        let b = Id::new();
        let c = Id::new();

        let inner_inner_lambda = Term::Lambda(TermLambda {
            var: z,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        let inner_lambda = Term::Lambda(TermLambda {
            var: y,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(inner_inner_lambda),
        });

        let outer_lambda = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(inner_lambda),
        });

        let app = Term::Application(TermApplication {
            f: Rc::new(outer_lambda),
            args: vec![
                Term::Variable(TermVariable { id: a }),
                Term::Variable(TermVariable { id: b }),
                Term::Variable(TermVariable { id: c }),
            ],
        });

        let normalized = normalize(&app);
        assert_eq!(normalized, Term::Variable(TermVariable { id: a }));
    }

    #[test]
    fn test_zeta_reduction_let_in() {
        // Test: let x = a in x → a
        let x = Id::new();
        let a = Id::new();

        let let_in = Term::LetIn(TermLetIn {
            var: x,
            term: Rc::new(Term::Variable(TermVariable { id: a })),
            ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            body: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        let reduced = reduce_step(&let_in);
        assert!(reduced.is_some());
        assert_eq!(reduced.unwrap(), Term::Variable(TermVariable { id: a }));
    }

    #[test]
    fn test_variable_is_normal_form() {
        let x = Id::new();
        let var = Term::Variable(TermVariable { id: x });

        assert!(reduce_step(&var).is_none());
    }

    #[test]
    fn test_sort_is_normal_form() {
        let sort = Term::Sort(TermSort {
            sort: Sort::Type(0),
        });

        assert!(reduce_step(&sort).is_none());
    }

    #[test]
    fn test_constant_is_normal_form() {
        let c = Id::new();
        let constant = Term::Constant(crate::term::TermConstant { id: c });

        assert!(reduce_step(&constant).is_none());
    }

    #[test]
    fn test_whnf_preserves_outer_structure() {
        // Test: (λx. (λy. y) x) z → (λy. y) z (WHNF, not fully reduced)
        let x = Id::new();
        let y = Id::new();
        let z = Id::new();

        let inner_lambda = Term::Lambda(TermLambda {
            var: y,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(Term::Variable(TermVariable { id: y })),
        });

        let inner_app = Term::Application(TermApplication {
            f: Rc::new(inner_lambda),
            args: vec![Term::Variable(TermVariable { id: x })],
        });

        let outer_lambda = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(inner_app),
        });

        let app = Term::Application(TermApplication {
            f: Rc::new(outer_lambda),
            args: vec![Term::Variable(TermVariable { id: z })],
        });

        let whnf_result = whnf(&app);

        // After WHNF reduction of (λx. (λy. y) x) z, we get (λy. y) z
        if let Term::Application(app) = &whnf_result {
            assert!(matches!(app.f.as_ref(), Term::Lambda(_)));
            assert_eq!(app.args.len(), 1);
            assert_eq!(app.args[0], Term::Variable(TermVariable { id: z }));
        } else {
            panic!("Expected application after WHNF, got {:?}", whnf_result);
        }
    }

    #[test]
    fn test_normalize_reduces_completely() {
        // Test that normalize reduces nested applications completely
        let x = Id::new();

        let lambda = Term::Lambda(TermLambda {
            var: x,
            source_ty: Rc::new(Term::Sort(TermSort { sort: Sort::Set })),
            target: Rc::new(Term::Variable(TermVariable { id: x })),
        });

        let app = Term::Application(TermApplication {
            f: Rc::new(lambda),
            args: vec![Term::Variable(TermVariable { id: x })],
        });

        let normalized = normalize(&app);
        assert_eq!(normalized, Term::Variable(TermVariable { id: x }));
    }
}
