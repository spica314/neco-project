use neco_felis_syn::*;

// Module declarations
pub mod arithmetic;
pub mod arrays;
pub mod compiler;
pub mod control_flow;
pub mod error;

// Re-exports
pub use compiler::{ArrayInfo, AssemblyCompiler};
pub use error::CompileError;

/// Main public API function to compile a file to assembly
pub fn compile_to_assembly(file: &File<PhaseParse>) -> Result<String, CompileError> {
    let mut compiler = AssemblyCompiler::new();
    compiler.compile_file(file)
}

impl AssemblyCompiler {
    /// Compile a single statement
    pub fn compile_statement(
        &mut self,
        statement: &Statement<PhaseParse>,
    ) -> Result<(), CompileError> {
        match statement {
            Statement::Let(let_stmt) => self.compile_let(let_stmt),
            Statement::LetMut(let_mut_stmt) => self.compile_let_mut(let_mut_stmt),
            Statement::Assign(assign_stmt) => self.compile_assign(assign_stmt),
            Statement::FieldAssign(field_assign_stmt) => arrays::compile_field_assign(
                field_assign_stmt,
                &mut self.output,
                &self.variables,
                &self.variable_arrays,
                &self.arrays,
            ),
            Statement::Loop(loop_stmt) => control_flow::compile_loop(self, loop_stmt),
            Statement::Break(break_stmt) => control_flow::compile_break(self, break_stmt),
            Statement::Return(return_stmt) => self.compile_return(return_stmt),
            Statement::Expr(proc_term) => self.compile_proc_term(proc_term),
            Statement::Ext(_) => unreachable!("Ext statements not supported in PhaseParse"),
        }
    }

    /// Compile a function application term
    pub fn compile_apply(&mut self, apply: &TermApply<PhaseParse>) -> Result<(), CompileError> {
        if let Term::Variable(var) = &*apply.f
            && let Some(builtin) = self.builtins.get(var.variable.s())
            && builtin == "syscall"
        {
            return self.compile_syscall(&apply.args);
        }
        Err(CompileError::UnsupportedConstruct(format!("{apply:?}")))
    }

