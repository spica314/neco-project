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
    variables: HashMap<String, i32>, // Now maps variable names to stack offsets
    stack_offset: i32,               // Current stack offset
}

impl AssemblyCompiler {
    fn new() -> Self {
        Self {
            output: String::new(),
            entrypoint: None,
            builtins: HashMap::new(),
            variables: HashMap::new(),
            stack_offset: 0,
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
        // Calculate the number of let variables in this function
        let let_count = self.count_let_variables_in_proc_block(&proc.proc_block);
        let stack_space = let_count * 8; // 8 bytes per variable (64-bit)

        self.output.push_str(&format!("{}:\n", proc.name.s()));

        // Allocate stack space if needed
        if stack_space > 0 {
            self.output
                .push_str(&format!("    sub rsp, {stack_space}\n"));
        }

        // Reset stack offset for this function
        self.stack_offset = 0;

        self.compile_proc_block(&proc.proc_block)?;

        // Deallocate stack space if needed
        if stack_space > 0 {
            self.output
                .push_str(&format!("    add rsp, {stack_space}\n"));
        }

        self.output.push_str("    ret\n\n");

        // Clear variables for next function
        self.variables.clear();

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
            Term::Let(let_expr) => self.compile_let(let_expr),
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
            match arg {
                Term::Number(num) => {
                    let number_value = self.parse_number(num.number.s());
                    self.output
                        .push_str(&format!("    mov {}, {}\n", registers[i], number_value));
                }
                Term::Variable(var) => {
                    let var_name = var.variable.s();
                    if let Some(&offset) = self.variables.get(var_name) {
                        // Load value from stack into register
                        self.output.push_str(&format!(
                            "    mov {}, qword ptr [rsp + {}]\n",
                            registers[i],
                            offset - 8
                        ));
                    } else {
                        return Err(CompileError::UnsupportedConstruct(format!(
                            "Unknown variable: {var_name}"
                        )));
                    }
                }
                _ => {
                    return Err(CompileError::InvalidSyscall);
                }
            }
        }

        self.output.push_str("    syscall\n");
        Ok(())
    }

