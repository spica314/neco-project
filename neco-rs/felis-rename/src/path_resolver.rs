use std::collections::HashMap;

use crate::SerialId;

pub struct PathResolver {
    children: HashMap<String, PathResolver>,
    items: HashMap<String, SerialId>,
}

impl PathResolver {
    pub fn new() -> PathResolver {
        PathResolver {
            children: HashMap::new(),
            items: HashMap::new(),
        }
    }
    pub fn get(&self, path: &[String]) -> Option<SerialId> {
        assert!(!path.is_empty());
        if path.len() == 1 {
            self.items.get(&path[0]).cloned()
        } else {
            self.children
                .get(&path[0])
                .and_then(|child| child.get(&path[1..]))
        }
    }
    pub fn set(&mut self, path: &[String], id: SerialId) {
        assert!(!path.is_empty());
        if path.len() == 1 {
            self.items.insert(path[0].clone(), id);
        } else {
            self.children
                .entry(path[0].clone())
                .or_insert_with(PathResolver::new)
                .set(&path[1..], id);
        }
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::SerialId;

    use super::PathResolver;

    #[test]
    fn test_path_resolver_1() {
        let mut resolver = PathResolver::new();
        let id1 = SerialId::new();
        resolver.set(&["foo".to_string()], id1);
        assert_eq!(resolver.get(&["foo".to_string()]), Some(id1));
    }

    #[test]
    fn test_path_resolver_2() {
        let mut resolver = PathResolver::new();
        let id1 = SerialId::new();
        resolver.set(&["foo".to_string(), "bar".to_string()], id1);
        assert_eq!(
            resolver.get(&["foo".to_string(), "bar".to_string()]),
            Some(id1)
        );
    }
}
