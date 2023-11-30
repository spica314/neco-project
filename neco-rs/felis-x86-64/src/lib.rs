use felis_rename::{
    rename_defs::{DefId, RenameDefContext},
    rename_uses::RenameDecoration,
};
use felis_syn::{
    syn_expr::SynExpr,
    syn_file::{SynFile, SynFileItem},
    syn_statement::SynStatement,
};
use neco_resolver::Resolver;

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

pub fn compile(file: SynFile<RenameDecoration>) -> String {
    let mut res = String::new();

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
    res.push_str(
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

    let mut strings = vec![];

    // Proc
    for item in &file.items {
        let proc = match item {
            SynFileItem::ProcDef(proc) => proc,
            _ => continue,
        };

        res.push_str(format!("{}_{}:\n", proc.name.as_str(), proc.ext.id.as_usize()).as_str());

        for statement in &proc.proc_block.statements {
            match statement {
                SynStatement::Expr(SynExpr::App(app)) => {
                    let fun = app.exprs[0].clone();
                    let SynExpr::IdentWithPath(fun) = fun else {
                        panic!("fun = {:?}", fun)
                    };
                    let fun = fun.ident.as_str();
                    if fun == "__write_to_stdout" {
                        let arg = app.exprs[1].clone();
                        let SynExpr::String(s) = arg else { panic!() };
                        let s = s.token_string.string.clone();
                        let label = format!("__string_{}", strings.len());
                        strings.push((label.clone(), s.clone()));
                        res.push_str("    mov rax, 1\n");
                        res.push_str("    mov rdi, 1\n");
                        res.push_str(format!("    lea rsi, {}\n", label).as_str());
                        res.push_str(format!("    mov rdx, {}\n", string_length(&s)).as_str());
                        res.push_str("    syscall\n");
                    } else {
                        panic!();
                    }
                }
                _ => panic!(),
            }
        }

        res.push_str("    mov rax, 0\n");
        res.push_str("    ret\n");
    }

    // string literals
    res.push_str("    .section .data\n");
    for (label, s) in strings {
        res.push_str(format!("{}:\n", label).as_str());
        res.push_str(format!("    .asciz \"{}\"\n", s).as_str());
    }

    res
}

pub fn setup_resolver_for_prelude(context: &mut RenameDefContext, resolver: &mut Resolver<DefId>) {
    let id = context.new_id();
    resolver.set("__write_to_stdout".to_string(), id);
}

#[cfg(test)]
mod test {
    use std::{path::Path, process::Command};

    use super::*;
    use felis_rename::{
        path_table::construct_path_table_syn_file,
        rename_defs::{rename_defs_file, RenameDefContext},
        rename_uses::{rename_uses_file, RenameUseContext},
    };
    use felis_syn::{decoration::UD, syn_file::SynFile, test_utils::parse_from_str};
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
        setup_resolver_for_prelude(&mut context, &mut resolver);
        let mut context2 = RenameUseContext::new(resolver, path_table);
        let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

        let res = compile(file_3);
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
}