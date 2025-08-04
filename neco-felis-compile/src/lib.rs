use neco_felis_syn::*;

// Module declarations
pub mod arithmetic;
pub mod arrays;
pub mod compile_options;
pub mod compiler;
pub mod control_flow;
pub mod error;
pub mod ptx;
pub mod statement;
pub mod syscall;

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
            Statement::CallPtx(call_ptx_stmt) => self.compile_call_ptx(call_ptx_stmt),
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
                    "    mov qword ptr [rbp - 8 - {}], {}\n",
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
                            "u64_to_f32" => return self.compile_u64_to_f32_let_proc(apply, offset),
                            "u64" => return self.compile_u64_let_proc(apply, offset),
                            "f32" => return self.compile_f32_let_proc(apply, offset),
                            "ctaid_x" => return self.compile_ctaid_x_let_proc(offset),
                            "ntid_x" => return self.compile_ntid_x_let_proc(offset),
                            "tid_x" => return self.compile_tid_x_let_proc(offset),
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
                        self.output.push_str(&format!(
                            "    mov qword ptr [rbp - 8 - {}], rax\n",
                            offset - 8
                        ));
                        Ok(())
                    }
                } else {
                    // Other constructor calls
                    self.compile_proc_constructor_call(constructor_call)?;
                    self.output.push_str(&format!(
                        "    mov qword ptr [rbp - 8 - {}], rax\n",
                        offset - 8
                    ));
                    Ok(())
                }
            }
            ProcTerm::Dereference(dereference) => {
                // Handle dereference in let expression: #let a = points .x 0 .*;
                // First compile the term that produces a reference (field access with index)
                self.compile_proc_term(&dereference.term)?;

                // Now rax contains the address, dereference it to get the value
                // For f32 values, we need to load it properly
                self.output.push_str("    mov rax, qword ptr [rax]\n");

                // Store the dereferenced value in the let variable's stack slot
                self.output.push_str(&format!(
                    "    mov qword ptr [rbp - 8 - {}], rax\n",
                    offset - 8
                ));
                Ok(())
            }
            ProcTerm::Paren(paren) => {
                // Handle parenthesized expressions in let: #let x = (expr);
                // Recursively compile the expression inside the parentheses
                match &*paren.proc_term {
                    ProcTerm::Dereference(dereference) => {
                        // Handle dereference inside parentheses: #let x = (ps .r 0 .*);
                        self.compile_proc_term(&dereference.term)?;

                        // Now rax contains the address, dereference it to get the value
                        self.output.push_str("    mov rax, qword ptr [rax]\n");

                        // Store the dereferenced value in the let variable's stack slot
                        self.output.push_str(&format!(
                            "    mov qword ptr [rbp - 8 - {}], rax\n",
                            offset - 8
                        ));
                        Ok(())
                    }
                    _ => {
                        // For other expressions inside parentheses, compile them and store result
                        self.compile_proc_term(&paren.proc_term)?;

                        // Store the result in the let variable's stack slot
                        self.output.push_str(&format!(
                            "    mov qword ptr [rbp - 8 - {}], rax\n",
                            offset - 8
                        ));
                        Ok(())
                    }
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
                    "    mov qword ptr [rbp - 8 - {}], {}\n",
                    value_offset - 8,
                    number_value
                ));

                // Store the address of the value variable in the reference variable
                self.output
                    .push_str(&format!("    lea rax, [rbp - 8 - {}]\n", value_offset - 8));
                self.output.push_str(&format!(
                    "    mov qword ptr [rbp - 8 - {}], rax\n",
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
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_sub" => {
                            self.compile_u64_sub_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_mul" => {
                            self.compile_u64_mul_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_div" => {
                            self.compile_u64_div_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_mod" => {
                            self.compile_u64_mod_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_add" => {
                            self.compile_f32_add_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_sub" => {
                            self.compile_f32_sub_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_mul" => {
                            self.compile_f32_mul_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_div" => {
                            self.compile_f32_div_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32_to_u64" => {
                            self.compile_f32_to_u64_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64_to_f32" => {
                            self.compile_u64_to_f32_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "u64" => {
                            self.compile_u64_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "f32" => {
                            self.compile_f32_let_proc(apply, value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "ctaid_x" => {
                            self.compile_ctaid_x_let_proc(value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "ntid_x" => {
                            self.compile_ntid_x_let_proc(value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
                                ref_offset - 8
                            ));
                            return Ok(());
                        }
                        "tid_x" => {
                            self.compile_tid_x_let_proc(value_offset)?;
                            self.output.push_str(&format!(
                                "    lea rax, [rbp - 8 - {}]\n",
                                value_offset - 8
                            ));
                            self.output.push_str(&format!(
                                "    mov qword ptr [rbp - 8 - {}], rax\n",
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
                    "    mov qword ptr [rbp - 8 - {}], rax\n",
                    value_offset - 8
                ));

                // Store the address of the value variable in the reference variable
                self.output
                    .push_str(&format!("    lea rax, [rbp - 8 - {}]\n", value_offset - 8));
                self.output.push_str(&format!(
                    "    mov qword ptr [rbp - 8 - {}], rax\n",
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
                        "    mov rax, qword ptr [rbp - 8 - {}]\n",
                        var_offset - 8
                    ));
                    // Store it to the value variable's stack location
                    self.output.push_str(&format!(
                        "    mov qword ptr [rbp - 8 - {}], rax\n",
                        value_offset - 8
                    ));

                    // Store the address of the value variable in the reference variable
                    self.output
                        .push_str(&format!("    lea rax, [rbp - 8 - {}]\n", value_offset - 8));
                    self.output.push_str(&format!(
                        "    mov qword ptr [rbp - 8 - {}], rax\n",
                        ref_offset - 8
                    ));
                    Ok(())
                } else {
                    Err(CompileError::UnsupportedConstruct(format!(
                        "let mut with undefined variable: {var_name}"
                    )))
                }
            }
            ProcTerm::Dereference(dereference) => {
                // Handle dereference in let mut expression: #let mut a = points .x 0 .*;
                // First compile the term that produces a reference (field access with index)
                self.compile_proc_term(&dereference.term)?;

                // Now rax contains the address, dereference it to get the value
                self.output.push_str("    mov rax, qword ptr [rax]\n");

                // Store the dereferenced value in the value variable's stack slot
                self.output.push_str(&format!(
                    "    mov qword ptr [rbp - 8 - {}], rax\n",
                    value_offset - 8
                ));

                // Store the address of the value variable in the reference variable
                self.output
                    .push_str(&format!("    lea rax, [rbp - 8 - {}]\n", value_offset - 8));
                self.output.push_str(&format!(
                    "    mov qword ptr [rbp - 8 - {}], rax\n",
                    ref_offset - 8
                ));
                Ok(())
            }
            ProcTerm::Paren(paren) => {
                // Handle parenthesized expressions in let mut: #let mut x = (expr);
                match &*paren.proc_term {
                    ProcTerm::Dereference(dereference) => {
                        // Handle dereference inside parentheses: #let mut x = (ps .r 0 .*);
                        self.compile_proc_term(&dereference.term)?;

                        // Now rax contains the address, dereference it to get the value
                        self.output.push_str("    mov rax, qword ptr [rax]\n");

                        // Store the dereferenced value in the value variable's stack slot
                        self.output.push_str(&format!(
                            "    mov qword ptr [rbp - 8 - {}], rax\n",
                            value_offset - 8
                        ));

                        // Store the address of the value variable in the reference variable
                        self.output
                            .push_str(&format!("    lea rax, [rbp - 8 - {}]\n", value_offset - 8));
                        self.output.push_str(&format!(
                            "    mov qword ptr [rbp - 8 - {}], rax\n",
                            ref_offset - 8
                        ));
                        Ok(())
                    }
                    _ => {
                        // For other expressions inside parentheses, compile them and store result
                        self.compile_proc_term(&paren.proc_term)?;

                        // Store the result in the value variable's stack slot
                        self.output.push_str(&format!(
                            "    mov qword ptr [rbp - 8 - {}], rax\n",
                            value_offset - 8
                        ));

                        // Store the address of the value variable in the reference variable
                        self.output
                            .push_str(&format!("    lea rax, [rbp - 8 - {}]\n", value_offset - 8));
                        self.output.push_str(&format!(
                            "    mov qword ptr [rbp - 8 - {}], rax\n",
                            ref_offset - 8
                        ));
                        Ok(())
                    }
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
                    self.output.push_str(&format!(
                        "    mov rax, qword ptr [rbp - 8 - {}]\n",
                        offset - 8
                    ));
                    self.output
                        .push_str(&format!("    mov qword ptr [rax], {number_value}\n"));
                } else {
                    // Regular variable - store directly
                    self.output.push_str(&format!(
                        "    mov qword ptr [rbp - 8 - {}], {}\n",
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
                                                "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                                "    mov rbx, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                                    "    mov rbx, qword ptr [rbp - 8 - {}]\n",
                                    offset - 8
                                ));
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rbp - 8 - {}]\n",
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
                self.output.push_str(&format!(
                    "    mov qword ptr [rbp - 8 - {}], rax\n",
                    offset - 8
                ));
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
                            "    mov {}, qword ptr [rbp - 8 - {}]\n",
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
            self.output.push_str(&format!(
                "    mov qword ptr [rbp - 8 - {}], rax\n",
                offset - 8
            ));

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
    if i != tokens.len() {
        return Err(format!("Failed to parse file. token at {} / {}", i, tokens.len()).into());
    }

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

    /// Compile a #call_ptx statement
    pub fn compile_call_ptx(
        &mut self,
        call_ptx: &StatementCallPtx<PhaseParse>,
    ) -> Result<(), CompileError> {
        let function_name = call_ptx.function_name.s();

        // Ensure this is a known PTX function
        if !self.ptx_functions.contains(&function_name.to_string()) {
            return Err(CompileError::UnsupportedConstruct(format!(
                "Unknown PTX function: {function_name}"
            )));
        }

        // Handle arguments
        let (has_array, array_var_name, array_info) = if !call_ptx.args.is_empty() {
            // Extract array name from the argument
            let array_var_name = match &call_ptx.args[0] {
                ProcTerm::Variable(var) => var.variable.s(),
                _ => {
                    return Err(CompileError::UnsupportedConstruct(
                        "call_ptx expects array variable as argument".to_string(),
                    ));
                }
            };

            // Get array info
            let array_type_name = self.variable_arrays.get(array_var_name).ok_or_else(|| {
                CompileError::UnsupportedConstruct(format!(
                    "Unknown array variable: {array_var_name}"
                ))
            })?;

            let array_info = self.arrays.get(array_type_name).ok_or_else(|| {
                CompileError::UnsupportedConstruct(format!("Unknown array type: {array_type_name}"))
            })?;

            (true, array_var_name, array_info.clone())
        } else {
            // No arguments - create dummy info
            (
                false,
                "",
                ArrayInfo {
                    element_type: String::new(),
                    field_names: vec![],
                    field_types: vec![],
                    dimension: 1,
                    size: None,
                },
            )
        };

        // Generate CUDA API calls
        self.output.push_str("    # call_ptx implementation\n");

        // self.output.push_str("    sub rsp, 8");

        // Load PTX module if not already loaded
        self.output.push_str("    # Load PTX module\n");
        self.output
            .push_str(&format!("    lea rdi, ptx_code_{function_name}[rip]\n"));
        self.output.push_str("    lea rsi, __cu_module[rip]\n");
        self.output.push_str("    call cuModuleLoadData@PLT\n");
        self.output.push_str("    test eax, eax\n");
        self.output.push_str("    jz module_load_ok\n");
        self.output
            .push_str("    # cuModuleLoadData failed - print error and exit\n");
        self.output.push_str("    mov     rax, 1\n");
        self.output.push_str("    mov     rdi, 2\n");
        self.output
            .push_str("    mov     rsi, offset cuda_module_error\n");
        self.output.push_str("    mov     rdx, 21\n");
        self.output.push_str("    syscall\n");
        self.output.push_str("    mov     rax, 60\n");
        self.output.push_str("    mov     rdi, 4\n");
        self.output.push_str("    syscall\n");
        self.output.push_str("module_load_ok:\n");

        // Get function from module
        self.output.push_str("    # Get function from module\n");
        self.output.push_str("    lea rdi, __cu_function[rip]\n");
        self.output
            .push_str("    mov rsi, QWORD PTR __cu_module[rip]\n");
        self.output.push_str(&format!(
            "    lea rdx, ptx_function_name_{function_name}[rip]\n"
        ));
        self.output.push_str("    call cuModuleGetFunction@PLT\n");
        self.output.push_str("    test eax, eax\n");
        self.output.push_str("    jz function_get_ok\n");
        self.output
            .push_str("    # cuModuleGetFunction failed - print error and exit\n");
        self.output.push_str("    mov     rax, 1\n");
        self.output.push_str("    mov     rdi, 2\n");
        self.output
            .push_str("    mov     rsi, offset cuda_function_error\n");
        self.output.push_str("    mov     rdx, 22\n");
        self.output.push_str("    syscall\n");
        self.output.push_str("    mov     rax, 60\n");
        self.output.push_str("    mov     rdi, 5\n");
        self.output.push_str("    syscall\n");
        self.output.push_str("function_get_ok:\n");

        // self.output.push_str("    mov rax, 231\n");
        // self.output.push_str("    mov rdi, 41\n");
        // self.output.push_str("    syscall\n");

        if has_array {
            // Allocate device memory for each field
            let field_count = array_info.field_names.len();
            for (i, field_name) in array_info.field_names.iter().enumerate() {
                self.output.push_str(&format!(
                    "    # Allocate device memory for field {field_name}\n"
                ));
                self.output
                    .push_str(&format!("    lea rdi, device_ptr_{}[rip]\n", i + 1));

                // Calculate size based on array size and element type
                // For now, assume 65536 elements and 8 bytes per element
                self.output.push_str("    mov rsi, 524288\n"); // 65536 * 8
                self.output.push_str("    call cuMemAlloc_v2@PLT\n");
            }

            // Copy data to device
            for (i, field_name) in array_info.field_names.iter().enumerate() {
                self.output
                    .push_str(&format!("    # Copy {field_name} data to device\n"));
                self.output.push_str(&format!(
                    "    mov rdi, QWORD PTR device_ptr_{}[rip]\n",
                    i + 1
                ));

                // Get host pointer for this field
                let field_ptr_var = format!("{array_var_name}_{field_name}_ptr");
                if let Some(&offset) = self.variables.get(&field_ptr_var) {
                    self.output.push_str(&format!(
                        "    mov rsi, QWORD PTR [rbp - 8 - {}]\n",
                        offset - 8
                    ));
                }

                self.output.push_str("    mov rdx, 524288\n"); // Size
                self.output.push_str("    call cuMemcpyHtoD_v2@PLT\n");
            }

            // Set up kernel parameters
            self.output.push_str("    # Set up kernel parameters\n");
            for i in 1..=field_count {
                self.output
                    .push_str(&format!("    lea rax, device_ptr_{i}[rip]\n"));
                self.output.push_str(&format!(
                    "    mov QWORD PTR [rbp - 8 - {}], rax\n",
                    200 + (i - 1) * 8
                ));
            }
        }

        // self.output.push_str("    mov rax, 231\n");
        // self.output.push_str("    mov rdi, 41\n");
        // self.output.push_str("    syscall\n");

        // Launch kernel
        self.output.push_str("    # Launch kernel\n");

        self.output.push_str("    sub rsp, 8\n");

        self.output
            .push_str("    mov rdi, QWORD PTR __cu_function[rip]\n");

        // Grid dimensions
        self.output
            .push_str(&format!("    mov rsi, {}\n", call_ptx.grid_dim_x.s()));
        self.output
            .push_str(&format!("    mov rdx, {}\n", call_ptx.grid_dim_y.s()));
        self.output
            .push_str(&format!("    mov rcx, {}\n", call_ptx.grid_dim_z.s()));

        // Block dimensions
        self.output
            .push_str(&format!("    mov r8, {}\n", call_ptx.block_dim_x.s()));
        self.output
            .push_str(&format!("    mov r9, {}\n", call_ptx.block_dim_y.s()));
        self.output
            .push_str(&format!("    push {}\n", call_ptx.block_dim_z.s()));

        // Extra (reverse stack order)
        self.output.push_str("    push 0\n");

        // Kernel params (reverse stack order)
        if has_array && !array_info.field_names.is_empty() {
            self.output.push_str("    lea rax, [rbp - 8 -  216]\n");
            self.output.push_str("    push rax\n");
        } else {
            self.output.push_str("    push 0\n"); // NULL params
        }

        // Shared memory and stream (reverse stack order)
        self.output.push_str("    push 0\n"); // sharedMemBytes
        self.output.push_str("    push 0\n"); // stream

        self.output.push_str("    call cuLaunchKernel@PLT\n");
        self.output.push_str("    add rsp, 48\n"); // Clean up stack
        self.output.push_str("    test eax, eax\n");
        self.output.push_str("    jz kernel_launch_ok\n");
        self.output
            .push_str("    # cuLaunchKernel failed - print error and exit\n");
        self.output.push_str("    mov     rax, 1\n");
        self.output.push_str("    mov     rdi, 2\n");
        self.output
            .push_str("    mov     rsi, offset cuda_launch_error\n");
        self.output.push_str("    mov     rdx, 22\n");
        self.output.push_str("    syscall\n");
        self.output.push_str("    mov     rax, 60\n");
        self.output.push_str("    mov     rdi, 6\n");
        self.output.push_str("    syscall\n");
        self.output.push_str("kernel_launch_ok:\n");

        // Synchronize
        self.output.push_str("    call cuCtxSynchronize@PLT\n");

        if has_array {
            // Copy results back
            for (i, field_name) in array_info.field_names.iter().enumerate() {
                self.output
                    .push_str(&format!("    # Copy {field_name} data back from device\n"));

                // Get host pointer for this field
                let field_ptr_var = format!("{array_var_name}_{field_name}_ptr");
                if let Some(&offset) = self.variables.get(&field_ptr_var) {
                    self.output.push_str(&format!(
                        "    mov rdi, QWORD PTR [rbp - 8 - {}]\n",
                        offset - 8
                    ));
                }

                self.output.push_str(&format!(
                    "    mov rsi, QWORD PTR device_ptr_{}[rip]\n",
                    i + 1
                ));
                self.output.push_str("    mov rdx, 524288\n"); // Size
                self.output.push_str("    call cuMemcpyDtoH_v2@PLT\n");
            }

            // Free device memory
            let field_count = array_info.field_names.len();
            for i in 1..=field_count {
                self.output
                    .push_str(&format!("    # Free device memory {i}\n"));
                self.output
                    .push_str(&format!("    mov rdi, QWORD PTR device_ptr_{i}[rip]\n"));
                self.output.push_str("    call cuMemFree_v2@PLT\n");
            }
        }

        // self.output.push_str("    mov rax, 231\n");
        // self.output.push_str("    mov rdi, 41\n");
        // self.output.push_str("    syscall\n");

        Ok(())
    }
}

#[cfg(test)]
mod tests;
