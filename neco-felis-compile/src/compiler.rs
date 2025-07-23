use crate::{compile_options::CompileOptions, error::CompileError};
use neco_felis_syn::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ArrayInfo {
    #[allow(dead_code)]
    pub element_type: String,
    pub field_names: Vec<String>,
    pub field_types: Vec<String>,
    #[allow(dead_code)]
    pub dimension: usize,
    #[allow(dead_code)]
    pub size: Option<usize>,
}

pub struct AssemblyCompiler {
    pub output: String,
    pub entrypoint: Option<String>,
    pub builtins: HashMap<String, String>,
    pub variables: HashMap<String, i32>,
    pub stack_offset: i32,
    pub arrays: HashMap<String, ArrayInfo>,
    pub variable_arrays: HashMap<String, String>,
    pub loop_stack: Vec<String>,
    pub compile_options: CompileOptions,
}

impl AssemblyCompiler {
    pub fn new(compile_options: CompileOptions) -> Self {
        Self {
            output: String::new(),
            entrypoint: None,
            builtins: HashMap::new(),
            variables: HashMap::new(),
            stack_offset: 0,
            arrays: HashMap::new(),
            variable_arrays: HashMap::new(),
            loop_stack: Vec::new(),
            compile_options,
        }
    }

    pub fn compile_file(&mut self, file: &File<PhaseParse>) -> Result<String, CompileError> {
        if !self.compile_options.use_ptx {
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
        } else {
            self.output.push_str(".intel_syntax noprefix\n");

            // ptx template
            let s = r#"
    .text
    .globl	__cu_device
    .bss
    .align 4
    .type	__cu_device, @object
    .size	__cu_device, 4
__cu_device:
    .zero	4
    .globl	__cu_context
    .align 8
    .type	__cu_context, @object
    .size	__cu_context, 8
__cu_context:
    .zero	8
    .globl	__cu_module
    .align 8
    .type	__cu_module, @object
    .size	__cu_module, 8
__cu_module:
    .zero	8
    .globl	__cu_function
    .align 8
    .type	__cu_function, @object
    .size	__cu_function, 8
__cu_function:
    .zero	8
    .globl	__cu_device_ptr
    .align 8
    .type	__cu_device_ptr, @object
    .size	__cu_device_ptr, 8
__cu_device_ptr:
    .zero	8
"#;
            self.output.push_str(s);

            self.output.push_str(".section .text\n");
            self.output.push_str(".globl main\n\n");

            for item in file.items() {
                self.compile_item(item)?;
            }

            if let Some(entrypoint) = &self.entrypoint {
                self.output.push_str("main:\n");

                self.output.push_str("    push rbp\n");
                let s = r#"
    # cuInit(0)
    mov     edi, 0
    call    cuInit@PLT
    # cuDeviceGet(&cu_device, 0)
    mov	    esi, 0
    lea	    rdi, __cu_device[rip]
    call    cuDeviceGet@PLT
    # cuCtxCreate_v2(&cu_context, 0, cu_device)
    mov	    eax, DWORD PTR __cu_device[rip]
    mov	    edx, eax
    mov	    esi, 0
    lea	    rdi, __cu_context[rip]
    call    cuCtxCreate_v2@PLT
"#;
                self.output.push_str(s);

                self.output.push_str(&format!("    call {entrypoint}\n"));
                self.output.push_str("    mov rax, 60\n");
                self.output.push_str("    mov rdi, 0\n");
                self.output.push_str("    syscall\n");
                self.output.push_str("    pop rbp\n");
            } else {
                return Err(CompileError::EntrypointNotFound);
            }

            Ok(self.output.clone())
        }
    }

