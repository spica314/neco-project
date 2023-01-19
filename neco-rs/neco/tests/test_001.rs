use neco::{run_cli, CliContext};

#[test]
fn test_001() {
    let context = CliContext::Compile("../../library/wip/prop.fe".into());
    run_cli(context);
}
