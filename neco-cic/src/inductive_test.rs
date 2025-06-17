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
    fn test_bool_inductive_definition() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

        let bool_def = env.inductives.get_inductive(bool_id).unwrap();
        assert_eq!(bool_def.name, bool_id);
        assert_eq!(bool_def.constructors.len(), 2);
    }

    #[test]
    fn test_bool_constructor_lookup() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

        let true_constr = env.inductives.get_constructor(true_id).unwrap();
        assert_eq!(true_constr.name, true_id);
        assert_eq!(true_constr.arity, 0);
    }

    #[test]
    fn test_nat_inductive_definition() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let nat_def = env.inductives.get_inductive(nat_id).unwrap();
        assert_eq!(nat_def.name, nat_id);
        assert_eq!(nat_def.constructors.len(), 2);
    }

    #[test]
    fn test_nat_zero_constructor() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let zero_constr = env.inductives.get_constructor(zero_id).unwrap();
        assert_eq!(zero_constr.name, zero_id);
        assert_eq!(zero_constr.arity, 0);
    }

    #[test]
    fn test_nat_succ_constructor() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let succ_constr = env.inductives.get_constructor(succ_id).unwrap();
        assert_eq!(succ_constr.name, succ_id);
        assert_eq!(succ_constr.arity, 1);
    }

    #[test]
    fn test_bool_true_constructor_typing() {
        let mut env = GlobalEnvironment::new();
        let ctx = LocalContext::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

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
    fn test_bool_false_constructor_typing() {
        let mut env = GlobalEnvironment::new();
        let ctx = LocalContext::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

        let false_constr = Term::Constructor(TermConstructor {
            constructor_id: false_id,
            inductive_id: bool_id,
            args: vec![],
        });

        let ty = infer_type(&ctx, &env, &false_constr).unwrap();
        assert_eq!(
            *ty,
            Term::Constant(TermConstant { id: bool_id })
        );
    }

    #[test]
    fn test_case_reduction_true_branch() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

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

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, true_constr);
    }

    #[test]
    fn test_case_reduction_false_branch() {
        let mut env = GlobalEnvironment::new();
        let bool_id = Id::new();
        let true_id = Id::new();
        let false_id = Id::new();

        env.inductives
            .add_bool(bool_id, true_id, false_id)
            .unwrap();

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
            scrutinee: Rc::new(false_constr.clone()),
            return_type: Rc::new(Term::Constant(TermConstant { id: bool_id })),
            branches: vec![
                CaseBranch {
                    constructor_id: true_id,
                    bound_vars: vec![],
                    body: Rc::new(true_constr),
                },
                CaseBranch {
                    constructor_id: false_id,
                    bound_vars: vec![],
                    body: Rc::new(false_constr.clone()),
                },
            ],
        });

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, false_constr);
    }

    #[test]
    fn test_nat_case_zero_branch() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let zero = Term::Constructor(TermConstructor {
            constructor_id: zero_id,
            inductive_id: nat_id,
            args: vec![],
        });

        let case_expr = Term::Case(TermCase {
            scrutinee: Rc::new(zero.clone()),
            return_type: Rc::new(Term::Constant(TermConstant { id: nat_id })),
            branches: vec![
                CaseBranch {
                    constructor_id: zero_id,
                    bound_vars: vec![],
                    body: Rc::new(zero.clone()),
                },
                CaseBranch {
                    constructor_id: succ_id,
                    bound_vars: vec![Id::new()],
                    body: Rc::new(zero.clone()),
                },
            ],
        });

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, zero);
    }

    #[test]
    fn test_nat_case_succ_branch_with_substitution() {
        let mut env = GlobalEnvironment::new();
        let nat_id = Id::new();
        let zero_id = Id::new();
        let succ_id = Id::new();

        env.inductives.add_nat(nat_id, zero_id, succ_id).unwrap();

        let n = Id::new();

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

        let reduced = reduce_step(&case_expr).unwrap();
        assert_eq!(reduced, one);
    }

    #[test]
    fn test_list_inductive_definition() {
        let mut inductive_env = InductiveEnvironment::new();
        
        let list_id = Id::new();
        let nil_id = Id::new();
        let cons_id = Id::new();
        let a_param_id = Id::new();

        let type0 = Rc::new(Term::Sort(TermSort { sort: Sort::Type(0) }));
        let var_a = Rc::new(Term::Variable(TermVariable { id: a_param_id }));
        let list_type = Rc::new(Term::Constant(TermConstant { id: list_id }));

        let nil_type = list_type.clone();
        let nil_constr = ConstructorDefinition::new(nil_id, nil_type, 0);

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

        let list_def = inductive_env.get_inductive(list_id).unwrap();
        assert_eq!(list_def.name, list_id);
        assert_eq!(list_def.parameters.len(), 1);
    }

    #[test]
    fn test_list_constructor_count() {
        let mut inductive_env = InductiveEnvironment::new();
        
        let list_id = Id::new();
        let nil_id = Id::new();
        let cons_id = Id::new();
        let a_param_id = Id::new();

        let type0 = Rc::new(Term::Sort(TermSort { sort: Sort::Type(0) }));
        let var_a = Rc::new(Term::Variable(TermVariable { id: a_param_id }));
        let list_type = Rc::new(Term::Constant(TermConstant { id: list_id }));

        let nil_type = list_type.clone();
        let nil_constr = ConstructorDefinition::new(nil_id, nil_type, 0);

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

        let list_def = inductive_env.get_inductive(list_id).unwrap();
        assert_eq!(list_def.constructors.len(), 2);
    }
}