use std::{collections::HashMap, rc::Rc};

use crate::{
    id::Id,
    term::{
        MatchBranch, Term, TermApplication, TermConstant, TermFix, TermLambda, TermLetIn,
        TermMatch, TermProduct, TermSort, TermVariable,
    },
};

// Note: Variable capture is not a concern in this implementation because each binding
// (in Lambda, Product, and LetIn) is assigned a unique ID when constructed.
// This means that bound variables will always have different IDs from any variables
// being substituted, preventing capture issues.

pub struct Substitution {
    subst: HashMap<Id, Rc<Term>>,
}

impl Substitution {
    pub fn new() -> Self {
        Substitution {
            subst: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: Id, term: Rc<Term>) {
        self.subst.insert(id, term);
    }

    pub fn get(&self, id: Id) -> Option<Rc<Term>> {
        self.subst.get(&id).cloned()
    }
}

pub fn substitute(term: &Term, subst: &Substitution) -> Term {
    match term {
        Term::Sort(term_sort) => substitute_sort(term_sort, subst),
        Term::Variable(term_variable) => substitute_variable(term_variable, subst),
        Term::Constant(term_constant) => substitute_constant(term_constant, subst),
        Term::Product(term_product) => substitute_product(term_product, subst),
        Term::Lambda(term_lambda) => substitute_lambda(term_lambda, subst),
        Term::Application(term_application) => substitute_application(term_application, subst),
        Term::LetIn(term_let_in) => substitute_let_in(term_let_in, subst),
        Term::Match(term_case) => substitute_case(term_case, subst),
        Term::Fix(term_fix) => substitute_fix(term_fix, subst),
    }
}

fn substitute_sort(term_sort: &TermSort, _subst: &Substitution) -> Term {
    Term::Sort(term_sort.clone())
}

fn substitute_variable(term_variable: &TermVariable, subst: &Substitution) -> Term {
    if let Some(u) = subst.get(term_variable.id) {
        u.as_ref().clone()
    } else {
        Term::Variable(term_variable.clone())
    }
}

fn substitute_constant(term_constant: &TermConstant, _subst: &Substitution) -> Term {
    Term::Constant(term_constant.clone())
}

fn substitute_product(term_product: &TermProduct, subst: &Substitution) -> Term {
    let source = substitute(&term_product.source, subst);
    let target = substitute(&term_product.target, subst);
    Term::Product(TermProduct {
        var: term_product.var,
        source: Rc::new(source),
        target: Rc::new(target),
    })
}

fn substitute_lambda(term_lambda: &TermLambda, subst: &Substitution) -> Term {
    let source_ty = substitute(&term_lambda.source_ty, subst);
    let target = substitute(&term_lambda.target, subst);
    Term::Lambda(TermLambda {
        var: term_lambda.var,
        source_ty: Rc::new(source_ty),
        target: Rc::new(target),
    })
}

fn substitute_application(term_application: &TermApplication, subst: &Substitution) -> Term {
    let f = substitute(&term_application.f, subst);
    let args: Vec<_> = term_application
        .args
        .iter()
        .map(|t| substitute(t, subst))
        .collect();
    Term::Application(TermApplication {
        f: Rc::new(f),
        args,
    })
}

fn substitute_let_in(term_let_in: &TermLetIn, subst: &Substitution) -> Term {
    let term = substitute(&term_let_in.term, subst);
    let ty = substitute(&term_let_in.ty, subst);
    let body = substitute(&term_let_in.body, subst);
    Term::LetIn(TermLetIn {
        var: term_let_in.var,
        term: Rc::new(term),
        ty: Rc::new(ty),
        body: Rc::new(body),
    })
}

fn substitute_case(term_case: &TermMatch, subst: &Substitution) -> Term {
    let scrutinee = substitute(&term_case.scrutinee, subst);
    let return_type = substitute(&term_case.return_type, subst);
    let branches: Vec<_> = term_case
        .branches
        .iter()
        .map(|branch| substitute_case_branch(branch, subst))
        .collect();
    Term::Match(TermMatch {
        scrutinee: Rc::new(scrutinee),
        return_type: Rc::new(return_type),
        branches,
    })
}

fn substitute_case_branch(branch: &MatchBranch, subst: &Substitution) -> MatchBranch {
    // For simplicity, don't handle bound variable capture for now
    // TODO: Properly handle bound variable capture
    let body = substitute(&branch.body, subst);
    MatchBranch {
        constructor_id: branch.constructor_id,
        bound_vars: branch.bound_vars.clone(),
        body: Rc::new(body),
    }
}

fn substitute_fix(term_fix: &TermFix, subst: &Substitution) -> Term {
    let body_type = substitute(&term_fix.body_type, subst);

    // For simplicity, don't handle bound variable capture for now
    // TODO: Properly handle bound variable capture
    let body = substitute(&term_fix.body, subst);

    Term::Fix(TermFix {
        fix_var: term_fix.fix_var,
        body_type: Rc::new(body_type),
        body: Rc::new(body),
    })
}