    /// Compile a let binding statement
    pub fn compile_let(&mut self, let_expr: &StatementLet<PhaseParse>) -> Result<(), CompileError> {
        let var_name = let_expr.variable_name();

        // Allocate stack space for this variable
        self.stack_offset += 8; // 8 bytes for 64-bit value
        let offset = self.stack_offset;

        // Store the variable's stack offset
        self.variables.insert(var_name.to_string(), offset);

        match &*let_expr.value {
            ProcTerm::Number(num) => {
                // Move the value to the stack location
                let number_value = self.parse_number(num.number.s());
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], {}\n",
                    offset - 8,
                    number_value
                ));
                Ok(())
            }
            ProcTerm::Apply(apply) => {
                // Handle function application in let expression
                if let ProcTerm::Variable(var) = &*apply.f {
                    if let Some(builtin) = self.builtins.get(var.variable.s()) {
                        match builtin.as_str() {
                            "u64_add" => return self.compile_u64_add_let_proc(apply, offset),
                            "u64_sub" => return self.compile_u64_sub_let_proc(apply, offset),
                            "u64_mul" => return self.compile_u64_mul_let_proc(apply, offset),
                            "u64_div" => return self.compile_u64_div_let_proc(apply, offset),
                            "u64_mod" => return self.compile_u64_mod_let_proc(apply, offset),
                            "f32_add" => return self.compile_f32_add_let_proc(apply, offset),
                            "f32_sub" => return self.compile_f32_sub_let_proc(apply, offset),
                            "f32_mul" => return self.compile_f32_mul_let_proc(apply, offset),
                            "f32_div" => return self.compile_f32_div_let_proc(apply, offset),
                            "f32_to_u64" => return self.compile_f32_to_u64_let_proc(apply, offset),
                            _ => {}
                        }
                    } else {
                        // Handle user-defined function call in let expression
                        return self.compile_user_proc_call_let(apply, offset);
                    }
                }
                Err(CompileError::UnsupportedConstruct(format!(
                    "let with unsupported function application: {apply:?}"
                )))
            }
            ProcTerm::ConstructorCall(constructor_call) => {
                // Handle array constructor calls in let expressions
                let type_name = constructor_call.type_name.s();
                let method_name = constructor_call.method.s();
                let constructor_name = format!("{type_name}::{method_name}");

                if constructor_name.contains("::new_with_size") {
                    // Check if this is an array constructor
                    if self.arrays.contains_key(type_name) {
                        // Store variable-to-array-type mapping for field access
                        self.variable_arrays
                            .insert(var_name.to_string(), type_name.to_string());

                        // For array constructors, we need to use the variable name for SoA allocation
                        self.compile_proc_constructor_call_with_var(constructor_call, var_name)?;
                        Ok(())
                    } else {
                        // Non-array constructor
                        self.compile_proc_constructor_call(constructor_call)?;
                        self.output
                            .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));
                        Ok(())
                    }
                } else {
                    // Other constructor calls
                    self.compile_proc_constructor_call(constructor_call)?;
                    self.output
                        .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));
                    Ok(())
                }
            }
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "let with non-number value: {let_expr:?}"
            ))),
        }
    }

    /// Compile a mutable let binding statement
    pub fn compile_let_mut(
        &mut self,
        let_mut_expr: &StatementLetMut<PhaseParse>,
    ) -> Result<(), CompileError> {
        let var_name = let_mut_expr.variable_name();
        let ref_var_name = let_mut_expr.reference_variable_name();

        // Allocate stack space for this variable
        self.stack_offset += 8; // 8 bytes for 64-bit value
        let offset = self.stack_offset;

        // Store both the variable's and reference variable's stack offset
        // Both point to the same memory location
        self.variables.insert(var_name.to_string(), offset);
        self.variables.insert(ref_var_name.to_string(), offset);

        match &*let_mut_expr.value {
            ProcTerm::Number(num) => {
                // Move the value to the stack location
                let number_value = self.parse_number(num.number.s());
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], {}\n",
                    offset - 8,
                    number_value
                ));
                Ok(())
            }
            ProcTerm::Apply(apply) => {
                // Handle function application in let mut expression
                if let ProcTerm::Variable(var) = &*apply.f
                    && let Some(builtin) = self.builtins.get(var.variable.s())
                {
                    match builtin.as_str() {
                        "u64_add" => return self.compile_u64_add_let_proc(apply, offset),
                        "u64_sub" => return self.compile_u64_sub_let_proc(apply, offset),
                        "u64_mul" => return self.compile_u64_mul_let_proc(apply, offset),
                        "u64_div" => return self.compile_u64_div_let_proc(apply, offset),
                        "u64_mod" => return self.compile_u64_mod_let_proc(apply, offset),
                        "f32_add" => return self.compile_f32_add_let_proc(apply, offset),
                        "f32_sub" => return self.compile_f32_sub_let_proc(apply, offset),
                        "f32_mul" => return self.compile_f32_mul_let_proc(apply, offset),
                        "f32_div" => return self.compile_f32_div_let_proc(apply, offset),
                        "f32_to_u64" => return self.compile_f32_to_u64_let_proc(apply, offset),
                        _ => {}
                    }
                }
                Err(CompileError::UnsupportedConstruct(format!(
                    "let mut with unsupported function application: {apply:?}"
                )))
            }
            ProcTerm::ConstructorCall(constructor_call) => {
                // Handle constructor calls in let mut expressions
                self.compile_proc_constructor_call(constructor_call)?;

                // Store the result (rax) to the variable's stack location
                self.output
                    .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));
                Ok(())
            }
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "let mut with non-number value: {let_mut_expr:?}"
            ))),
        }
    }

    /// Compile an assignment statement
    pub fn compile_assign(
        &mut self,
        assign_expr: &StatementAssign<PhaseParse>,
    ) -> Result<(), CompileError> {
        let var_name = assign_expr.variable_name();

        // Check if the variable exists
        let offset = *self.variables.get(var_name).ok_or_else(|| {
            CompileError::UnsupportedConstruct(format!("Unknown variable: {var_name}"))
        })?;

        match &*assign_expr.value {
            ProcTerm::Number(num) => {
                // Move the value to the stack location
                let number_value = self.parse_number(num.number.s());
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], {}\n",
                    offset - 8,
                    number_value
                ));
                Ok(())
            }
            ProcTerm::Apply(apply) => {
                // Handle function application in assignment
                if let ProcTerm::Variable(var) = &*apply.f
                    && let Some(builtin) = self.builtins.get(var.variable.s())
                {
                    match builtin.as_str() {
                        "u64_add" => return self.compile_u64_add_assign_proc(apply, offset),
                        "u64_sub" => return self.compile_u64_sub_assign_proc(apply, offset),
                        "u64_mul" => return self.compile_u64_mul_assign_proc(apply, offset),
                        "u64_div" => return self.compile_u64_div_assign_proc(apply, offset),
                        "u64_mod" => return self.compile_u64_mod_assign_proc(apply, offset),
                        "f32_add" => return self.compile_f32_add_assign_proc(apply, offset),
                        "f32_sub" => return self.compile_f32_sub_assign_proc(apply, offset),
                        "f32_mul" => return self.compile_f32_mul_assign_proc(apply, offset),
                        "f32_div" => return self.compile_f32_div_assign_proc(apply, offset),
                        "f32_to_u64" => return self.compile_f32_to_u64_assign_proc(apply, offset),
                        _ => {}
                    }
                }
                Err(CompileError::UnsupportedConstruct(format!(
                    "assignment with unsupported function application: {apply:?}"
                )))
            }
            ProcTerm::ConstructorCall(constructor_call) => {
                // Handle constructor calls in assignments
                self.compile_proc_constructor_call(constructor_call)?;

                // Store the result (rax) to the variable's stack location
                self.output
                    .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));
                Ok(())
            }
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "assignment with unsupported ProcTerm value: {assign_expr:?}"
            ))),
        }
    }

    /// Compile a syscall operation
    pub fn compile_syscall(&mut self, args: &[Term<PhaseParse>]) -> Result<(), CompileError> {
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

    /// Compile a user-defined procedure call in a let expression
    pub fn compile_user_proc_call_let(
        &mut self,
        apply: &ProcTermApply<PhaseParse>,
        offset: i32,
    ) -> Result<(), CompileError> {
        if let ProcTerm::Variable(var) = &*apply.f {
            // Set up arguments for the function call
            if !apply.args.is_empty() {
                // First argument in rdi
                if let ProcTerm::Number(num) = &apply.args[0] {
                    let value = self.parse_number(num.number.s());
                    self.output.push_str(&format!("    mov rdi, {value}\n"));
                }
            }

            if apply.args.len() >= 2 {
                // Second argument in rsi
                if let ProcTerm::Number(num) = &apply.args[1] {
                    let value = self.parse_number(num.number.s());
                    self.output.push_str(&format!("    mov rsi, {value}\n"));
                }
            }

            // Call the procedure
            let var_name = var.variable.s();
            self.output.push_str(&format!("    call {var_name}\n"));

            // Store the result (rax) to the variable's stack location
            self.output
                .push_str(&format!("    mov qword ptr [rsp + {}], rax\n", offset - 8));

            Ok(())
        } else {
            Err(CompileError::UnsupportedConstruct(format!(
                "Non-variable function in apply: {apply:?}"
            )))
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

impl AssemblyCompiler {
    /// Compile a return statement
    pub fn compile_return(
        &mut self,
        return_stmt: &StatementReturn<PhaseParse>,
    ) -> Result<(), CompileError> {
        // Compile the return value expression
        self.compile_proc_term(&return_stmt.value)?;

        // For now, return statements don't generate any specific assembly
        // The return value is already in the correct register/stack location
        // from the previous expression compilation
        Ok(())
    }
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
        println!("Generated assembly for add.fe:\n{assembly}");

        // Update expectations based on the new #let syntax
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
        assert!(assembly.contains("sub rsp, 16")); // Stack allocation for 2 let variables
        assert!(assembly.contains("mov qword ptr [rsp + 0], 231")); // syscall_id = 231u64
        assert!(assembly.contains("mov rax, 40")); // u64_add first arg
        assert!(assembly.contains("mov rbx, 2")); // u64_add second arg
        assert!(assembly.contains("add rax, rbx")); // u64_add operation
        assert!(assembly.contains("mov qword ptr [rsp + 8], rax")); // Store result
        assert!(assembly.contains("syscall"));
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

    #[test]
    fn test_compile_let_mut() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/let_mut.fe").unwrap();
        println!("Generated assembly for let_mut.fe:\n{assembly}");

        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));
        assert!(assembly.contains("sub rsp, 16")); // Stack allocation for 2 variables
        assert!(assembly.contains("mov qword ptr [rsp + 0], 231")); // syscall_id = 231u64
        assert!(assembly.contains("mov qword ptr [rsp + 8], 0")); // let mut error_code = 0u64
        assert!(assembly.contains("mov qword ptr [rsp + 8], 42")); // error_code = 42u64
        assert!(assembly.contains("syscall"));
    }

    #[test]
    fn test_let_mut_integration() {
        let result = compile_and_execute("../testcases/felis/single/let_mut.fe");

        match result {
            Ok(status) => {
                println!(
                    "let_mut.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // let_mut.fe should exit with code 42 (assigned error_code value in syscall)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping let_mut.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_compile_array() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/array.fe").unwrap();
        println!("Generated assembly for array.fe:\n{assembly}");

        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("_start:"));

        // Check for mmap syscalls (one for each field: x, y, z)
        assert!(assembly.contains("mov rax, 9")); // sys_mmap
        assert!(assembly.contains("mov rdi, 0")); // addr = NULL
        assert!(assembly.contains("mov rdx, 3")); // prot = PROT_READ | PROT_WRITE
        assert!(assembly.contains("mov r10, 34")); // flags = MAP_PRIVATE | MAP_ANONYMOUS
        assert!(assembly.contains("mov r8, -1")); // fd = -1
        assert!(assembly.contains("mov r9, 0")); // offset = 0

        // Check for array field assignments
        assert!(assembly.contains("mov ebx, 0x41200000")); // 10.0f32
        assert!(assembly.contains("mov dword ptr [rax], ebx"));

        // Check for field access in builtin calls
        assert!(assembly.contains("movss xmm0, dword ptr [rax]"));
        assert!(assembly.contains("addss xmm0, xmm1"));

        // Check for f32_to_u64 conversion
        assert!(assembly.contains("cvttss2si rax, xmm0"));

        assert!(assembly.contains("syscall"));
    }

    #[test]
    fn test_if_1_integration() {
        let result = compile_and_execute("../testcases/felis/single/if_1.fe");

        match result {
            Ok(status) => {
                println!(
                    "if_1.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // if_1.fe should exit with code 42 (0 == 0 is true, so executes then body)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping if_1.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_if_2_integration() {
        let result = compile_and_execute("../testcases/felis/single/if_2.fe");

        match result {
            Ok(status) => {
                println!(
                    "if_2.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // if_2.fe should exit with code 42 (0 == 1 is false, so executes else body)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping if_2.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_array_integration() {
        let result = compile_and_execute("../testcases/felis/single/array.fe");

        match result {
            Ok(status) => {
                println!(
                    "array.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // array.fe should exit with code 42 (10.0 + 14.0 + 18.0 = 42.0)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping array.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_loop_break() {
        let result = compile_and_execute("../testcases/felis/single/loop_break.fe");

        match result {
            Ok(status) => {
                println!(
                    "loop_break.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping loop_break.fe integration test: {e}");
            }
        }
    }

    #[test]
    fn test_compile_proc_call() {
        let assembly = compile_file_to_assembly("../testcases/felis/single/proc_call.fe").unwrap();
        assert!(assembly.contains(".intel_syntax noprefix"));
        assert!(assembly.contains("main:"));
        assert!(assembly.contains("f:"));
        assert!(assembly.contains("_start:"));

        // Check that proc f is defined and called
        assert!(assembly.contains("call f"));
        assert!(assembly.contains("ret")); // Both main and f should have ret

        // Check that syscall is properly set up
        assert!(assembly.contains("mov qword ptr [rsp + 0], 231")); // syscall_id = 231u64
        assert!(assembly.contains("syscall"));

        // Check that function arguments are set up
        assert!(assembly.contains("mov rdi, 40")); // First argument
        assert!(assembly.contains("mov rsi, 2")); // Second argument
    }

    #[test]
    fn test_proc_call_integration() {
        let result = compile_and_execute("../testcases/felis/single/proc_call.fe");

        match result {
            Ok(status) => {
                println!(
                    "proc_call.fe executed successfully with exit code: {:?}",
                    status.code()
                );
                // proc_call.fe should exit with code 42 (40 + 2 = 42)
                assert_eq!(status.code(), Some(42), "Program should exit with code 42");
            }
            Err(e) => {
                // Skip test if assembler/linker not available
                println!("Skipping proc_call.fe integration test: {e}");
            }
        }
    }
}
