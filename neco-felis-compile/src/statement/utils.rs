use neco_felis_syn::*;

pub fn count_let_variables_in_statements(statements: &Statements<PhaseParse>) -> i32 {
    match statements {
        Statements::Then(then) => {
            count_let_variables_in_statement(&then.head)
                + count_let_variables_in_statements(&then.tail)
        }
        Statements::Statement(statement) => count_let_variables_in_statement(statement),
        Statements::Nil => 0,
    }
}

pub fn count_let_variables_in_statement(statement: &Statement<PhaseParse>) -> i32 {
    match statement {
        Statement::Let(_) => 1,
        Statement::LetMut(_) => 2, // let mut uses 2 stack slots: one for value, one for reference
        Statement::Expr(proc_term) => count_let_variables_in_proc_term(proc_term),
        _ => 0,
    }
}

pub fn count_let_variables_in_proc_term(_proc_term: &ProcTerm<PhaseParse>) -> i32 {
    0
}

pub fn has_ptx_calls_in_statements(statements: &Statements<PhaseParse>) -> bool {
    match statements {
        Statements::Then(then) => {
            has_ptx_calls_in_statement(&then.head) || has_ptx_calls_in_statements(&then.tail)
        }
        Statements::Statement(statement) => has_ptx_calls_in_statement(statement),
        Statements::Nil => false,
    }
}

pub fn has_ptx_calls_in_statement(statement: &Statement<PhaseParse>) -> bool {
    match statement {
        Statement::CallPtx(_) => true,
        Statement::Expr(proc_term) => has_ptx_calls_in_proc_term(proc_term),
        _ => false,
    }
}

pub fn has_ptx_calls_in_proc_term(proc_term: &ProcTerm<PhaseParse>) -> bool {
    match proc_term {
        ProcTerm::If(if_expr) => {
            has_ptx_calls_in_statements(&if_expr.then_body)
                || if_expr
                    .else_clause
                    .as_ref()
                    .is_some_and(|else_clause| has_ptx_calls_in_statements(&else_clause.else_body))
        }
        _ => false,
    }
}
