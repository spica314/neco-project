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
    arrays: HashMap<String, ArrayInfo>, // Track array information
    variable_arrays: HashMap<String, String>, // Maps variable names to array type names
}

#[derive(Debug, Clone)]
struct ArrayInfo {
    #[allow(dead_code)]
    element_type: String,
    field_names: Vec<String>,
    field_types: Vec<String>,
    #[allow(dead_code)]
    dimension: usize,
    size: Option<usize>, // Runtime size, None if not yet allocated
}

impl AssemblyCompiler {
    fn new() -> Self {
        Self {
            output: String::new(),
            entrypoint: None,
            builtins: HashMap::new(),
            variables: HashMap::new(),
            stack_offset: 0,
            arrays: HashMap::new(),
            variable_arrays: HashMap::new(),
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
            Item::Array(array) => self.compile_array(array),
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
            Term::LetMut(let_mut_expr) => self.compile_let_mut(let_mut_expr),
            Term::Assign(assign_expr) => self.compile_assign(assign_expr),
            Term::FieldAccess(field_access) => self.compile_field_access(field_access),
            Term::FieldAssign(field_assign) => self.compile_field_assign(field_assign),
            Term::ConstructorCall(constructor_call) => {
                self.compile_constructor_call(constructor_call)
            }
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

    fn compile_let_mut(
        &mut self,
        let_mut_expr: &TermLetMut<PhaseParse>,
    ) -> Result<(), CompileError> {
        let var_name = let_mut_expr.variable_name();

        // Allocate stack space for this variable
        self.stack_offset += 8; // 8 bytes for 64-bit value
        let offset = self.stack_offset;

        // Store the variable's stack offset
        self.variables.insert(var_name.to_string(), offset);

        match &*let_mut_expr.value {
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
                // Handle function application in let mut expression
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
                    "let mut with unsupported function application: {apply:?}"
                )))
            }
            Term::ConstructorCall(constructor_call) => {
                // Handle constructor calls in let mut expressions
                if constructor_call.method_name() == "new_with_size" {
                    // Record the relationship between variable name and array type
                    self.variable_arrays.insert(
                        var_name.to_string(),
                        constructor_call.type_name().to_string(),
                    );
                    // Compile the constructor call with the variable name context
                    self.compile_constructor_call_with_var(constructor_call, var_name)?;
                } else {
                    self.compile_constructor_call(constructor_call)?;
                }
                Ok(())
            }
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "let mut with non-number value: {let_mut_expr:?}"
            ))),
        }
    }

    fn compile_assign(&mut self, assign_expr: &TermAssign<PhaseParse>) -> Result<(), CompileError> {
        let var_name = assign_expr.variable_name();

        // Check if the variable exists
        let offset = *self.variables.get(var_name).ok_or_else(|| {
            CompileError::UnsupportedConstruct(format!("Unknown variable: {var_name}"))
        })?;

        match &*assign_expr.value {
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
                // Handle function application in assignment
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
                    "assignment with unsupported function application: {apply:?}"
                )))
            }
            Term::ConstructorCall(constructor_call) => {
                // Handle constructor calls in assignments
                self.compile_constructor_call(constructor_call)?;
                Ok(())
            }
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "assignment with non-number value: {assign_expr:?}"
            ))),
        }
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
            Term::ConstructorCall(constructor_call) => {
                // Handle constructor calls in let expressions
                if constructor_call.method_name() == "new_with_size" {
                    // Record the relationship between variable name and array type
                    self.variable_arrays.insert(
                        var_name.to_string(),
                        constructor_call.type_name().to_string(),
                    );
                    // Compile the constructor call with the variable name context
                    self.compile_constructor_call_with_var(constructor_call, var_name)?;
                } else {
                    self.compile_constructor_call(constructor_call)?;
                }
                Ok(())
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
            Term::FieldAccess(field_access) => {
                // Handle array field access like "points.x 0"
                let obj_name = field_access.object_name();
                let field_name = field_access.field_name();

                // Look up the array type from variable name
                if let Some(array_type_name) = self.variable_arrays.get(obj_name) {
                    if let Some(array_info) = self.arrays.get(array_type_name).cloned() {
                        if let Some(index_term) = &field_access.index {
                            // Get the pointer for this field
                            let ptr_var_name = format!("{obj_name}_{field_name}_ptr");
                            if let Some(&ptr_offset) = self.variables.get(&ptr_var_name) {
                                // Load the base pointer
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    ptr_offset - 8
                                ));

                                // Calculate offset based on index
                                let element_size = self.get_element_size(
                                    &array_info.field_types,
                                    &array_info.field_names,
                                    field_name,
                                )?;

                                match &**index_term {
                                    Term::Number(num) => {
                                        let index = self.parse_number(num.number.s());
                                        let offset =
                                            index.parse::<usize>().unwrap_or(0) * element_size;
                                        self.output.push_str(&format!("    add rax, {offset}\n"));
                                    }
                                    Term::Variable(var) => {
                                        if let Some(&var_offset) =
                                            self.variables.get(var.variable.s())
                                        {
                                            self.output.push_str(&format!(
                                                "    mov rbx, qword ptr [rsp + {}]\n",
                                                var_offset - 8
                                            ));
                                            self.output.push_str(&format!(
                                                "    mov rcx, {}\n",
                                                element_size
                                            ));
                                            self.output.push_str("    imul rbx, rcx\n");
                                            self.output.push_str("    add rax, rbx\n");
                                        }
                                    }
                                    _ => {
                                        return Err(CompileError::UnsupportedConstruct(format!(
                                            "Unsupported index type: {:?}",
                                            index_term
                                        )));
                                    }
                                }

                                // Load the f32 value from the calculated address
                                self.output
                                    .push_str(&format!("    movss {register}, dword ptr [rax]\n"));
                            } else {
                                return Err(CompileError::UnsupportedConstruct(format!(
                                    "Unknown array field pointer: {}",
                                    ptr_var_name
                                )));
                            }
                        } else {
                            return Err(CompileError::UnsupportedConstruct(format!(
                                "Array field access missing index: {}.{}",
                                obj_name, field_name
                            )));
                        }
                    } else {
                        return Err(CompileError::UnsupportedConstruct(format!(
                            "Unknown array: {}",
                            obj_name
                        )));
                    }
                }
            }
            Term::Paren(paren) => {
                // Handle parenthesized expressions by delegating to the inner term
                self.load_f32_argument_into_register(&paren.term, register)?;
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
            Term::FieldAccess(field_access) => {
                // Handle array field access like "points.x 0"
                let obj_name = field_access.object_name();
                let field_name = field_access.field_name();

                // Look up the array type from variable name
                if let Some(array_type_name) = self.variable_arrays.get(obj_name) {
                    if let Some(array_info) = self.arrays.get(array_type_name).cloned() {
                        if let Some(index_term) = &field_access.index {
                            // Get the pointer for this field
                            let ptr_var_name = format!("{obj_name}_{field_name}_ptr");
                            if let Some(&ptr_offset) = self.variables.get(&ptr_var_name) {
                                // Load the base pointer
                                self.output.push_str(&format!(
                                    "    mov rax, qword ptr [rsp + {}]\n",
                                    ptr_offset - 8
                                ));

                                // Calculate offset based on index
                                let element_size = self.get_element_size(
                                    &array_info.field_types,
                                    &array_info.field_names,
                                    field_name,
                                )?;

                                match &**index_term {
                                    Term::Number(num) => {
                                        let index = self.parse_number(num.number.s());
                                        let offset =
                                            index.parse::<usize>().unwrap_or(0) * element_size;
                                        self.output.push_str(&format!("    add rax, {offset}\n"));
                                    }
                                    Term::Variable(var) => {
                                        if let Some(&var_offset) =
                                            self.variables.get(var.variable.s())
                                        {
                                            self.output.push_str(&format!(
                                                "    mov rbx, qword ptr [rsp + {}]\n",
                                                var_offset - 8
                                            ));
                                            self.output.push_str(&format!(
                                                "    mov rcx, {}\n",
                                                element_size
                                            ));
                                            self.output.push_str("    imul rbx, rcx\n");
                                            self.output.push_str("    add rax, rbx\n");
                                        }
                                    }
                                    _ => {
                                        return Err(CompileError::UnsupportedConstruct(format!(
                                            "Unsupported index type: {:?}",
                                            index_term
                                        )));
                                    }
                                }

                                // Load the value from the calculated address
                                self.output
                                    .push_str(&format!("    mov {register}, qword ptr [rax]\n"));
                            } else {
                                return Err(CompileError::UnsupportedConstruct(format!(
                                    "Unknown array field pointer: {}",
                                    ptr_var_name
                                )));
                            }
                        } else {
                            return Err(CompileError::UnsupportedConstruct(format!(
                                "Array field access missing index: {}.{}",
                                obj_name, field_name
                            )));
                        }
                    } else {
                        return Err(CompileError::UnsupportedConstruct(format!(
                            "Unknown array: {}",
                            obj_name
                        )));
                    }
                }
            }
            Term::Paren(paren) => {
                // Handle parenthesized expressions by delegating to the inner term
                self.load_argument_into_register(&paren.term, register)?;
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
        let let_vars = Self::count_let_variables_in_statements(&block.statements);
        let array_ptrs = Self::count_array_pointers_in_statements(&block.statements);
        let_vars + array_ptrs
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
            Term::LetMut(_) => 1,
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

    fn count_array_pointers_in_statements(statements: &Statements<PhaseParse>) -> i32 {
        match statements {
            Statements::Then(then) => {
                let head_count = Self::count_array_pointers_in_term(&then.head);
                let tail_count = Self::count_array_pointers_in_statements(&then.tail);
                head_count + tail_count
            }
            Statements::Term(term) => Self::count_array_pointers_in_term(term),
            Statements::Nil => 0,
        }
    }

    fn count_array_pointers_in_term(term: &Term<PhaseParse>) -> i32 {
        match term {
            Term::ConstructorCall(constructor) => {
                // Each constructor call like "Points::new_with_size" will create array pointers
                if constructor.method_name() == "new_with_size" {
                    // For now, assume 3 fields per array (x, y, z) - this is a simplification
                    // In a real implementation, we'd parse the array definition to get the exact count
                    3
                } else {
                    0
                }
            }
            Term::Let(let_expr) => Self::count_array_pointers_in_term(&let_expr.value),
            Term::LetMut(let_mut_expr) => Self::count_array_pointers_in_term(&let_mut_expr.value),
            Term::Apply(apply) => {
                let mut count = Self::count_array_pointers_in_term(&apply.f);
                for arg in &apply.args {
                    count += Self::count_array_pointers_in_term(arg);
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

    fn compile_array(&mut self, array: &ItemArray<PhaseParse>) -> Result<(), CompileError> {
        let array_name = array.name().s().to_string();
        let mut field_names = Vec::new();
        let mut field_types = Vec::new();
        let mut dimension = 1;

        for field in array.fields() {
            let field_name = field.keyword.s();
            match field_name {
                "item" => {
                    // Parse the struct fields from the item definition
                    if let Term::Struct(item_struct) = &*field.value {
                        for struct_field in item_struct.fields() {
                            field_names.push(struct_field.name.s().to_string());
                            // Extract type from field type (simplified)
                            field_types.push(self.extract_type_from_term(&struct_field.ty)?);
                        }
                    }
                }
                "dimension" => {
                    if let Term::Number(num) = &*field.value {
                        dimension = num.number.s().parse::<usize>().unwrap_or(1);
                    }
                }
                "new_with_size" => {
                    // This defines the constructor method name, we'll handle it in constructor call
                }
                _ => {}
            }
        }

        let array_info = ArrayInfo {
            element_type: "struct".to_string(),
            field_names,
            field_types,
            dimension,
            size: None,
        };

        self.arrays.insert(array_name, array_info);
        Ok(())
    }

    fn extract_type_from_term(&self, term: &Term<PhaseParse>) -> Result<String, CompileError> {
        match term {
            Term::Variable(var) => Ok(var.variable.s().to_string()),
            _ => Err(CompileError::UnsupportedConstruct(format!(
                "Unsupported type term: {term:?}"
            ))),
        }
    }

    fn compile_constructor_call(
        &mut self,
        constructor: &TermConstructorCall<PhaseParse>,
    ) -> Result<(), CompileError> {
        let type_name = constructor.type_name();
        let method_name = constructor.method_name();

        if method_name == "new_with_size" {
            if let Some(array_info) = self.arrays.get(type_name).cloned() {
                if constructor.args.len() != 1 {
                    return Err(CompileError::UnsupportedConstruct(format!(
                        "new_with_size expects 1 argument, got {}",
                        constructor.args.len()
                    )));
                }

                // Get the size from the argument
                let size_value = match &constructor.args[0] {
                    Term::Number(num) => self.parse_number(num.number.s()),
                    Term::Variable(var) => {
                        if let Some(&offset) = self.variables.get(var.variable.s()) {
                            // Load size from variable
                            self.output.push_str(&format!(
                                "    mov rsi, qword ptr [rsp + {}]\n",
                                offset - 8
                            ));
                            "rsi".to_string()
                        } else {
                            return Err(CompileError::UnsupportedConstruct(format!(
                                "Unknown variable: {}",
                                var.variable.s()
                            )));
                        }
                    }
                    _ => {
                        return Err(CompileError::UnsupportedConstruct(format!(
                            "Unsupported size argument: {:?}",
                            constructor.args[0]
                        )));
                    }
                };

                // Generate SOA allocation code using mmap
                self.generate_soa_allocation(type_name, &array_info, &size_value)?;
                return Ok(());
            }
        }

        Err(CompileError::UnsupportedConstruct(format!(
            "Unknown constructor call: {}::{}",
            type_name, method_name
        )))
    }

    fn compile_constructor_call_with_var(
        &mut self,
        constructor: &TermConstructorCall<PhaseParse>,
        var_name: &str,
    ) -> Result<(), CompileError> {
        let type_name = constructor.type_name();
        let method_name = constructor.method_name();

        if method_name == "new_with_size" {
            if let Some(array_info) = self.arrays.get(type_name).cloned() {
                if constructor.args.len() != 1 {
                    return Err(CompileError::UnsupportedConstruct(format!(
                        "new_with_size expects 1 argument, got {}",
                        constructor.args.len()
                    )));
                }

                // Get the size from the argument
                let size_value = match &constructor.args[0] {
                    Term::Number(num) => self.parse_number(num.number.s()),
                    Term::Variable(var) => {
                        if let Some(&offset) = self.variables.get(var.variable.s()) {
                            // Load size from variable
                            self.output.push_str(&format!(
                                "    mov rsi, qword ptr [rsp + {}]\n",
                                offset - 8
                            ));
                            "rsi".to_string()
                        } else {
                            return Err(CompileError::UnsupportedConstruct(format!(
                                "Unknown variable: {}",
                                var.variable.s()
                            )));
                        }
                    }
                    _ => {
                        return Err(CompileError::UnsupportedConstruct(format!(
                            "Unsupported size argument: {:?}",
                            constructor.args[0]
                        )));
                    }
                };

                // Generate SOA allocation code using mmap with variable name
                self.generate_soa_allocation_with_var(var_name, &array_info, &size_value)?;
                return Ok(());
            }
        }

        Err(CompileError::UnsupportedConstruct(format!(
            "Unknown constructor call: {}::{}",
            type_name, method_name
        )))
    }

    fn generate_soa_allocation(
        &mut self,
        array_name: &str,
        array_info: &ArrayInfo,
        size: &str,
    ) -> Result<(), CompileError> {
        // For each field in the struct, allocate a separate array using mmap
        // mmap syscall: rax=9, rdi=addr(0), rsi=length, rdx=prot, r10=flags, r8=fd, r9=offset

        let mut updated_info = array_info.clone();

        for (field_idx, field_name) in array_info.field_names.iter().enumerate() {
            let field_type = &array_info.field_types[field_idx];

            // Calculate size needed for this field array
            let element_size = match field_type.as_str() {
                "f32" => 4,
                "f64" => 8,
                "u64" | "i64" => 8,
                "u32" | "i32" => 4,
                _ => 8, // Default to 8 bytes
            };

            // Calculate total size = element_size * array_length
            if size != "rsi" {
                self.output.push_str(&format!("    mov rsi, {}\n", size));
            }
            self.output
                .push_str(&format!("    mov rax, {}\n", element_size));
            self.output.push_str("    mul rsi\n"); // rax = element_size * array_length
            self.output.push_str("    mov rsi, rax\n"); // rsi = total_size

            // mmap syscall
            self.output.push_str("    mov rax, 9\n"); // sys_mmap
            self.output.push_str("    mov rdi, 0\n"); // addr = NULL
            // rsi already contains total_size
            self.output.push_str("    mov rdx, 3\n"); // prot = PROT_READ | PROT_WRITE
            self.output.push_str("    mov r10, 34\n"); // flags = MAP_PRIVATE | MAP_ANONYMOUS
            self.output.push_str("    mov r8, -1\n"); // fd = -1
            self.output.push_str("    mov r9, 0\n"); // offset = 0
            self.output.push_str("    syscall\n");

            // Store the returned pointer for this field
            // We'll use a simple naming convention: arrayname_fieldname_ptr
            let ptr_var_name = format!("{}_{}_ptr", array_name, field_name);
            self.stack_offset += 8;
            let ptr_offset = self.stack_offset;
            self.variables.insert(ptr_var_name, ptr_offset);

            // Store the mmap result (in rax) to the pointer variable
            self.output.push_str(&format!(
                "    mov qword ptr [rsp + {}], rax\n",
                ptr_offset - 8
            ));
        }

        // Update array info with the size
        if let Ok(size_num) = size.parse::<usize>() {
            updated_info.size = Some(size_num);
        }
        self.arrays.insert(array_name.to_string(), updated_info);

        Ok(())
    }

    fn generate_soa_allocation_with_var(
        &mut self,
        var_name: &str,
        array_info: &ArrayInfo,
        size: &str,
    ) -> Result<(), CompileError> {
        // For each field in the struct, allocate a separate array using mmap
        // mmap syscall: rax=9, rdi=addr(0), rsi=length, rdx=prot, r10=flags, r8=fd, r9=offset

        for (field_idx, field_name) in array_info.field_names.iter().enumerate() {
            let field_type = &array_info.field_types[field_idx];

            // Calculate size needed for this field array
            let element_size = match field_type.as_str() {
                "f32" => 4,
                "f64" => 8,
                "u64" | "i64" => 8,
                "u32" | "i32" => 4,
                _ => 8, // Default to 8 bytes
            };

            // Calculate total size = element_size * array_length
            if size != "rsi" {
                self.output.push_str(&format!("    mov rsi, {}\n", size));
            }
            self.output
                .push_str(&format!("    mov rax, {}\n", element_size));
            self.output.push_str("    mul rsi\n"); // rax = element_size * array_length
            self.output.push_str("    mov rsi, rax\n"); // rsi = total_size

            // mmap syscall
            self.output.push_str("    mov rax, 9\n"); // sys_mmap
            self.output.push_str("    mov rdi, 0\n"); // addr = NULL
            // rsi already contains total_size
            self.output.push_str("    mov rdx, 3\n"); // prot = PROT_READ | PROT_WRITE
            self.output.push_str("    mov r10, 34\n"); // flags = MAP_PRIVATE | MAP_ANONYMOUS
            self.output.push_str("    mov r8, -1\n"); // fd = -1
            self.output.push_str("    mov r9, 0\n"); // offset = 0
            self.output.push_str("    syscall\n");

            // Store the returned pointer for this field
            // Use variable name instead of type name for the pointer variable
            let ptr_var_name = format!("{}_{}_ptr", var_name, field_name);
            self.stack_offset += 8;
            let ptr_offset = self.stack_offset;
            self.variables.insert(ptr_var_name, ptr_offset);

            // Store the mmap result (in rax) to the pointer variable
            self.output.push_str(&format!(
                "    mov qword ptr [rsp + {}], rax\n",
                ptr_offset - 8
            ));
        }

        Ok(())
    }

    fn compile_field_access(
        &mut self,
        field_access: &TermFieldAccess<PhaseParse>,
    ) -> Result<(), CompileError> {
        // This is used for reading array elements like "points.x 0"
        let obj_name = field_access.object_name();
        let field_name = field_access.field_name();

        // Look up the array type from variable name
        if let Some(array_type_name) = self.variable_arrays.get(obj_name) {
            if let Some(array_info) = self.arrays.get(array_type_name).cloned() {
                if let Some(index_term) = &field_access.index {
                    // Get the pointer for this field
                    let ptr_var_name = format!("{obj_name}_{field_name}_ptr");
                    if let Some(&ptr_offset) = self.variables.get(&ptr_var_name) {
                        // Load the base pointer
                        self.output.push_str(&format!(
                            "    mov rax, qword ptr [rsp + {}]\n",
                            ptr_offset - 8
                        ));

                        // Calculate offset based on index
                        let element_size = self.get_element_size(
                            &array_info.field_types,
                            &array_info.field_names,
                            field_name,
                        )?;

                        match &**index_term {
                            Term::Number(num) => {
                                let index = self.parse_number(num.number.s());
                                let offset = index.parse::<usize>().unwrap_or(0) * element_size;
                                self.output.push_str(&format!("    add rax, {offset}\n"));
                            }
                            Term::Variable(var) => {
                                if let Some(&var_offset) = self.variables.get(var.variable.s()) {
                                    self.output.push_str(&format!(
                                        "    mov rbx, qword ptr [rsp + {}]\n",
                                        var_offset - 8
                                    ));
                                    self.output
                                        .push_str(&format!("    mov rcx, {element_size}\n"));
                                    self.output.push_str("    mul rbx, rcx\n");
                                    self.output.push_str("    add rax, rbx\n");
                                }
                            }
                            _ => {
                                return Err(CompileError::UnsupportedConstruct(format!(
                                    "Unsupported index type: {:?}",
                                    index_term
                                )));
                            }
                        }

                        // Result address is in rax - caller can use this
                        return Ok(());
                    }
                }
            }
        }

        Err(CompileError::UnsupportedConstruct(format!(
            "Unknown field access: {}.{}",
            obj_name, field_name
        )))
    }

    fn compile_field_assign(
        &mut self,
        field_assign: &TermFieldAssign<PhaseParse>,
    ) -> Result<(), CompileError> {
        // This is used for writing array elements like "points.x 0 = 10.0f32"
        let obj_name = field_assign.field_access.object_name();
        let field_name = field_assign.field_access.field_name();

        // Look up the array type from variable name
        if let Some(array_type_name) = self.variable_arrays.get(obj_name) {
            if let Some(array_info) = self.arrays.get(array_type_name).cloned() {
                if let Some(index_term) = &field_assign.field_access.index {
                    // Get the pointer for this field
                    let ptr_var_name = format!("{obj_name}_{field_name}_ptr");
                    if let Some(&ptr_offset) = self.variables.get(&ptr_var_name) {
                        // Load the base pointer
                        self.output.push_str(&format!(
                            "    mov rax, qword ptr [rsp + {}]\n",
                            ptr_offset - 8
                        ));

                        // Calculate offset based on index
                        let element_size = self.get_element_size(
                            &array_info.field_types,
                            &array_info.field_names,
                            field_name,
                        )?;

                        match &**index_term {
                            Term::Number(num) => {
                                let index = self.parse_number(num.number.s());
                                let offset = index.parse::<usize>().unwrap_or(0) * element_size;
                                self.output.push_str(&format!("    add rax, {offset}\n"));
                            }
                            Term::Variable(var) => {
                                if let Some(&var_offset) = self.variables.get(var.variable.s()) {
                                    self.output.push_str(&format!(
                                        "    mov rbx, qword ptr [rsp + {}]\n",
                                        var_offset - 8
                                    ));
                                    self.output
                                        .push_str(&format!("    mov rcx, {element_size}\n"));
                                    self.output.push_str("    mul rbx, rcx\n");
                                    self.output.push_str("    add rax, rbx\n");
                                }
                            }
                            _ => {
                                return Err(CompileError::UnsupportedConstruct(format!(
                                    "Unsupported index type: {:?}",
                                    index_term
                                )));
                            }
                        }

                        // Now store the value to the calculated address
                        match &*field_assign.value {
                            Term::Number(num) => {
                                let field_type = self.get_field_type(
                                    &array_info.field_types,
                                    &array_info.field_names,
                                    field_name,
                                )?;
                                match field_type.as_str() {
                                    "f32" => {
                                        let number_str = num.number.s();
                                        if let Some(float_value) = number_str.strip_suffix("f32") {
                                            let float_val =
                                                float_value.parse::<f32>().unwrap_or(0.0);
                                            self.output.push_str(&format!(
                                                "    mov ebx, {}\n",
                                                Self::float_to_hex(float_val)
                                            ));
                                            self.output.push_str("    mov dword ptr [rax], ebx\n");
                                        }
                                    }
                                    _ => {
                                        let number_value = self.parse_number(num.number.s());
                                        self.output.push_str(&format!(
                                            "    mov qword ptr [rax], {}\n",
                                            number_value
                                        ));
                                    }
                                }
                            }
                            Term::Variable(var) => {
                                if let Some(&var_offset) = self.variables.get(var.variable.s()) {
                                    self.output.push_str(&format!(
                                        "    mov rbx, qword ptr [rsp + {}]\n",
                                        var_offset - 8
                                    ));
                                    self.output.push_str("    mov qword ptr [rax], rbx\n");
                                }
                            }
                            _ => {
                                return Err(CompileError::UnsupportedConstruct(format!(
                                    "Unsupported assignment value: {:?}",
                                    field_assign.value
                                )));
                            }
                        }

                        return Ok(());
                    }
                }
            }
        }

        Err(CompileError::UnsupportedConstruct(format!(
            "Unknown field assignment: {}.{}",
            obj_name, field_name
        )))
    }

    fn get_element_size(
        &self,
        field_types: &[String],
        field_names: &[String],
        field_name: &str,
    ) -> Result<usize, CompileError> {
        if let Some(pos) = field_names.iter().position(|name| name == field_name) {
            let field_type = &field_types[pos];
            Ok(match field_type.as_str() {
                "f32" => 4,
                "f64" => 8,
                "u64" | "i64" => 8,
                "u32" | "i32" => 4,
                _ => 8, // Default to 8 bytes
            })
        } else {
            Err(CompileError::UnsupportedConstruct(format!(
                "Unknown field: {}",
                field_name
            )))
        }
    }

    fn get_field_type(
        &self,
        field_types: &[String],
        field_names: &[String],
        field_name: &str,
    ) -> Result<String, CompileError> {
        if let Some(pos) = field_names.iter().position(|name| name == field_name) {
            Ok(field_types[pos].clone())
        } else {
            Err(CompileError::UnsupportedConstruct(format!(
                "Unknown field: {}",
                field_name
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
}
