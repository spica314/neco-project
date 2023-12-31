use neco::{run_cli, CliContext};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let file_path = &args[1];

    let cli_context = CliContext::Compile(file_path.clone());
    run_cli(cli_context);
}
