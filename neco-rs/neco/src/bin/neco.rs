// use felis_rename::{
//     path_table::construct_path_table_syn_file, rename_defs::rename_defs_file,
//     rename_uses::rename_uses_file, SerialId, SerialIdTable,
// };
// use felis_syn::{decoration::UD, syn_file::SynFile, test_utils::parse_from_str};
// use felis_term::{Term, TermAtom, TermStar, TypedTerm};
// use felis_type_check_2::{type_check_syn_file, TypedTermTable, TypedTermTableForAtom};
// use felis_type_defs::gen_type_def_table_file;
// use neco_resolver::Resolver;

// fn main() {
//     let args: Vec<_> = std::env::args().collect();

//     let s = std::fs::read_to_string(&args[1]).unwrap();
//     let file = parse_from_str::<SynFile<UD>>(&s).unwrap().unwrap();
//     /* def */
//     let defs_table = rename_defs_file(&file).unwrap();
//     /* path */
//     let path_table = construct_path_table_syn_file(&file, &defs_table).unwrap();
//     /* use */
//     let mut resolver = Resolver::new();
//     let prop_id = SerialId::new();
//     resolver.set("Prop".to_string(), prop_id);
//     path_table.setup_resolver(*defs_table.get(file.syn_tree_id()).unwrap(), &mut resolver);
//     let uses_table = rename_uses_file(&file, &defs_table, resolver, &path_table).unwrap();
//     /* merge def and use */
//     let mut rename_table = SerialIdTable::new();
//     rename_table.merge_mut(defs_table);
//     rename_table.merge_mut(uses_table);
//     let mut typed_term_table = TypedTermTable::new();
//     let mut typed_term_table_for_atom = TypedTermTableForAtom::new();
//     typed_term_table_for_atom.insert(
//         prop_id,
//         TypedTerm {
//             term: Term::Atom(TermAtom::new(2, prop_id)),
//             ty: TermStar::new(3).into(),
//         },
//     );
//     let type_def_table = gen_type_def_table_file(&file, &rename_table);

//     let type_check_res = type_check_syn_file(
//         &file,
//         &rename_table,
//         &mut typed_term_table,
//         &mut typed_term_table_for_atom,
//         &type_def_table,
//     );

//     if let Err(why) = type_check_res {
//         panic!("error: {:?}", why);
//     }

//     println!("Successfully passed type checking.")
// }

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

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let file_path = &args[1];

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
    context.type_checker.resolve();
    let file_4 = typing_file(&mut context, &file_3);

    let file_5 = felis_code_gen_prepared::prepare_code_gen_file(&file_4);

    let res = compile(file_5);
    let base_path = format!("/tmp/neco");
    {
        let base_path = Path::new(&base_path);
        if base_path.exists() {
            std::fs::remove_dir_all(&base_path).unwrap();
        }
    }
    std::fs::create_dir(&base_path).unwrap();
    std::fs::write(format!("{}/main.s", base_path), &res).unwrap();

    let status = Command::new("as")
        .args(&[
            "-o",
            &format!("{}/main.o", base_path),
            &format!("{}/main.s", base_path),
        ])
        .status()
        .expect("");
    assert_eq!(status.code(), Some(0));
    let status = Command::new("ld")
        .args(&["-o", "./a.out", &format!("{}/main.o", base_path)])
        .status()
        .expect("");
    assert_eq!(status.code(), Some(0));
}
