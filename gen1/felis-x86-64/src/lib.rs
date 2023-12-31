use felis_code_gen_prepared::CodeGenPreparedDecoration;
use felis_rename::rename_defs::DefId;
use felis_syn::{
    syn_expr::{SynExpr, SynExprApp, SynExprIdentWithPath, SynExprNumber, SynExprString},
    syn_file::{SynFile, SynFileItem},
    syn_statement::SynStatement,
};
use felis_type_checker::retrieve::RetrieveContext;
use neco_type_checker::type_term::TypeTerm;

fn string_length(s: &str) -> usize {
    let cs: Vec<_> = s.chars().collect();
    let mut res = 0;
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '\\' {
            i += 1;
        }
        res += 1;
        i += 1;
    }
    res
}

pub struct CompileContext {
    pub res: String,
    pub strings: Vec<(String, String)>,
    pub def_id_to_offset: std::collections::HashMap<DefId, usize>,
    pub retrieve_context: RetrieveContext,
}

pub fn byte_length_of_type_term(context: &CompileContext, ty: &TypeTerm) -> usize {
    match ty {
        TypeTerm::Star(_) => todo!(),
        TypeTerm::Base(ty_id) => {
            if ty_id == &context.retrieve_context.ty_unit {
                0
            } else if ty_id == &context.retrieve_context.ty_i64 {
                8
            } else if ty_id == &context.retrieve_context.ty_str {
                16
            } else {
                panic!()
            }
        }
        TypeTerm::Arrow(_, _) => todo!(),
        TypeTerm::Var(_) => todo!(),
        TypeTerm::App(_, _) => todo!(),
        TypeTerm::Candidates(_) => todo!(),
        TypeTerm::Unknown => todo!(),
    }
}

pub fn ty_of_expr(expr: &SynExpr<CodeGenPreparedDecoration>) -> TypeTerm {
    match expr {
        SynExpr::IdentWithPath(ident_with_path) => ident_with_path.ext.ty.clone(),
        SynExpr::App(app) => app.ext.ty.clone(),
        SynExpr::Match(_) => todo!(),
        SynExpr::Paren(_) => todo!(),
        SynExpr::String(s) => s.ext.ty.clone(),
        SynExpr::Number(number) => number.ext.ty.clone(),
    }
}

pub fn compile_expr_r_app(
    context: &mut CompileContext,
    expr_app: &SynExprApp<CodeGenPreparedDecoration>,
) {
    let app = expr_app;
    let fun = app.exprs[0].clone();
    let SynExpr::IdentWithPath(fun) = fun else {
        panic!()
    };
    let fun = fun.ident.as_str();
    if fun == "__write_to_stdout" {
        // sys_write
        // rax: syscall number = 1
        // rdi: file descriptor = 1 (stdout)
        // rsi: buffer = ptr
        // rdx: buffer size = length
        let arg = app.exprs[1].clone();
        compile_expr_r(context, &arg);
        // pop: from front to back
        context.res.push_str("    mov rax, 1\n");
        context.res.push_str("    mov rdi, 1\n");
        context.res.push_str("    pop rdx\n");
        context.res.push_str("    pop rsi\n");
        context.res.push_str("    syscall\n");
    } else if fun == "__exit" {
        // sys_exit
        // rax: syscall number = 60
        // rdi: exit status = status
        let arg = app.exprs[1].clone();
        compile_expr_r(context, &arg);
        context.res.push_str("    mov rax, 60\n");
        context.res.push_str("    pop rdi\n");
        context.res.push_str("    syscall\n");
    } else if fun == "__add_i64" {
        // add
        // rax: arg1
        // rdi: arg2
        // rax: result
        let arg1 = app.exprs[1].clone();
        let arg2 = app.exprs[2].clone();
        compile_expr_r(context, &arg1);
        compile_expr_r(context, &arg2);
        context.res.push_str("    pop rdi\n");
        context.res.push_str("    pop rax\n");
        context.res.push_str("    add rax, rdi\n");
        context.res.push_str("    push rax\n");
    } else {
        panic!();
    }
}

