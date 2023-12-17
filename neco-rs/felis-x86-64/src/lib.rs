use felis_code_gen_prepared::CodeGenPreparedDecoration;
use felis_rename::rename_defs::DefId;
use felis_syn::{
    syn_expr::{SynExpr, SynExprApp, SynExprIdentWithPath, SynExprString},
    syn_file::{SynFile, SynFileItem},
    syn_statement::SynStatement,
};

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
}

pub fn compile_expr_app(
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
        let arg = app.exprs[1].clone();
        compile_expr(context, &arg);
        context.res.push_str("    mov rax, 1\n");
        context.res.push_str("    mov rdi, 1\n");
        context.res.push_str("    pop rsi\n");
        context.res.push_str("    pop rdx\n");
        context.res.push_str("    syscall\n");
    } else {
        panic!();
    }
}

pub fn compile_expr_string(
    context: &mut CompileContext,
    expr_string: &SynExprString<CodeGenPreparedDecoration>,
) {
    let s = expr_string;
    let s = s.token_string.string.clone();
    let label = format!("__string_{}", context.strings.len());
    context.strings.push((label.clone(), s.clone()));
    context
        .res
        .push_str(&format!("    mov rax, {}\n", string_length(&s)));
    context.res.push_str("    push rax\n");
    context
        .res
        .push_str(format!("    lea rax, {}\n", label).as_str());
    context.res.push_str("    push rax\n");
    context.res.push_str("    syscall\n");
}

pub fn compile_expr_ident_with_path(
    context: &mut CompileContext,
    expr_ident_with_path: &SynExprIdentWithPath<CodeGenPreparedDecoration>,
) {
    let def_id = expr_ident_with_path.ext.use_id;
    let offset = context.def_id_to_offset.get(&def_id).unwrap();
    context
        .res
        .push_str(format!("    mov rax, [rbp-{}-8]\n", offset).as_str());
    context.res.push_str("    push rax\n");
    context
        .res
        .push_str(format!("    mov rax, [rbp-{}-16]\n", offset).as_str());
    context.res.push_str("    push rax\n");
}

pub fn compile_expr(context: &mut CompileContext, expr: &SynExpr<CodeGenPreparedDecoration>) {
    match expr {
        SynExpr::IdentWithPath(expr_ident_with_path) => {
            compile_expr_ident_with_path(context, expr_ident_with_path);
        }
        SynExpr::App(expr_app) => {
            compile_expr_app(context, expr_app);
        }
        SynExpr::Match(_) => todo!(),
        SynExpr::Paren(_) => todo!(),
        SynExpr::String(expr_string) => {
            compile_expr_string(context, expr_string);
        }
        SynExpr::Block(_) => todo!(),
    }
}

