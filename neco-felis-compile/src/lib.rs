use neco_felis_syn::*;

// Module declarations
pub mod arithmetic;
pub mod arrays;
pub mod compile_options;
pub mod compiler;
pub mod control_flow;
pub mod error;

// Re-exports
pub use compiler::{ArrayInfo, AssemblyCompiler};
pub use error::CompileError;

use crate::compile_options::CompileOptions;

/// Main public API function to compile a file to assembly
pub fn compile_to_assembly(
    file: &File<PhaseParse>,
    compile_options: CompileOptions,
) -> Result<String, CompileError> {
    let mut compiler = AssemblyCompiler::new(compile_options);
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

        // Allocate stack space for the value variable
        self.stack_offset += 8; // 8 bytes for 64-bit value
        let value_offset = self.stack_offset;

        // Allocate stack space for the reference variable (to hold address)
        self.stack_offset += 8; // 8 bytes for 64-bit address
        let ref_offset = self.stack_offset;

        // Store the value variable's stack offset
        self.variables.insert(var_name.to_string(), value_offset);
        // Store the reference variable's stack offset (it holds the address of value_var)
        self.variables.insert(ref_var_name.to_string(), ref_offset);
        // Track that ref_var_name is a reference to var_name
        self.reference_variables
            .insert(ref_var_name.to_string(), var_name.to_string());

        match &*let_mut_expr.value {
            ProcTerm::Number(num) => {
                // Store the value in the value variable's location
                let number_value = self.parse_number(num.number.s());
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], {}\n",
                    value_offset - 8,
                    number_value
                ));

                // Store the address of the value variable in the reference variable
                self.output
                    .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], rax\n",
                    ref_offset - 8
                ));
                Ok(())
            }
            ProcTerm::Apply(apply) => {
                // Handle function application in let mut expression
                if let ProcTerm::Variable(var) = &*apply.f
                    && let Some(builtin) = self.builtins.get(var.variable.s())
                {
                    match builtin.as_str() {
                        "u64_add" => {
                            self.compile_u64_add_let_proc(apply, value_offset)?;
                            // Store the address of the value variable in the reference variable
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_sub" => {
                            self.compile_u64_sub_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_mul" => {
                            self.compile_u64_mul_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_div" => {
                            self.compile_u64_div_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_mod" => {
                            self.compile_u64_mod_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_add" => {
                            self.compile_f32_add_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_sub" => {
                            self.compile_f32_sub_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_mul" => {
                            self.compile_f32_mul_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_div" => {
                            self.compile_f32_div_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_to_u64" => {
                            self.compile_f32_to_u64_let_proc(apply, value_offset)?;
                            self.output
                                .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rsp + {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
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

                // Store the result (rax) to the value variable's stack location
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], rax\n",
                    value_offset - 8
                ));

                // Store the address of the value variable in the reference variable
                self.output
                    .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                self.output.push_str(&format!(
                    "    mov qword ptr [rsp + {}], rax\n",
                    ref_offset - 8
                ));
                Ok(())
            }
            ProcTerm::Variable(var) => {
                // Handle variable references in let mut expressions
                let var_name = var.variable.s();
                if let Some(&var_offset) = self.variables.get(var_name) {
                    // Load value from the source variable's stack location
                    self.output.push_str(&format!(
                        "    mov rax, qword ptr [rsp + {}]\n",
                        var_offset - 8
                    ));
                    // Store it to the value variable's stack location
                    self.output.push_str(&format!(
                        "    mov qword ptr [rsp + {}], rax\n",
                        value_offset - 8
                    ));

                    // Store the address of the value variable in the reference variable
                    self.output
                        .push_str(&format!("    lea rax, [rsp + {}]\n", value_offset - 8));
                    self.output.push_str(&format!(
                        "    mov qword ptr [rsp + {}], rax\n",
                        ref_offset - 8
                    ));
                    Ok(())
                } else {
                    Err(CompileError::UnsupportedConstruct(format!(
                        "let mut with undefined variable: {var_name}"
                    )))
                }
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

        // Check if this is a reference variable
        let is_reference = self.reference_variables.contains_key(var_name);

        match &*assign_expr.value {
            ProcTerm::Number(num) => {
                let number_value = self.parse_number(num.number.s());
                if is_reference {
                    // This is a reference variable - load the address and store to that location
                    self.output
                        .push_str(&format!("    mov rax, qword ptr [rsp + {}]\n", offset - 8));
                    self.output
                        .push_str(&format!("    mov qword ptr [rax], {number_value}\n"));
                } else {
                    // Regular variable - store directly
                    self.output.push_str(&format!(
                        "    mov qword ptr [rsp + {}], {}\n",
                        offset - 8,
                        number_value
                    ));
                }
                Ok(())
            }
            ProcTerm::Apply(apply) => {
                // Handle function application in assignment
                if let ProcTerm::Variable(var) = &*apply.f
                    && let Some(builtin) = self.builtins.get(var.variable.s())
                {
                    match builtin.as_str() {
                        "u64_add" => {
                            if is_reference {
                                // For reference variables, compute in rax then store via indirect addressing
                                if apply.args.len() != 2 {
                                    return Err(CompileError::UnsupportedConstruct(format!(
                                        "u64_add expects 2 arguments, got {}",
                                        apply.args.len()
                                    )));
                                }

                                // Load first argument into rax
                                match &apply.args[0] {
                                    ProcTerm::Number(num) => {
                                        let value = self.parse_number(num.number.s());
                                        self.output.push_str(&format!("    mov rax, {value}\n"));
                                    }
                                    ProcTerm::Variable(var) => {
                                        if let Some(&var_offset) =
                                            self.variables.get(var.variable.s())
                                        {
                                            self.output.push_str(&format!(
                                                "    mov rax, qword ptr [rsp + {}]\n",
                                                var_offset - 8
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(CompileError::UnsupportedConstruct(
                                            "Unsupported first argument in u64_add".to_string(),
                                        ));
                                    }
                                }

                                // Load second argument into rbx and add
                                match &apply.args[1] {
                                    ProcTerm::Number(num) => {
                                        let value = self.parse_number(num.number.s());
                                        self.output.push_str(&format!("    mov rbx, {value}\n"));
                                    }
                                    ProcTerm::Variable(var) => {
                                        if let Some(&var_offset) =
                                            self.variables.get(var.variable.s())
                                        {
                                            self.output.push_str(&format!(
                                                "    mov rbx, qword ptr [rsp + {}]\n",
                                                var_offset - 8
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(CompileError::UnsupportedConstruct(
                                            "Unsupported second argument in u64_add".to_string(),
                                        ));
                                    }
                                }

                                self.output.push_str("    add rax, rbx\n");

                                // Store result via reference
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_u64_add_assign_proc(apply, offset);
                            }
                        }
                        "u64_sub" => {
                            if is_reference {
                                self.compile_u64_sub_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_u64_sub_assign_proc(apply, offset);
                            }
                        }
                        "u64_mul" => {
                            if is_reference {
                                self.compile_u64_mul_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_u64_mul_assign_proc(apply, offset);
                            }
                        }
                        "u64_div" => {
                            if is_reference {
                                self.compile_u64_div_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_u64_div_assign_proc(apply, offset);
                            }
                        }
                        "u64_mod" => {
                            if is_reference {
                                self.compile_u64_mod_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_u64_mod_assign_proc(apply, offset);
                            }
                        }
                        "f32_add" => {
                            if is_reference {
                                self.compile_f32_add_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_f32_add_assign_proc(apply, offset);
                            }
                        }
                        "f32_sub" => {
                            if is_reference {
                                self.compile_f32_sub_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_f32_sub_assign_proc(apply, offset);
                            }
                        }
                        "f32_mul" => {
                            if is_reference {
                                self.compile_f32_mul_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_f32_mul_assign_proc(apply, offset);
                            }
                        }
                        "f32_div" => {
                            if is_reference {
                                self.compile_f32_div_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_f32_div_assign_proc(apply, offset);
                            }
                        }
                        "f32_to_u64" => {
                            if is_reference {
                                self.compile_f32_to_u64_assign_proc(apply, offset)?;
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str("    mov qword ptr [rbx], rax\n");
                                return Ok(());
                            } else {
                                return self.compile_f32_to_u64_assign_proc(apply, offset);
                            }
                        }
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
    if i != tokens.len() {
        return Err(format!("Failed to parse file. token at {} / {}", i, tokens.len()).into());
    }

    let compile_options = CompileOptions { use_ptx: false };
    let assembly = compile_to_assembly(&file, compile_options)?;
    Ok(assembly)
}

pub fn compile_file_to_assembly_with_ptx(
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut file_id_generator = FileIdGenerator::new();
    let file_id = file_id_generator.generate_file_id();
    let source = std::fs::read_to_string(file_path)?;
    let tokens = Token::lex(&source, file_id);

    let mut i = 0;
    let file = File::parse(&tokens, &mut i)?.ok_or("Failed to parse file")?;

    let compile_options = CompileOptions { use_ptx: true };
    let assembly = compile_to_assembly(&file, compile_options)?;
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
mod tests;