pub fn compile_expr_r_string(
    context: &mut CompileContext,
    expr_string: &SynExprString<CodeGenPreparedDecoration>,
) {
    let s = expr_string;
    let s = s.token_string.string.clone();
    let label = format!("__string_{}", context.strings.len());
    context.strings.push((label.clone(), s.clone()));
    // push: from back to front
    context
        .res
        .push_str(format!("    lea rax, {}\n", label).as_str());
    context.res.push_str("    push rax\n");
    context
        .res
        .push_str(&format!("    mov rax, {}\n", string_length(&s)));
    context.res.push_str("    push rax\n");
}

pub fn compile_expr_r_ident_with_path(
    context: &mut CompileContext,
    expr_ident_with_path: &SynExprIdentWithPath<CodeGenPreparedDecoration>,
) {
    let def_id = expr_ident_with_path.ext.use_id;
    let offset = context.def_id_to_offset.get(&def_id).unwrap();
    // push: from back to front
    let byte_length = byte_length_of_type_term(&context, &expr_ident_with_path.ext.ty);
    assert!(byte_length % 8 == 0);
    eprintln!("byte_length(1): {}", byte_length);
    for i in (0..byte_length / 8).rev() {
        context
            .res
            .push_str(format!("    mov rax, [rbp-{}+{}]\n", offset, 8 * i).as_str());
        context.res.push_str("    push rax\n");
    }
}

// Return the address of the variable ('s first element).
pub fn compile_expr_l_ident_with_path(
    context: &mut CompileContext,
    expr_ident_with_path: &SynExprIdentWithPath<CodeGenPreparedDecoration>,
) {
    let def_id = expr_ident_with_path.ext.use_id;
    let offset = context.def_id_to_offset.get(&def_id).unwrap();
    context
        .res
        .push_str(format!("    lea rax, [rbp-{}]\n", offset).as_str());
    context.res.push_str("    push rax\n");
}

pub fn compile_expr_r(context: &mut CompileContext, expr: &SynExpr<CodeGenPreparedDecoration>) {
    match expr {
        SynExpr::IdentWithPath(expr_ident_with_path) => {
            compile_expr_r_ident_with_path(context, expr_ident_with_path);
        }
        SynExpr::App(expr_app) => {
            compile_expr_r_app(context, expr_app);
        }
        SynExpr::Match(_) => todo!(),
        SynExpr::Paren(_) => todo!(),
        SynExpr::String(expr_string) => {
            compile_expr_r_string(context, expr_string);
        }
        SynExpr::Number(expr_number) => {
            compile_expr_r_number(context, expr_number);
        }
    }
}

pub fn number_string_to_number(s: &str) -> String {
    s.to_string()
}

pub fn compile_expr_r_number(
    context: &mut CompileContext,
    expr_number: &SynExprNumber<CodeGenPreparedDecoration>,
) {
    let s = expr_number.number.as_str();
    let s = number_string_to_number(s);
    context
        .res
        .push_str(format!("    mov rax, {}\n", number_string_to_number(&s)).as_str());
    context.res.push_str("    push rax\n");
}

pub fn compile_expr_l(context: &mut CompileContext, expr: &SynExpr<CodeGenPreparedDecoration>) {
    match expr {
        SynExpr::IdentWithPath(expr_ident_with_path) => {
            compile_expr_l_ident_with_path(context, expr_ident_with_path);
        }
        SynExpr::App(_) => todo!(),
        SynExpr::Match(_) => todo!(),
        SynExpr::Paren(_) => todo!(),
        SynExpr::String(_) => todo!(),
        SynExpr::Number(_) => todo!(),
    }
}

