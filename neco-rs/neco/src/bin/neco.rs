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

fn main() {}
