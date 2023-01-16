use std::str::FromStr;

use neco::generate_context;
use neco_core_expr::CoreExpr;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];
    let s = std::fs::read_to_string(filename).unwrap();
    let expr = CoreExpr::from_str(&s).unwrap();
    eprintln!("expr = {expr:?}");
    let context = generate_context(&expr);
    eprintln!("context = {context:?}");
    // assert!(context.type_check());
}
