use neco::{run_cli, CliContext};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];
    let context = CliContext::Compile(filename.to_string());
    run_cli(context);
}
