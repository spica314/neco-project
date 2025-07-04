use neco_felis_compile::compile_file_to_assembly;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assembly = compile_file_to_assembly("testcases/felis/single/exit_42.fe")?;
    println!("{assembly}");
    Ok(())
}
