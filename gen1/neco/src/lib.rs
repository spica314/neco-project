use std::{path::Path, process::Command};

use felis_rename::{
    path_table::construct_path_table_syn_file,
    rename_defs::{rename_defs_file, RenameDefContext},
    rename_uses::{rename_uses_file, RenameUseContext},
    setup_resolver_for_prelude,
};
use felis_syn::{decoration::UD, syn_file::SynFile, test_utils::parse_from_str};
use felis_type_checker::{
    retrieve::{retrieve_file, setup_retrieve_context},
    typing::typing_file,
};
use felis_x86_64::compile;
use neco_resolver::Resolver;

#[derive(Debug, Clone)]
pub enum CliContext {
    Compile(String),
}

pub fn run_cli(context: CliContext) {
    match context {
        CliContext::Compile(path) => {
            run_compile(&path);
        }
    }
}

fn run_compile(file_path: &str) {
    let s = std::fs::read_to_string(file_path).unwrap();
    let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
    let mut context = RenameDefContext::new();
    let file_2 = rename_defs_file(&mut context, &file).unwrap();
    let mut resolver = Resolver::new();
    let path_table = construct_path_table_syn_file(&file_2).unwrap();
    path_table.setup_resolver(file_2.ext.id, &mut resolver);
    let prelude_map = setup_resolver_for_prelude(&mut context, &mut resolver);
    let mut context2 = RenameUseContext::new(resolver, path_table);
    let file_3 = rename_uses_file(&mut context2, &file_2).unwrap();

    let mut context = setup_retrieve_context(&prelude_map);
    retrieve_file(&mut context, &file_3);
    // for v in context.type_checker.get_relations() {
    //     eprintln!("{:?}", v);
    // }
    context.type_checker.resolve();
    // for (k, v) in context.type_checker.get_all() {
    //     eprintln!("{:?}: {:?}", k, v);
    // }
    // for v in context.type_checker.get_relations() {
    //     eprintln!("{:?}", v);
    // }
    let file_4 = typing_file(&mut context, &file_3);

    let file_5 = felis_code_gen_prepared::prepare_code_gen_file(&file_4);

    let res = compile(context, file_5);
    let base_path = "/tmp/neco".to_string();
    {
        let base_path = Path::new(&base_path);
        if base_path.exists() {
            std::fs::remove_dir_all(base_path).unwrap();
        }
    }
    std::fs::create_dir(&base_path).unwrap();
    std::fs::write(format!("{}/main.s", base_path), res).unwrap();

    let status = Command::new("as")
        .args([
            "-o",
            &format!("{}/main.o", base_path),
            &format!("{}/main.s", base_path),
        ])
        .status()
        .expect("");
    assert_eq!(status.code(), Some(0));
    let status = Command::new("ld")
        .args(["-o", "./a.out", &format!("{}/main.o", base_path)])
        .status()
        .expect("");
    assert_eq!(status.code(), Some(0));
}
