#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        global_environment::GlobalEnvironment,
        id::Id,
        local_context::LocalContext,
        term::{Sort, Term, TermApplication, TermLambda, TermProduct, TermSort, TermVariable},
        typechecker::{TypeError, check_type, infer_type},
    };

    #[test]
    fn test_sort_typing() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment {};

        // Test: Set : Type(0)
        let set = Term::Sort(TermSort { sort: Sort::Set });
        let ty = infer_type(&ctx, &env, &set).unwrap();
        assert_eq!(
            *ty,
            Term::Sort(TermSort {
                sort: Sort::Type(0)
            })
        );

        // Test: Type(0) : Type(1)
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
    fn test_variable_typing() {
        let mut ctx = LocalContext::new();
        let env = GlobalEnvironment {};
        let x = Id::new();

        // Test: unbound variable should fail
        let var = Term::Variable(TermVariable { id: x });
        assert!(matches!(
            infer_type(&ctx, &env, &var),
            Err(TypeError::UnboundVariable(_))
        ));

        // Test: bound variable should succeed
        let set = Rc::new(Term::Sort(TermSort { sort: Sort::Set }));
        ctx.extend(x, set.clone()).unwrap();
        let ty = infer_type(&ctx, &env, &var).unwrap();
        assert_eq!(ty, set);
    }

    #[test]
    fn test_lambda_typing() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment {};
        let x = Id::new();

        // Test: λx:Set. x : Set → Set
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
        let env = GlobalEnvironment {};
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

        // Test: (λx:Set. x) y : Set
        let ty = infer_type(&ctx_with_y, &env, &app).unwrap();
        assert_eq!(*ty, set);
    }

    #[test]
    fn test_product_typing() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment {};
        let x = Id::new();

        // Test: Πx:Set. Set : Type(0)
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
    fn test_type_checking() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment {};
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

        // Should succeed
        assert!(check_type(&ctx, &env, &id_fun, &expected_type).is_ok());

        // Wrong type should fail
        assert!(check_type(&ctx, &env, &id_fun, &set).is_err());
    }

    #[test]
    fn test_dependent_product() {
        let ctx = LocalContext::new();
        let env = GlobalEnvironment {};
        let a = Id::new();
        let x = Id::new();

        // Test: Πa:Type(0). Πx:a. a : Type(0)
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
