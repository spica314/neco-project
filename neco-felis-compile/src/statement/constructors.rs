use crate::{ArrayInfo, error::CompileError};
use neco_felis_syn::*;
use std::collections::HashMap;

pub fn compile_proc_constructor_call_with_var(
    constructor_call: &ProcTermConstructorCall<PhaseParse>,
    var_name: &str,
    arrays: &HashMap<String, ArrayInfo>,
    output: &mut String,
    stack_offset: &mut i32,
    variables: &mut HashMap<String, i32>,
    variable_arrays: &mut HashMap<String, String>,
) -> Result<(), CompileError> {
    let type_name = constructor_call.type_name.s();
    let method_name = constructor_call.method.s();
    let constructor_name = format!("{type_name}::{method_name}");

    if constructor_name.contains("::new_with_size") {
        // Look up array information
        if let Some(array_info) = arrays.get(type_name).cloned() {
            // Register the variable to array type mapping
            variable_arrays.insert(var_name.to_string(), type_name.to_string());
            // Get the size argument
            let size_arg = if !constructor_call.args.is_empty()
                && let Some(arg) = constructor_call.args.first()
            {
                match arg {
                    ProcTerm::Number(num) => num.number.s().to_string(),
                    ProcTerm::Variable(var) => {
                        if let Some(&offset) = variables.get(var.variable.s()) {
                            // Load variable value into rsi for use by SoA allocation
                            output.push_str(&format!(
                                "    mov rsi, qword ptr [rbp - 8 - {}]\n",
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
                output,
                stack_offset,
                variables,
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
    constructor_call: &ProcTermConstructorCall<PhaseParse>,
    arrays: &HashMap<String, ArrayInfo>,
    output: &mut String,
    stack_offset: &mut i32,
    variables: &mut HashMap<String, i32>,
) -> Result<(), CompileError> {
    let type_name = constructor_call.type_name.s();
    let method_name = constructor_call.method.s();
    let constructor_name = format!("{type_name}::{method_name}");

    if constructor_name.contains("::new_with_size") {
        // Look up array information
        if let Some(array_info) = arrays.get(type_name).cloned() {
            // Get the size argument
            let size_arg = if !constructor_call.args.is_empty()
                && let Some(arg) = constructor_call.args.first()
            {
                match arg {
                    ProcTerm::Number(num) => num.number.s().to_string(),
                    ProcTerm::Variable(var) => {
                        if let Some(&offset) = variables.get(var.variable.s()) {
                            // Load variable value into rsi for use by SoA allocation
                            output.push_str(&format!(
                                "    mov rsi, qword ptr [rbp - 8 - {}]\n",
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
                output,
                stack_offset,
                variables,
                &mut HashMap::new(),
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