pub fn compile(file: SynFile<CodeGenPreparedDecoration>) -> String {
    let mut context = CompileContext {
        res: String::new(),
        strings: vec![],
        def_id_to_offset: std::collections::HashMap::new(),
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
            context.def_id_to_offset.insert(a.0, offset);
            offset += 16;
        }

        context.res.push_str(&format!("    sub rsp, {}\n", offset));

        for statement in &proc.proc_block.statements {
            match statement {
                SynStatement::Expr(expr) => {
                    compile_expr(&mut context, expr);
                }
                SynStatement::ExprSemi(expr_semi) => {
                    let expr = expr_semi.expr.clone();
                    compile_expr(&mut context, &expr);
                }
                SynStatement::Let(statement_let) => {
                    compile_expr(&mut context, &statement_let.expr);
                    let offset = context.def_id_to_offset.get(&statement_let.ext.id).unwrap();
                    context.res.push_str("    pop rax\n");
                    context
                        .res
                        .push_str(format!("    mov [rbp-{}-16], rax\n", offset).as_str());
                    context.res.push_str("    pop rax\n");
                    context
                        .res
                        .push_str(format!("    mov [rbp-{}-8], rax\n", offset).as_str());
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

#[cfg(test)]
mod test {
    use std::{path::Path, process::Command};

    use super::*;
    use felis_rename::{
        path_table::construct_path_table_syn_file,
        rename_defs::{rename_defs_file, RenameDefContext},
        rename_uses::{rename_uses_file, RenameUseContext},
        setup_resolver_for_prelude,
    };
    use felis_syn::{decoration::UD, syn_file::SynFile, test_utils::parse_from_str};
    use felis_type_checker::{
        retrieve::{retrieve_file, setup_retrieve_context},
        typing::typing_file,
    };
    use neco_resolver::Resolver;

    fn test_name_to_file_name(name: &str) -> String {
        name.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
    }

    #[test]
    fn felis_compile_test_1() {
        let s = std::fs::read_to_string("../../examples/hello-world/main.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        let mut resolver = Resolver::new();
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        path_table.setup_resolver(file_2.ext.id, &mut resolver);
        let prelude_map = setup_resolver_for_prelude(&mut context, &mut resolver);
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        let mut context = setup_retrieve_context(&prelude_map);
        retrieve_file(&mut context, &file_3);
        context.type_checker.resolve();
        let var_types = context.type_checker.get_all();
        for (var_id, ty) in &var_types {
            eprintln!("{:?}: {:?}", var_id, ty);
        }
        assert_eq!(var_types.len(), 2);
        let file_4 = typing_file(&mut context, &file_3);

        let file_5 = felis_code_gen_prepared::prepare_code_gen_file(&file_4);

        let res = compile(file_5);
        let base_path = format!("/tmp/{}", test_name_to_file_name("felis_compile_test_1"));
        {
            let base_path = Path::new(&base_path);
            if base_path.exists() {
                std::fs::remove_dir_all(&base_path).unwrap();
            }
        }
        std::fs::create_dir(&base_path).unwrap();
        std::fs::write(format!("{}/main.s", base_path), &res).unwrap();

        let status = Command::new("as")
            .args(&[
                "-o",
                &format!("{}/main.o", base_path),
                &format!("{}/main.s", base_path),
            ])
            .status()
            .expect("");
        assert_eq!(status.code(), Some(0));
        let status = Command::new("ld")
            .args(&[
                "-o",
                &format!("{}/a.out", base_path),
                &format!("{}/main.o", base_path),
            ])
            .status()
            .expect("");
        assert_eq!(status.code(), Some(0));
        let output = Command::new(&format!("{}/a.out", base_path))
            .output()
            .expect("");
        assert_eq!(output.stdout, b"Hello, world!\n");
        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn felis_compile_test_2() {
        let s = std::fs::read_to_string("../../examples/let-string/main.fe").unwrap();
        let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
        let mut context = RenameDefContext::new();
        let file_2 = rename_defs_file(&mut context, &file).unwrap();
        let mut resolver = Resolver::new();
        let path_table = construct_path_table_syn_file(&file_2).unwrap();
        path_table.setup_resolver(file_2.ext.id, &mut resolver);
        let prelude_map = setup_resolver_for_prelude(&mut context, &mut resolver);
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        let mut context = setup_retrieve_context(&prelude_map);
        retrieve_file(&mut context, &file_3);
        context.type_checker.resolve();
        let var_types = context.type_checker.get_all();
        for (var_id, ty) in &var_types {
            eprintln!("{:?}: {:?}", var_id, ty);
        }
        assert_eq!(var_types.len(), 4);
        let file_4 = typing_file(&mut context, &file_3);

        let file_5 = felis_code_gen_prepared::prepare_code_gen_file(&file_4);

        let res = compile(file_5);
        let base_path = format!("/tmp/{}", test_name_to_file_name("felis_compile_test_2"));
        {
            let base_path = Path::new(&base_path);
            if base_path.exists() {
                std::fs::remove_dir_all(&base_path).unwrap();
            }
        }
        std::fs::create_dir(&base_path).unwrap();
        std::fs::write(format!("{}/main.s", base_path), &res).unwrap();

        let status = Command::new("as")
            .args(&[
                "-o",
                &format!("{}/main.o", base_path),
                &format!("{}/main.s", base_path),
            ])
            .status()
            .expect("");
        assert_eq!(status.code(), Some(0));
        let status = Command::new("ld")
            .args(&[
                "-o",
                &format!("{}/a.out", base_path),
                &format!("{}/main.o", base_path),
            ])
            .status()
            .expect("");
        assert_eq!(status.code(), Some(0));
        let output = Command::new(&format!("{}/a.out", base_path))
            .output()
            .expect("");
        assert_eq!(output.stdout, b"Hello, world!\n");
        assert_eq!(output.status.code(), Some(0));
    }
}