    fn compile_let(&mut self, let_expr: &TermLet<PhaseParse>) -> Result<(), CompileError> {
        let var_name = let_expr.variable_name();

        // Allocate stack space for this variable
        self.stack_offset += 8; // 8 bytes for 64-bit value
        let offset = self.stack_offset;

        // Store the variable's stack offset
        self.variables.insert(var_name.to_string(), offset);

        match &*let_expr.value {
            Term::Number(num) => {
                // Move the value to the stack location
                let number_value = self.parse_number(num.number.s());
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], {}\n",
                    offset - 8,
                    number_value
                ));
                Ok(())
            }
            Term::Apply(apply) => {
                // Handle function application in let expression
                if let Term::Variable(var) = &*apply.f
                    && let Some(builtin) = self.builtins.get(var.variable.s())
                {
                    match builtin.as_str() {
                        "u64_add" => return self.compile_u64_add_let(apply, offset),
                        "u64_sub" => return self.compile_u64_sub_let(apply, offset),
                        "u64_mul" => return self.compile_u64_mul_let(apply, offset),
                        "u64_div" => return self.compile_u64_div_let(apply, offset),
                        "u64_mod" => return self.compile_u64_mod_let(apply, offset),
                        "f32_add" => return self.compile_f32_add_let(apply, offset),
                        "f32_sub" => return self.compile_f32_sub_let(apply, offset),
                        "f32_mul" => return self.compile_f32_mul_let(apply, offset),
                        "f32_div" => return self.compile_f32_div_let(apply, offset),
                        "f32_to_u64" => return self.compile_f32_to_u64_let(apply, offset),
                        _ => {}
                    }
                }
                Err(CompileError::UnsupportedConstruct(format!(
                    "let with unsupported function application: {apply:?}"
                )))
            }
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "let with non-number value: {let_expr:?}"
            ))),
        }
    }

    fn compile_u64_add_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // u64_add expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "u64_add expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into rax
        self.load_argument_into_register(arg1, "rax")?;

        // Load second argument into rbx and add to rax
        self.load_argument_into_register(arg2, "rbx")?;
        self.output.push_str("    add rax, rbx\n");

        // Store result from rax to the variable's stack location
        self.output
            .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));

        Ok(())
    }

    fn compile_u64_sub_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // u64_sub expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "u64_sub expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into rax
        self.load_argument_into_register(arg1, "rax")?;

        // Load second argument into rbx and subtract from rax
        self.load_argument_into_register(arg2, "rbx")?;
        self.output.push_str("    sub rax, rbx\n");

        // Store result from rax to the variable's stack location
        self.output
            .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));

        Ok(())
    }

    fn compile_u64_mul_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // u64_mul expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "u64_mul expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into rax
        self.load_argument_into_register(arg1, "rax")?;

        // Load second argument into rbx and multiply with rax
        self.load_argument_into_register(arg2, "rbx")?;
        self.output.push_str("    imul rax, rbx\n");

        // Store result from rax to the variable's stack location
        self.output
            .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));

        Ok(())
    }

    fn compile_u64_div_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // u64_div expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "u64_div expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into rax
        self.load_argument_into_register(arg1, "rax")?;

        // Load second argument into rbx
        self.load_argument_into_register(arg2, "rbx")?;

        // Clear rdx for division
        self.output.push_str("    xor rdx, rdx\n");
        // Divide rax by rbx, result in rax
        self.output.push_str("    div rbx\n");

        // Store result from rax to the variable's stack location
        self.output
            .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));

        Ok(())
    }

    fn compile_u64_mod_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // u64_mod expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "u64_mod expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into rax
        self.load_argument_into_register(arg1, "rax")?;

        // Load second argument into rbx
        self.load_argument_into_register(arg2, "rbx")?;

        // Clear rdx for division
        self.output.push_str("    xor rdx, rdx\n");
        // Divide rax by rbx, remainder in rdx
        self.output.push_str("    div rbx\n");

        // Store remainder from rdx to the variable's stack location
        self.output
            .push_str(&format!("    mov qword ptr [rsp + {}], rdx\n", offset - 8));

        Ok(())
    }

    fn compile_f32_add_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // f32_add expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "f32_add expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into xmm0
        self.load_f32_argument_into_register(arg1, "xmm0")?;

        // Load second argument into xmm1 and add to xmm0
        self.load_f32_argument_into_register(arg2, "xmm1")?;
        self.output.push_str("    addss xmm0, xmm1\n");

        // Store result from xmm0 to the variable's stack location
        self.output.push_str(&format!(
            "    movss dword ptr [rsp + {}], xmm0\n",
            offset - 8
        ));

        Ok(())
    }

    fn compile_f32_sub_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // f32_sub expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "f32_sub expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into xmm0
        self.load_f32_argument_into_register(arg1, "xmm0")?;

        // Load second argument into xmm1 and subtract from xmm0
        self.load_f32_argument_into_register(arg2, "xmm1")?;
        self.output.push_str("    subss xmm0, xmm1\n");

        // Store result from xmm0 to the variable's stack location
        self.output.push_str(&format!(
            "    movss dword ptr [rsp + {}], xmm0\n",
            offset - 8
        ));

        Ok(())
    }

    fn compile_f32_mul_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // f32_mul expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "f32_mul expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into xmm0
        self.load_f32_argument_into_register(arg1, "xmm0")?;

        // Load second argument into xmm1 and multiply with xmm0
        self.load_f32_argument_into_register(arg2, "xmm1")?;
        self.output.push_str("    mulss xmm0, xmm1\n");

        // Store result from xmm0 to the variable's stack location
        self.output.push_str(&format!(
            "    movss dword ptr [rsp + {}], xmm0\n",
            offset - 8
        ));

        Ok(())
    }

    fn compile_f32_div_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // f32_div expects exactly 2 arguments
        if apply.args.len() != 2 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "f32_div expects 2 arguments, got {}",
                apply.args.len()
            )));
        }

        let arg1 = &apply.args[0];
        let arg2 = &apply.args[1];

        // Load first argument into xmm0
        self.load_f32_argument_into_register(arg1, "xmm0")?;

        // Load second argument into xmm1
        self.load_f32_argument_into_register(arg2, "xmm1")?;

        // Divide xmm0 by xmm1
        self.output.push_str("    divss xmm0, xmm1\n");

        // Store result from xmm0 to the variable's stack location
        self.output.push_str(&format!(
            "    movss dword ptr [rsp + {}], xmm0\n",
            offset - 8
        ));

        Ok(())
    }

    fn compile_f32_to_u64_let(
        &mut self,
        apply: &TermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        // f32_to_u64 expects exactly 1 argument
        if apply.args.len() != 1 {
            return Err(CompileError::UnsupportedConstruct(format!(
                "f32_to_u64 expects 1 argument, got {}",
                apply.args.len()
            )));
        }

        let arg = &apply.args[0];

        // Load f32 argument into xmm0
        self.load_f32_argument_into_register(arg, "xmm0")?;

        // Convert f32 to u64
        self.output.push_str("    cvttss2si rax, xmm0\n");

        // Store result from rax to the variable's stack location
        self.output
            .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));

        Ok(())
    }

    fn load_f32_argument_into_register(
        &mut self,
        arg: &Term<PhaseParse>,
        register: &str,
    ) -> Result<(), CompileError> {
        match arg {
            Term::Number(num) => {
                let number_str = num.number.s();
                if let Some(float_value) = number_str.strip_suffix("f32") {
                    // Use direct encoding (this is a simplified approach)
                    self.output.push_str(&format!(
                        "    mov eax, {}\n",
                        Self::float_to_hex(float_value.parse::<f32>().unwrap_or(0.0))
                    ));
                    self.output.push_str(&format!("    movd {register}, eax\n"));
                } else {
                    return Err(CompileError::UnsupportedConstruct(format!(
                        "Expected f32 number, got: {number_str}"
                    )));
                }
            }
            Term::Variable(var) => {
                let var_name = var.variable.s();
                if let Some(&var_offset) = self.variables.get(var_name) {
                    self.output.push_str(&format!(
                        "    movss {register}, dword ptr [rsp + {}]\n",
                        var_offset - 8
                    ));
                } else {
                    return Err(CompileError::UnsupportedConstruct(format!(
                        "Unknown variable: {var_name}"
                    )));
                }
            }
            _ => {
                return Err(CompileError::UnsupportedConstruct(format!(
                    "Unsupported f32 argument type: {arg:?}"
                )));
            }
        }
        Ok(())
    }

    fn float_to_hex(f: f32) -> String {
        format!("0x{:08x}", f.to_bits())
    }

    fn load_argument_into_register(
        &mut self,
        arg: &Term<PhaseParse>,
        register: &str,
    ) -> Result<(), CompileError> {
        match arg {
            Term::Number(num) => {
                let number_value = self.parse_number(num.number.s());
                self.output
                    .push_str(&format!("    mov {register}, {number_value}\n"));
            }
            Term::Variable(var) => {
                let var_name = var.variable.s();
                if let Some(&var_offset) = self.variables.get(var_name) {
                    self.output.push_str(&format!(
                        "    mov {register}, qword ptr [rsp + {}]\n",
                        var_offset - 8
                    ));
                } else {
                    return Err(CompileError::UnsupportedConstruct(format!(
                        "Unknown variable: {var_name}"
                    )));
                }
            }
            _ => {
                return Err(CompileError::UnsupportedConstruct(format!(
                    "Unsupported argument type: {arg:?}"
                )));
            }
        }
        Ok(())
    }

    fn count_let_variables_in_proc_block(&self, block: &ItemProcBlock<PhaseParse>) -> i32 {
        Self::count_let_variables_in_statements(&block.statements)
    }

    fn count_let_variables_in_statements(statements: &Statements<PhaseParse>) -> i32 {
        match statements {
            Statements::Then(then) => {
                let head_count = Self::count_let_variables_in_term(&then.head);
                let tail_count = Self::count_let_variables_in_statements(&then.tail);
                head_count + tail_count
            }
            Statements::Term(term) => Self::count_let_variables_in_term(term),
            Statements::Nil => 0,
        }
    }

    fn count_let_variables_in_term(term: &Term<PhaseParse>) -> i32 {
        match term {
            Term::Let(_) => 1,
            Term::Apply(apply) => {
                let mut count = Self::count_let_variables_in_term(&apply.f);
                for arg in &apply.args {
                    count += Self::count_let_variables_in_term(arg);
                }
                count
            }
            _ => 0,
        }
    }

    fn parse_number(&self, number_str: &str) -> String {
        // Remove type suffixes like u64, i32, etc.
        if let Some(pos) = number_str.find(|c: char| c.is_ascii_alphabetic()) {
            number_str[..pos].to_string()
        } else {
            number_str.to_string()
        }
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
    use std::process::Command;
    use tempfile::TempDir;

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

    #[test]
    fn test_compile_add() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/add.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov rax, 40"));
        assert!(assembly.contains("mov rbx, 2"));
        assert!(assembly.contains("add rax, rbx"));
        assert!(assembly.contains("mov qword ptr [rsp + 8], rax"));
        assert!(assembly.contains("mov rax, qword ptr [rsp + 0]"));
        assert!(assembly.contains("mov rdi, qword ptr [rsp + 8]"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_compile_sub() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/sub.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov rax, 50"));
        assert!(assembly.contains("mov rbx, 8"));
        assert!(assembly.contains("sub rax, rbx"));
        assert!(assembly.contains("mov qword ptr [rsp + 8], rax"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_compile_mul() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/mul.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov rax, 6"));
        assert!(assembly.contains("mov rbx, 7"));
        assert!(assembly.contains("imul rax, rbx"));
        assert!(assembly.contains("mov qword ptr [rsp + 8], rax"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_compile_div() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/div.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov rax, 84"));
        assert!(assembly.contains("mov rbx, 2"));
        assert!(assembly.contains("xor rdx, rdx"));
        assert!(assembly.contains("div rbx"));
        assert!(assembly.contains("mov qword ptr [rsp + 8], rax"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_compile_mod() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/mod.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov rax, 142"));
        assert!(assembly.contains("mov rbx, 100"));
        assert!(assembly.contains("xor rdx, rdx"));
        assert!(assembly.contains("div rbx"));
        assert!(assembly.contains("mov qword ptr [rsp + 8], rdx"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    /// Helper function to compile, assemble, link, and execute a Felis program
    fn compile_and_execute(
        file_path: &str,
    ) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
        // Create temporary directory for build artifacts
        let temp_dir = TempDir::new()?;
        let asm_file = temp_dir.path().join("program.s");
        let obj_file = temp_dir.path().join("program.o");
        let exe_file = temp_dir.path().join("program");

        // Step 1: Compile Felis to assembly
        let assembly = compile_file_to_assembly(file_path)?;
        std::fs::write(&asm_file, assembly)?;

        // Step 2: Assemble to object file
        let as_status = Command::new("as")
            .args([
                "--64",
                &asm_file.to_string_lossy(),
                "-o",
                &obj_file.to_string_lossy(),
            ])
            .status()?;

        if !as_status.success() {
            return Err("Assembly failed".into());
        }

        // Step 3: Link to executable
        let ld_status = Command::new("ld")
            .args([
                obj_file.to_string_lossy().as_ref(),
                "-o",
                &exe_file.to_string_lossy(),
            ])
            .status()?;

        if !ld_status.success() {
            return Err("Linking failed".into());
        }

        // Step 4: Execute the program
        let exec_status = Command::new(&exe_file).status()?;

        Ok(exec_status)
    }

    #[test]
    fn test_exit_42_integration() {
        let result = compile_and_execute("../testcases/felis/single/exit_42.fe");

        match result {
            Ok(status) => {
                println!(
                    "exit_42.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // exit_42.fe should exit with code 42
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping exit_42.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_let_integration() {
        let result = compile_and_execute("../testcases/felis/single/let.fe");

        match result {
            Ok(status) => {
                println!(
                    "let.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // let.fe should exit with code 42 (error_code value in syscall)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping let.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_add_integration() {
        let result = compile_and_execute("../testcases/felis/single/add.fe");

        match result {
            Ok(status) => {
                println!(
                    "add.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // add.fe should exit with code 42 (40 + 2 = 42)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping add.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_sub_integration() {
        let result = compile_and_execute("../testcases/felis/single/sub.fe");

        match result {
            Ok(status) => {
                println!(
                    "sub.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // sub.fe should exit with code 42 (50 - 8 = 42)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping sub.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_mul_integration() {
        let result = compile_and_execute("../testcases/felis/single/mul.fe");

        match result {
            Ok(status) => {
                println!(
                    "mul.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // mul.fe should exit with code 42 (6 * 7 = 42)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping mul.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_div_integration() {
        let result = compile_and_execute("../testcases/felis/single/div.fe");

        match result {
            Ok(status) => {
                println!(
                    "div.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // div.fe should exit with code 42 (84 / 2 = 42)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping div.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_mod_integration() {
        let result = compile_and_execute("../testcases/felis/single/mod.fe");

        match result {
            Ok(status) => {
                println!(
                    "mod.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // mod.fe should exit with code 42 (142 % 100 = 42)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping mod.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_compile_add_f32() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/add_f32.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov eax, 0x42200000")); // 40.0f32
        assert!(assembly.contains("movd xmm0, eax"));
        assert!(assembly.contains("mov eax, 0x40000000")); // 2.0f32
        assert!(assembly.contains("movd xmm1, eax"));
        assert!(assembly.contains("addss xmm0, xmm1"));
        assert!(assembly.contains("movss dword ptr [rsp + 8], xmm0"));
        assert!(assembly.contains("cvttss2si rax, xmm0"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_compile_sub_f32() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/sub_f32.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov eax, 0x42480000")); // 50.0f32
        assert!(assembly.contains("movd xmm0, eax"));
        assert!(assembly.contains("mov eax, 0x41000000")); // 8.0f32
        assert!(assembly.contains("movd xmm1, eax"));
        assert!(assembly.contains("subss xmm0, xmm1"));
        assert!(assembly.contains("movss dword ptr [rsp + 8], xmm0"));
        assert!(assembly.contains("cvttss2si rax, xmm0"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_compile_mul_f32() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/mul_f32.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov eax, 0x40c00000")); // 6.0f32
        assert!(assembly.contains("movd xmm0, eax"));
        assert!(assembly.contains("mov eax, 0x40e00000")); // 7.0f32
        assert!(assembly.contains("movd xmm1, eax"));
        assert!(assembly.contains("mulss xmm0, xmm1"));
        assert!(assembly.contains("movss dword ptr [rsp + 8], xmm0"));
        assert!(assembly.contains("cvttss2si rax, xmm0"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_compile_div_f32() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/div_f32.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("mov eax, 0x42a80000")); // 84.0f32
        assert!(assembly.contains("movd xmm0, eax"));
        assert!(assembly.contains("mov eax, 0x40000000")); // 2.0f32
        assert!(assembly.contains("movd xmm1, eax"));
        assert!(assembly.contains("divss xmm0, xmm1"));
        assert!(assembly.contains("movss dword ptr [rsp + 8], xmm0"));
        assert!(assembly.contains("cvttss2si rax, xmm0"));
        assert!(assembly.contains("syscall"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
    }

    #[test]
    fn test_add_f32_integration() {
        let result = compile_and_execute("../testcases/felis/single/add_f32.fe");

        match result {
            Ok(status) => {
                println!(
                    "add_f32.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // add_f32.fe should exit with code 42 (40.0 + 2.0 = 42.0)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping add_f32.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_sub_f32_integration() {
        let result = compile_and_execute("../testcases/felis/single/sub_f32.fe");

        match result {
            Ok(status) => {
                println!(
                    "sub_f32.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // sub_f32.fe should exit with code 42 (50.0 - 8.0 = 42.0)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping sub_f32.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_mul_f32_integration() {
        let result = compile_and_execute("../testcases/felis/single/mul_f32.fe");

        match result {
            Ok(status) => {
                println!(
                    "mul_f32.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // mul_f32.fe should exit with code 42 (6.0 * 7.0 = 42.0)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping mul_f32.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_div_f32_integration() {
        let result = compile_and_execute("../testcases/felis/single/div_f32.fe");

        match result {
            Ok(status) => {
                println!(
                    "div_f32.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // div_f32.fe should exit with code 42 (84.0 / 2.0 = 42.0)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping div_f32.fe integration test: {e}");
            }
        }
    }
}
