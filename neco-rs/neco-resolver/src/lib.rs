use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Scope<T> {
    map: HashMap<String, (usize, T)>,
}

impl<T> Scope<T> {
    fn new() -> Scope<T> {
        Scope {
            map: HashMap::new(),
        }
    }
    fn into_records(self) -> Vec<(String, T)> {
        let mut res: Vec<_> = self.map.into_iter().collect();
        res.sort_by(|x, y| x.1 .0.cmp(&y.1 .0));
        let res2: Vec<_> = res.into_iter().map(|x| (x.0, x.1 .1)).collect();
        res2
    }
    fn set(&mut self, name: String, record: T) -> Result<(), ()> {
        if self.map.contains_key(&name) {
            return Err(());
        }
        let i = self.map.len();
        self.map.insert(name, (i, record));
        Ok(())
    }
    fn get(&self, name: &str) -> Option<&T> {
        self.map.get(name).map(|(_, t)| t)
    }
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Resolver<T> {
    scopes: Vec<Scope<T>>,
}

impl<T> Resolver<T> {
    pub fn new() -> Resolver<T> {
        Resolver {
            scopes: vec![Scope::new()],
        }
    }
    pub fn enter_scope(&mut self) {
        let scope = Scope::new();
        self.scopes.push(scope);
    }
    pub fn leave_scope(&mut self) -> Vec<(String, T)> {
        let scope = self.scopes.pop().unwrap();
        scope.into_records()
    }
    #[deprecated]
    pub fn record(&mut self, name: String, record: T) -> Result<(), ()> {
        let top = self.scopes.last_mut().unwrap();
        top.set(name, record)
    }
    pub fn set(&mut self, name: String, record: T) -> Result<(), ()> {
        let top = self.scopes.last_mut().unwrap();
        top.set(name, record)
    }
    pub fn get(&self, name: &str) -> Option<&T> {
        for scope in self.scopes.iter().rev() {
            if let Some(res) = scope.get(name) {
                return Some(res);
            }
        }
        None
    }
}

impl<T> Default for Resolver<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_resolver_1() {
        let mut resolver = Resolver::<i64>::new();
        resolver.record("hoge".to_string(), 1).unwrap();
        assert_eq!(resolver.get("hoge"), Some(&1));
        resolver.record("fuga".to_string(), 2).unwrap();
        assert_eq!(resolver.get("fuga"), Some(&2));
        assert_eq!(resolver.get("hoge"), Some(&1));
    }

    #[test]
    fn test_resolver_2() {
        let mut resolver = Resolver::<i64>::new();
        resolver.record("hoge".to_string(), 1).unwrap();
        resolver.enter_scope();
        resolver.record("hoge".to_string(), 2).unwrap();
        assert_eq!(resolver.get("hoge"), Some(&2));
        let leaved = resolver.leave_scope();
        assert_eq!(leaved, vec![("hoge".to_string(), 2)]);
        assert_eq!(resolver.get("hoge"), Some(&1));
    }

    #[test]
    fn test_resolver_3() {
        let mut resolver = Resolver::<i64>::new();
        resolver.record("hoge".to_string(), 1).unwrap();
        assert!(resolver.record("hoge".to_string(), 1).is_err());
    }
}