pub fn compile(
    retrieve_context: RetrieveContext,
    file: SynFile<CodeGenPreparedDecoration>,
) -> String {
    let mut context = CompileContext {
        res: String::new(),
        strings: vec![],
        def_id_to_offset: std::collections::HashMap::new(),
        retrieve_context,
    };

    // get entrypoint
    let mut entrypoint_name = None;
    for item in &file.items {
        let entrypoint = match item {
            SynFileItem::Entrypoint(entrypoint) => entrypoint,
            _ => continue,
        };
        entrypoint_name = Some(format!(
            "{}_{}",
            entrypoint.ident.as_str(),
            entrypoint.ext.use_id.as_usize()
        ));
    }

    // asm template
    context.res.push_str(
        format!(
            r#"
.intel_syntax noprefix
    .global _start
_start:
    pushfq
    pop rax
    mov rdi, 0x0000000000040000
    or rax, rdi
    push rax
    popfq
    call {}
    mov rdi, rax
    mov rax, 60
    syscall
"#,
            entrypoint_name.unwrap()
        )
        .as_str(),
    );

    // Proc
    for item in &file.items {
        let proc = match item {
            SynFileItem::ProcDef(proc) => proc,
            _ => continue,
        };

        context
            .res
            .push_str(format!("{}_{}:\n", proc.name.as_str(), proc.ext.id.as_usize()).as_str());

        context.res.push_str("    push rbp\n");
        context.res.push_str("    mov rbp, rsp\n");

        context.def_id_to_offset.clear();
        let mut offset = 0;
        for a in &proc.ext.lets {
            let byte_length = byte_length_of_type_term(&context, &a.1);
            offset += byte_length;
            context.def_id_to_offset.insert(a.0, offset);
        }

        context.res.push_str(&format!("    sub rsp, {}\n", offset));

        for statement in &proc.proc_block.statements {
            match statement {
                SynStatement::ExprSemi(expr_semi) => {
                    let expr = expr_semi.expr.clone();
                    compile_expr_r(&mut context, &expr);
                }
                SynStatement::Let(statement_let) => {
                    if let Some(initial) = &statement_let.initial {
                        compile_expr_r(&mut context, &initial.expr);
                        let offset = context.def_id_to_offset.get(&statement_let.ext.id).unwrap();

                        eprintln!("statement_let = {:?}", statement_let);
                        let ty = ty_of_expr(&initial.expr);
                        let byte_length = byte_length_of_type_term(&context, &ty);
                        assert!(byte_length % 8 == 0);
                        eprintln!("byte_length(2): {}", byte_length);
                        for i in 0..byte_length / 8 {
                            context.res.push_str("    pop rax\n");
                            context.res.push_str(
                                format!("    mov [rbp-{}+{}], rax\n", offset, 8 * i).as_str(),
                            );
                        }
                    }
                }
                SynStatement::Assign(statement_assign) => {
                    compile_expr_r(&mut context, &statement_assign.rhs);
                    compile_expr_l(&mut context, &statement_assign.lhs);
                    let ty = ty_of_expr(&statement_assign.rhs);
                    let byte_length = byte_length_of_type_term(&context, &ty);
                    assert!(byte_length % 8 == 0);
                    eprintln!("byte_length(3): {}", byte_length);
                    context.res.push_str("    pop rax\n");
                    for i in 0..byte_length / 8 {
                        context.res.push_str("    pop rdi\n");
                        context
                            .res
                            .push_str(format!("    mov [rax+{}], rdi\n", 8 * i).as_str());
                    }
                }
            }
        }

        context.res.push_str("    mov rsp, rbp\n");
        context.res.push_str("    pop rbp\n");
        context.res.push_str("    mov rax, 0\n");
        context.res.push_str("    ret\n");
    }

    // string literals
    context.res.push_str("    .section .data\n");
    for (label, s) in &context.strings {
        context.res.push_str(format!("{}:\n", label).as_str());
        context
            .res
            .push_str(format!("    .asciz \"{}\"\n", s).as_str());
    }

    context.res
}
