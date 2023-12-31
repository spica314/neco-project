use std::process::Command;

use neco::{run_cli, CliContext};

#[test]
fn test_hello_world() {
    let file_path = "../../examples/hello-world/main.fe";

    let cli_context = CliContext::Compile(file_path.to_string());
    run_cli(cli_context);

    let out = Command::new("./a.out")
        .output()
        .expect("failed to execute process");
    assert_eq!(out.stdout, b"Hello, world!\n");
}

#[test]
fn test_let_string() {
    let file_path = "../../examples/let-string/main.fe";

    let cli_context = CliContext::Compile(file_path.to_string());
    run_cli(cli_context);

    let out = Command::new("./a.out")
        .output()
        .expect("failed to execute process");
    assert_eq!(out.stdout, b"Hello, world!\n");
}

#[test]
fn test_let_mut_string() {
    let file_path = "../../examples/let-mut-string/main.fe";

    let cli_context = CliContext::Compile(file_path.to_string());
    run_cli(cli_context);

    let out = Command::new("./a.out")
        .output()
        .expect("failed to execute process");
    assert_eq!(out.stdout, b"Hello, world!!\n");
}

#[test]
fn test_exit_42() {
    let file_path = "../../examples/exit-42/main.fe";

    let cli_context = CliContext::Compile(file_path.to_string());
    run_cli(cli_context);

    let out = Command::new("./a.out")
        .output()
        .expect("failed to execute process");
    assert_eq!(out.status.code(), Some(42));
}

#[test]
fn test_add_i64() {
    let file_path = "../../examples/i64-add/main.fe";

    let cli_context = CliContext::Compile(file_path.to_string());
    run_cli(cli_context);

    let out = Command::new("./a.out")
        .output()
        .expect("failed to execute process");
    assert_eq!(out.status.code(), Some(42));
}