    pub fn compile_item(&mut self, item: &Item<PhaseParse>) -> Result<(), CompileError> {
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
            Item::Array(array) => crate::arrays::compile_array(array, &mut self.arrays),
            _ => Err(CompileError::UnsupportedConstruct(format!("{item:?}"))),
        }
    }

    pub fn compile_proc(&mut self, proc: &ItemProc<PhaseParse>) -> Result<(), CompileError> {
        // Extract parameter names from the function type
        let param_names = self.extract_proc_parameters(&proc.ty);
        let param_count = param_names.len();
        let let_count = self.count_let_variables_in_proc_block(&proc.proc_block);
        let total_stack_space = (param_count + let_count as usize) * 8;

        self.output.push_str(&format!("{}:\n", proc.name.s()));

        if total_stack_space > 0 {
            self.output
                .push_str(&format!("    sub rsp, {total_stack_space}\n"));
        }

        self.stack_offset = 0;

        // Store parameters from registers to stack and register them as variables
        for (i, param_name) in param_names.iter().enumerate() {
            self.stack_offset += 8;
            let offset = self.stack_offset;
            self.variables.insert(param_name.clone(), offset);

            // Store parameter from register to stack
            match i {
                0 => self
                    .output
                    .push_str(&format!("    mov qword ptr [rsp + {}], rdi\n", offset - 8)),
                1 => self
                    .output
                    .push_str(&format!("    mov qword ptr [rsp + {}], rsi\n", offset - 8)),
                // For now, only support up to 2 parameters
                _ => {
                    return Err(CompileError::UnsupportedConstruct(
                        "More than 2 parameters not supported".to_string(),
                    ));
                }
            }
        }

        self.compile_proc_block(&proc.proc_block)?;

        if total_stack_space > 0 {
            self.output
                .push_str(&format!("    add rsp, {total_stack_space}\n"));
        }

        self.output.push_str("    ret\n\n");
        self.variables.clear();

        Ok(())
    }

    /// Extract parameter names from a procedure type signature
    fn extract_proc_parameters(&self, ty: &Term<PhaseParse>) -> Vec<String> {
        let mut params = Vec::new();
        Self::extract_params_recursive(ty, &mut params);
        params
    }

    /// Recursively extract parameters from dependent arrow types
    fn extract_params_recursive(term: &Term<PhaseParse>, params: &mut Vec<String>) {
        match term {
            Term::ArrowDep(arrow_dep) => {
                // Extract parameter name from dependent arrow (x : A) -> B
                params.push(arrow_dep.from.variable.s().to_string());
                // Continue with the return type to find more parameters
                Self::extract_params_recursive(&arrow_dep.to, params);
            }
            Term::ArrowNodep(arrow_nodep) => {
                // Check if this is a unit type () -> something
                if let Term::Unit(_) = &*arrow_nodep.from {
                    // This is () -> B, so no parameters from this arrow
                    Self::extract_params_recursive(&arrow_nodep.to, params);
                } else {
                    // For non-dependent arrows A -> B, we don't have parameter names
                    // This shouldn't happen in well-formed procedure signatures, but handle it gracefully
                    params.push(format!("param_{}", params.len()));
                    // Continue with the return type
                    Self::extract_params_recursive(&arrow_nodep.to, params);
                }
            }
            // Base case: we've reached the return type (including unit type)
            _ => {}
        }
    }

    pub fn compile_proc_block(
        &mut self,
        block: &ItemProcBlock<PhaseParse>,
    ) -> Result<(), CompileError> {
        self.compile_statements(&block.statements)
    }

    pub fn compile_statements(
        &mut self,
        statements: &Statements<PhaseParse>,
    ) -> Result<(), CompileError> {
        match statements {
            Statements::Then(then) => {
                self.compile_statement(&then.head)?;
                match &*then.tail {
                    Statements::Nil => Ok(()),
                    _ => self.compile_statements(&then.tail),
                }
            }
            Statements::Statement(statement) => self.compile_statement(statement),
            Statements::Nil => Ok(()),
        }
    }

    pub fn compile_proc_term(
        &mut self,
        proc_term: &ProcTerm<PhaseParse>,
    ) -> Result<(), CompileError> {
        match proc_term {
            ProcTerm::Apply(apply) => self.compile_proc_apply(apply),
            ProcTerm::Variable(var) => self.compile_proc_variable(var),
            ProcTerm::FieldAccess(field_access) => self.compile_proc_field_access(field_access),
            ProcTerm::ConstructorCall(constructor_call) => {
                self.compile_proc_constructor_call(constructor_call)
            }
            ProcTerm::Dereference(dereference) => self.compile_proc_dereference(dereference),
            ProcTerm::If(if_expr) => crate::control_flow::compile_proc_if(self, if_expr),
            _ => Err(CompileError::UnsupportedConstruct(format!("{proc_term:?}"))),
        }
    }

    pub fn compile_proc_variable(
        &mut self,
        var: &ProcTermVariable<PhaseParse>,
    ) -> Result<(), CompileError> {
        let var_name = var.variable.s();

        // Check if the variable exists in our variable map
        if let Some(&offset) = self.variables.get(var_name) {
            // Load the variable value from its stack location into rax
            self.output
                .push_str(&format!("    mov rax, qword ptr [rsp + {}]\n", offset - 8));
            Ok(())
        } else {
            Err(CompileError::UnsupportedConstruct(format!(
                "Unknown variable: {var_name}"
            )))
        }
    }

    pub fn compile_proc_apply(
        &mut self,
        apply: &ProcTermApply<PhaseParse>,
    ) -> Result<(), CompileError> {
        if let ProcTerm::Variable(var) = &*apply.f {
            if let Some(builtin) = self.builtins.get(var.variable.s()) {
                if builtin == "syscall" {
                    return self.compile_proc_syscall(&apply.args);
                }
            } else {
                // This is a call to a user-defined procedure
                todo!();
            }
        }
        Err(CompileError::UnsupportedConstruct(format!("{apply:?}")))
    }

    pub fn compile_proc_field_access(
        &mut self,
        field_access: &ProcTermFieldAccess<PhaseParse>,
    ) -> Result<(), CompileError> {
        let object_name = field_access.object.s();
        let field_name = field_access.field.s();

        // Check if this is a Structure of Arrays (SoA) access
        let soa_ptr_var_name = format!("{object_name}_{field_name}_ptr");
        if let Some(&ptr_offset) = self.variables.get(&soa_ptr_var_name) {
            // This is SoA access - load the field array pointer
            self.output.push_str(&format!(
                "    mov rax, qword ptr [rsp + {}]\n",
                ptr_offset - 8
            ));

            // Handle index if present
            if let Some(index_term) = &field_access.index {
                // Get array info for element size calculation
                if let Some(array_type_name) = self.variable_arrays.get(object_name)
                    && let Some(array_info) = self.arrays.get(array_type_name)
                {
                    let element_size = crate::arrays::get_element_size(
                        &array_info.field_types,
                        &array_info.field_names,
                        field_name,
                    )?;

                    match &**index_term {
                        ProcTerm::Number(num) => {
                            let index = crate::arrays::parse_number(num.number.s());
                            let offset = index.parse::<usize>().unwrap_or(0) * element_size;
                            if offset > 0 {
                                self.output.push_str(&format!("    add rax, {offset}\n"));
                            }
                        }
                        ProcTerm::Variable(var) => {
                            if let Some(&var_offset) = self.variables.get(var.variable.s()) {
                                self.output.push_str(&format!(
                                    "    mov rbx, qword ptr [rsp + {}]\n",
                                    var_offset - 8
                                ));
                                self.output
                                    .push_str(&format!("    mov rcx, {element_size}\n"));
                                self.output.push_str("    imul rbx, rcx\n");
                                self.output.push_str("    add rax, rbx\n");
                            }
                        }
                        _ => {
                            return Err(CompileError::UnsupportedConstruct(format!(
                                "Unsupported index type in field access: {index_term:?}"
                            )));
                        }
                    }
                }
            }
            // rax now contains the address of the field element in the SoA
            Ok(())
        } else {
            Err(CompileError::UnsupportedConstruct(format!(
                "Unknown variable: {object_name}"
            )))
        }
    }

    pub fn compile_proc_dereference(
        &mut self,
        dereference: &ProcTermDereference<PhaseParse>,
    ) -> Result<(), CompileError> {
        // First compile the term that produces a reference
        self.compile_proc_term(&dereference.term)?;

        // Then dereference it - rax contains the address, we need to load the value
        self.output.push_str("    mov eax, dword ptr [rax]\n");

        Ok(())
    }

    pub fn compile_proc_constructor_call_with_var(
        &mut self,
        constructor_call: &ProcTermConstructorCall<PhaseParse>,
        var_name: &str,
    ) -> Result<(), CompileError> {
        let type_name = constructor_call.type_name.s();
        let method_name = constructor_call.method.s();
        let constructor_name = format!("{type_name}::{method_name}");

        if constructor_name.contains("::new_with_size") {
            // Look up array information
            if let Some(array_info) = self.arrays.get(type_name).cloned() {
                // Get the size argument
                let size_arg = if !constructor_call.args.is_empty()
                    && let Some(arg) = constructor_call.args.first()
                {
                    match arg {
                        ProcTerm::Number(num) => num.number.s().to_string(),
                        ProcTerm::Variable(var) => {
                            if let Some(&offset) = self.variables.get(var.variable.s()) {
                                // Load variable value into rsi for use by SoA allocation
                                self.output.push_str(&format!(
                                    "    mov rsi, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                "rsi".to_string()
                            } else {
                                return Err(CompileError::UnsupportedConstruct(format!(
                                    "Unknown variable in array size: {}",
                                    var.variable.s()
                                )));
                            }
                        }
                        _ => {
                            return Err(CompileError::UnsupportedConstruct(format!(
                                "Unsupported size argument type: {arg:?}"
                            )));
                        }
                    }
                } else {
                    return Err(CompileError::UnsupportedConstruct(
                        "Missing size argument for array constructor".to_string(),
                    ));
                };

                // Generate Structure of Arrays allocation using variable name
                crate::arrays::generate_soa_allocation_with_var(
                    var_name,
                    &array_info,
                    &size_arg,
                    &mut self.output,
                    &mut self.stack_offset,
                    &mut self.variables,
                )?;

                Ok(())
            } else {
                Err(CompileError::UnsupportedConstruct(format!(
                    "Array type not found: {type_name}"
                )))
            }
        } else {
            Err(CompileError::UnsupportedConstruct(format!(
                "Constructor call not yet implemented: {constructor_name}"
            )))
        }
    }

    pub fn compile_proc_constructor_call(
        &mut self,
        constructor_call: &ProcTermConstructorCall<PhaseParse>,
    ) -> Result<(), CompileError> {
        let type_name = constructor_call.type_name.s();
        let method_name = constructor_call.method.s();
        let constructor_name = format!("{type_name}::{method_name}");

        if constructor_name.contains("::new_with_size") {
            // Look up array information
            if let Some(array_info) = self.arrays.get(type_name).cloned() {
                // Get the size argument
                let size_arg = if !constructor_call.args.is_empty()
                    && let Some(arg) = constructor_call.args.first()
                {
                    match arg {
                        ProcTerm::Number(num) => num.number.s().to_string(),
                        ProcTerm::Variable(var) => {
                            if let Some(&offset) = self.variables.get(var.variable.s()) {
                                // Load variable value into rsi for use by SoA allocation
                                self.output.push_str(&format!(
                                    "    mov rsi, qword ptr [rsp + {}]\n",
                                    offset - 8
                                ));
                                "rsi".to_string()
                            } else {
                                return Err(CompileError::UnsupportedConstruct(format!(
                                    "Unknown variable in array size: {}",
                                    var.variable.s()
                                )));
                            }
                        }
                        _ => {
                            return Err(CompileError::UnsupportedConstruct(format!(
                                "Unsupported size argument type: {arg:?}"
                            )));
                        }
                    }
                } else {
                    return Err(CompileError::UnsupportedConstruct(
                        "Missing size argument for array constructor".to_string(),
                    ));
                };

                // Generate Structure of Arrays allocation
                crate::arrays::generate_soa_allocation(
                    type_name,
                    &array_info,
                    &size_arg,
                    &mut self.output,
                    &mut self.stack_offset,
                    &mut self.variables,
                    &mut self.arrays,
                )?;

                Ok(())
            } else {
                Err(CompileError::UnsupportedConstruct(format!(
                    "Array type not found: {type_name}"
                )))
            }
        } else {
            Err(CompileError::UnsupportedConstruct(format!(
                "Constructor call not yet implemented: {constructor_name}"
            )))
        }
    }

    pub fn compile_proc_syscall(
        &mut self,
        args: &[ProcTerm<PhaseParse>],
    ) -> Result<(), CompileError> {
        if args.len() != 6 {
            return Err(CompileError::InvalidSyscall);
        }

        let registers = ["rax", "rdi", "rsi", "rdx", "r10", "r8"];

        for (i, arg) in args.iter().enumerate() {
            match arg {
                ProcTerm::Number(num) => {
                    let number_value = self.parse_number(num.number.s());
                    self.output
                        .push_str(&format!("    mov {}, {}\n", registers[i], number_value));
                }
                ProcTerm::Variable(var) => {
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

    pub fn parse_number(&self, number_str: &str) -> String {
        if number_str.ends_with("u64") {
            number_str.trim_end_matches("u64").to_string()
        } else {
            number_str.to_string()
        }
    }

    pub fn load_proc_argument_into_register(
        &mut self,
        arg: &ProcTerm<PhaseParse>,
        register: &str,
    ) -> Result<(), CompileError> {
        match arg {
            ProcTerm::Number(number) => {
                let value = self.parse_number(number.number.s());
                self.output
                    .push_str(&format!("    mov {register}, {value}\n"));
                Ok(())
            }
            ProcTerm::Variable(var) => {
                if let Some(offset) = self.variables.get(var.variable.s()) {
                    self.output.push_str(&format!(
                        "    mov {register}, qword ptr [rsp + {}]\n",
                        offset - 8
                    ));
                } else {
                    self.output.push_str(&format!("    mov {register}, 0\n"));
                }
                Ok(())
            }
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "Unsupported argument type: {arg:?}"
            ))),
        }
    }

    pub fn count_let_variables_in_proc_block(&self, block: &ItemProcBlock<PhaseParse>) -> i32 {
        Self::count_let_variables_in_statements(&block.statements)
    }

    pub fn count_let_variables_in_statements(statements: &Statements<PhaseParse>) -> i32 {
        match statements {
            Statements::Then(then) => {
                Self::count_let_variables_in_statement(&then.head)
                    + Self::count_let_variables_in_statements(&then.tail)
            }
            Statements::Statement(statement) => Self::count_let_variables_in_statement(statement),
            Statements::Nil => 0,
        }
    }

    pub fn count_let_variables_in_statement(statement: &Statement<PhaseParse>) -> i32 {
        match statement {
            Statement::Let(_) => 1,
            Statement::LetMut(_) => 1,
            Statement::Expr(proc_term) => Self::count_let_variables_in_proc_term(proc_term),
            _ => 0,
        }
    }

    pub fn count_let_variables_in_proc_term(_proc_term: &ProcTerm<PhaseParse>) -> i32 {
        0
    }
}
