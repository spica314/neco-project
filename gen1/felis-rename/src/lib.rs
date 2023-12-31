use std::collections::HashMap;

use neco_resolver::Resolver;
use rename_defs::{DefId, RenameDefContext};

pub mod path_table;
pub mod rename_defs;
pub mod rename_uses;

pub fn setup_resolver_for_prelude(
    context: &mut RenameDefContext,
    resolver: &mut Resolver<DefId>,
) -> HashMap<String, DefId> {
    let mut res = HashMap::new();

    let id = context.new_id();
    resolver.set("__write_to_stdout".to_string(), id);
    res.insert("__write_to_stdout".to_string(), id);

    let id = context.new_id();
    resolver.set("__exit".to_string(), id);
    res.insert("__exit".to_string(), id);

    let id = context.new_id();
    resolver.set("__add_i64".to_string(), id);
    res.insert("__add_i64".to_string(), id);

    res
}
