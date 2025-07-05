use neco_felis_compile::compile_file_to_assembly;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let assembly = compile_file_to_assembly(&args[1])?;
    println!("{assembly}");
    Ok(())
}
