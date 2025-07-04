use neco_felis_syn::*;
use std::collections::HashMap;

pub fn compile_to_assembly(file: &File<PhaseParse>) -> Result<String, CompileError> {
    let mut compiler = AssemblyCompiler::new();
    compiler.compile_file(file)
}

#[derive(Debug, Clone)]
pub enum CompileError {
    UnsupportedConstruct(String),
    EntrypointNotFound,
    InvalidSyscall,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnsupportedConstruct(msg) => write!(f, "Unsupported construct: {msg}"),
            CompileError::EntrypointNotFound => write!(f, "Entrypoint not found"),
            CompileError::InvalidSyscall => write!(f, "Invalid syscall"),
        }
    }
}

impl std::error::Error for CompileError {}

struct AssemblyCompiler {
    output: String,
    entrypoint: Option<String>,
    builtins: HashMap<String, String>,
}

impl AssemblyCompiler {
    fn new() -> Self {
        Self {
            output: String::new(),
            entrypoint: None,
            builtins: HashMap::new(),
        }
    }

    fn compile_file(&mut self, file: &File<PhaseParse>) -> Result<String, CompileError> {
        self.output.push_str(".intel_syntax noprefix\n");
        self.output.push_str(".section .text\n");
        self.output.push_str(".globl _start\n\n");

        for item in file.items() {
            self.compile_item(item)?;
        }

        if let Some(entrypoint) = &self.entrypoint {
            self.output.push_str("_start:\n");
            self.output.push_str(&format!("    call {entrypoint}\n"));
            self.output.push_str("    mov rax, 60\n");
            self.output.push_str("    mov rdi, 0\n");
            self.output.push_str("    syscall\n");
        } else {
            return Err(CompileError::EntrypointNotFound);
        }

        Ok(self.output.clone())
    }

    fn compile_item(&mut self, item: &Item<PhaseParse>) -> Result<(), CompileError> {
        match item {
            Item::Entrypoint(entrypoint) => {
                self.entrypoint = Some(entrypoint.name.s().to_string());
                Ok(())
            }
            Item::UseBuiltin(use_builtin) => {
                self.builtins.insert(
                    use_builtin.name.s().to_string(),
                    use_builtin.builtin_name.s().to_string(),
                );
                Ok(())
            }
            Item::Proc(proc) => self.compile_proc(proc),
            _ => Err(CompileError::UnsupportedConstruct(format!("{item:?}"))),
        }
    }

    fn compile_proc(&mut self, proc: &ItemProc<PhaseParse>) -> Result<(), CompileError> {
        self.output.push_str(&format!("{}:\n", proc.name.s()));
        self.compile_proc_block(&proc.proc_block)?;
        self.output.push_str("    ret\n\n");
        Ok(())
    }

    fn compile_proc_block(
        &mut self,
        block: &ItemProcBlock<PhaseParse>,
    ) -> Result<(), CompileError> {
        self.compile_statements(&block.statements)
    }

    fn compile_statements(
        &mut self,
        statements: &Statements<PhaseParse>,
    ) -> Result<(), CompileError> {
        match statements {
            Statements::Then(then) => {
                self.compile_term(&then.head)?;
                match &*then.tail {
                    Statements::Nil => Ok(()),
                    _ => self.compile_statements(&then.tail),
                }
            }
            Statements::Nil => Ok(()),
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "{statements:?}"
            ))),
        }
    }

    fn compile_term(&mut self, term: &Term<PhaseParse>) -> Result<(), CompileError> {
        match term {
            Term::Apply(apply) => self.compile_apply(apply),
            _ => Err(CompileError::UnsupportedConstruct(format!("{term:?}"))),
        }
    }

    fn compile_apply(&mut self, apply: &TermApply<PhaseParse>) -> Result<(), CompileError> {
        if let Term::Variable(var) = &*apply.f
            && let Some(builtin) = self.builtins.get(var.variable.s())
            && builtin == "syscall"
        {
            return self.compile_syscall(&apply.args);
        }
        Err(CompileError::UnsupportedConstruct(format!("{apply:?}")))
    }

    fn compile_syscall(&mut self, args: &[Term<PhaseParse>]) -> Result<(), CompileError> {
        if args.len() != 6 {
            return Err(CompileError::InvalidSyscall);
        }

        let registers = ["rax", "rdi", "rsi", "rdx", "r10", "r8"];

        for (i, arg) in args.iter().enumerate() {
            if let Term::Number(num) = arg {
                self.output
                    .push_str(&format!("    mov {}, {}\n", registers[i], num.number.s()));
            } else {
                return Err(CompileError::InvalidSyscall);
            }
        }

        self.output.push_str("    syscall\n");
        Ok(())
    }
}

pub fn compile_file_to_assembly(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file_id_generator = FileIdGenerator::new();
    let file_id = file_id_generator.generate_file_id();
    let source = std::fs::read_to_string(file_path)?;
    let tokens = Token::lex(&source, file_id);

    let mut i = 0;
    let file = File::parse(&tokens, &mut i)?.ok_or("Failed to parse file")?;

    let assembly = compile_to_assembly(&file)?;
    Ok(assembly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_exit_42() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/exit_42.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov rax, 231"));
        assert!(assembly.contains("mov rdi, 42"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }
}
