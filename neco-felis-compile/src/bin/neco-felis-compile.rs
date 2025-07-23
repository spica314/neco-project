use neco_felis_compile::{compile_file_to_assembly, compile_file_to_assembly_with_ptx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|s| s.contains("--ptx")) {
        let assembly = compile_file_to_assembly_with_ptx(&args[1])?;
        println!("{assembly}");
    } else {
        let assembly = compile_file_to_assembly(&args[1])?;
        println!("{assembly}");
    }
    Ok(())
}
