#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        id::Id,
        reduction::{normalize, reduce_step, whnf},
        term::{
            Sort, Term, TermApplication, TermLambda, TermLetIn, TermSort, TermVariable,
        },
    };

    #[test]
    fn test_beta_reduction_simple() {
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
    fn test_beta_reduction_nested() {
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
    fn test_let_reduction() {
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
    fn test_no_reduction_for_normal_form() {
        // Test that variables and sorts don't reduce
        let x = Id::new();
        let var = Term::Variable(TermVariable { id: x });
        let sort = Term::Sort(TermSort { sort: Sort::Type(0) });
        
        assert!(reduce_step(&var).is_none());
        assert!(reduce_step(&sort).is_none());
    }

    #[test]
    fn test_whnf() {
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
    fn test_multiple_arguments() {
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
}