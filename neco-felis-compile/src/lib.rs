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
