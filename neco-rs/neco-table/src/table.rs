use std::collections::{hash_map::Entry, HashMap};

use crate::TableId;

pub struct Table<T> {
    map: HashMap<TableId, T>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn merge(mut table1: Table<T>, mut table2: Table<T>) -> Table<T> {
        if table1.len() >= table2.len() {
            for (k, v) in table2.map {
                table1.insert(k, v);
            }
            table1
        } else {
            for (k, v) in table1.map {
                table2.insert(k, v);
            }
            table2
        }
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
        assert_eq!(table.len(), 1);
        table.remove(id);
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_table_2() {
        let mut table = Table::<i64>::new();
        let id = TableId::new();
        *table.entry(id).or_insert_with(|| 0) += 1;
        *table.entry(id).or_insert_with(|| panic!()) += 1;
        assert_eq!(table.get(id), Some(&2));
    }

    #[test]
    fn test_table_merge_1() {
        let mut table = Table::<i64>::new();
        for i in 0..10000 {
            let mut table2 = Table::<i64>::new();
            for _ in 0..10 {
                let id = TableId::new();
                table2.insert(id, 0);
            }
            table = Table::merge(table, table2);

            let mut table2 = Table::<i64>::new();
            for _ in 0..10 {
                let id = TableId::new();
                table2.insert(id, 0);
            }
            table = Table::merge(table2, table);

            assert_eq!(table.len(), 20 * (i + 1));
        }
    }
}
