#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        global_environment::GlobalEnvironment,
        id::Id,
        inductive::{ConstructorDefinition, InductiveDefinition, InductiveEnvironment, Parameter},
        local_context::LocalContext,
        term::{
            Sort, Term, TermCase, TermConstant, TermConstructor, TermSort, TermVariable, CaseBranch,
        },
        typechecker::infer_type,
        reduction::reduce_step,
    };

    #[test]
    fn test_bool_inductive() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        // Add Bool inductive type
        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

        // Test that Bool is defined
        let bool_def = env.inductives.get_inductive(bool_id).unwrap();
        assert_eq!(bool_def.name, bool_id);
        assert_eq!(bool_def.constructors.len(), 2);

        // Test constructor lookup
        let true_constr = env.inductives.get_constructor(true_id).unwrap();
        assert_eq!(true_constr.name, true_id);
        assert_eq!(true_constr.arity, 0);
    }

    #[test]
    fn test_nat_inductive() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        // Add Nat inductive type
        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        // Test that Nat is defined
        let nat_def = env.inductives.get_inductive(nat_id).unwrap();
        assert_eq!(nat_def.name, nat_id);
        assert_eq!(nat_def.constructors.len(), 2);

        // Test constructor lookup
        let zero_constr = env.inductives.get_constructor(zero_id).unwrap();
        assert_eq!(zero_constr.name, zero_id);
        assert_eq!(zero_constr.arity, 0);

        let succ_constr = env.inductives.get_constructor(succ_id).unwrap();
        assert_eq!(succ_constr.name, succ_id);
        assert_eq!(succ_constr.arity, 1);
    }

    #[test]
    fn test_constructor_typing() {
        let mut env = GlobalEnvironment::new();
        let ctx = LocalContext::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        // Add Bool inductive type
        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

        // Test True constructor
        let true_constr = Term::Constructor(TermConstructor {
            constructor_id: true_id,
            inductive_id: bool_id,
            args: vec![],
        });

        let ty = infer_type(&ctx, &env, &true_constr).unwrap();
        assert_eq!(
            *ty,
            Term::Constant(TermConstant { id: bool_id })
        );
    }

    #[test]
    fn test_case_reduction() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        // Add Bool inductive type
        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

        // Create: match True with | True => True | False => False
        let true_constr = Term::Constructor(TermConstructor {
            constructor_id: true_id,
            inductive_id: bool_id,
            args: vec![],
        });

        let false_constr = Term::Constructor(TermConstructor {
            constructor_id: false_id,
            inductive_id: bool_id,
            args: vec![],
        });

        let case_expr = Term::Case(TermCase {
            scrutinee: Rc::new(true_constr.clone()),
            return_type: Rc::new(Term::Constant(TermConstant { id: bool_id })),
            branches: vec![
                CaseBranch {
                    constructor_id: true_id,
                    bound_vars: vec![],
                    body: Rc::new(true_constr.clone()),
                },
                CaseBranch {
                    constructor_id: false_id,
                    bound_vars: vec![],
                    body: Rc::new(false_constr),
                },
            ],
        });

        // Test that case reduces to True
        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, true_constr);
    }

    #[test]
    fn test_nat_case_with_succ() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        // Add Nat inductive type
        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let n = Id::new();

        // Create: S (S O)
        let zero = Term::Constructor(TermConstructor {
            constructor_id: zero_id,
            inductive_id: nat_id,
            args: vec![],
        });

        let one = Term::Constructor(TermConstructor {
            constructor_id: succ_id,
            inductive_id: nat_id,
            args: vec![zero.clone()],
        });

        let two = Term::Constructor(TermConstructor {
            constructor_id: succ_id,
            inductive_id: nat_id,
            args: vec![one.clone()],
        });

        // Create: match (S (S O)) with | O => O | S n => n
        let case_expr = Term::Case(TermCase {
            scrutinee: Rc::new(two),
            return_type: Rc::new(Term::Constant(TermConstant { id: nat_id })),
            branches: vec![
                CaseBranch {
                    constructor_id: zero_id,
                    bound_vars: vec![],
                    body: Rc::new(zero),
                },
                CaseBranch {
                    constructor_id: succ_id,
                    bound_vars: vec![n],
                    body: Rc::new(Term::Variable(TermVariable { id: n })),
                },
            ],
        });

        // Test that case reduces to S O (which is one)
        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, one);
    }

    #[test]
    fn test_custom_inductive() {
        let mut inductive_env = InductiveEnvironment::new();
        
        // Define List A
        let list_id = Id::new();
        let nil_id = Id::new();
        let cons_id = Id::new();
        let a_param_id = Id::new();

        let type0 = Rc::new(Term::Sort(TermSort { sort: Sort::Type(0) }));
        let var_a = Rc::new(Term::Variable(TermVariable { id: a_param_id }));

        // List A : Type(0)
        let list_type = Rc::new(Term::Constant(TermConstant { id: list_id }));

        // nil : List A
        let nil_type = list_type.clone();
        let nil_constr = ConstructorDefinition::new(nil_id, nil_type, 0);

        // cons : A -> List A -> List A
        let cons_type = Rc::new(Term::Product(crate::term::TermProduct {
            var: Id::new(),
            source: var_a.clone(),
            target: Rc::new(Term::Product(crate::term::TermProduct {
                var: Id::new(),
                source: list_type.clone(),
                target: list_type,
            })),
        }));
        let cons_constr = ConstructorDefinition::new(cons_id, cons_type, 2);

        let list_def = InductiveDefinition::new(
            list_id,
            vec![Parameter {
                name: a_param_id,
                ty: type0,
            }],
            Rc::new(Term::Sort(TermSort { sort: Sort::Type(0) })),
            vec![nil_constr, cons_constr],
        );

        inductive_env.add_inductive(list_def).unwrap();

        // Test that List is defined
        let list_def = inductive_env.get_inductive(list_id).unwrap();
        assert_eq!(list_def.name, list_id);
        assert_eq!(list_def.parameters.len(), 1);
        assert_eq!(list_def.constructors.len(), 2);
    }
}