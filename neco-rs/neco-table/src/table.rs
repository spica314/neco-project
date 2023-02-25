use std::collections::{hash_map::Entry, HashMap};

use crate::TableId;

pub struct Table<T> {
    map: HashMap<TableId, T>,
}

impl<T> Table<T> {
    pub fn new() -> Table<T> {
        Table {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, k: TableId) -> Option<&T> {
        self.map.get(&k)
    }

    pub fn insert(&mut self, k: TableId, v: T) -> Option<T> {
        self.map.insert(k, v)
    }

    pub fn entry(&mut self, k: TableId) -> Entry<'_, TableId, T> {
        self.map.entry(k)
    }

    pub fn remove(&mut self, k: TableId) -> Option<T> {
        self.map.remove(&k)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_table_1() {
        let mut table = Table::<i64>::new();
        let id = TableId::new();
        assert!(table.get(id).is_none());
        table.insert(id, 1);
        assert_eq!(table.get(id), Some(&1));
        assert!(!table.is_empty());
        table.remove(id);
        assert!(table.is_empty());
    }

    #[test]
    fn test_table_2() {
        let mut table = Table::<i64>::new();
        let id = TableId::new();
        *table.entry(id).or_insert_with(|| 0) += 1;
        *table.entry(id).or_insert_with(|| panic!()) += 1;
        assert_eq!(table.get(id), Some(&2));
    }
}
