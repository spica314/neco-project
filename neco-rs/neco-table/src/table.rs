use std::collections::{hash_map::Entry, HashMap};

use crate::TableId;

#[derive(Debug, Clone)]
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

    pub fn merge_mut(&mut self, mut table: Table<T>) {
        if self.map.len() < table.len() {
            std::mem::swap(&mut self.map, &mut table.map);
        }
        for (k, v) in table.map {
            self.map.insert(k, v);
        }
    }
}

#[macro_export]
macro_rules! define_wrapper_of_table {
    ( $x:ident, $y:ident, $z:ident ) => {
        #[derive(Debug)]
        pub struct $x(pub(crate) $crate::Table<$z>);

        impl $x {
            pub fn new() -> $x {
                $x($crate::Table::<$z>::new())
            }

            pub fn get(&self, k: $y) -> Option<&$z> {
                self.0.get(k.into())
            }

            pub fn insert(&mut self, k: $y, v: $z) -> Option<$z> {
                self.0.insert(k.into(), v)
            }

            pub fn entry(
                &mut self,
                k: $y,
            ) -> std::collections::hash_map::Entry<'_, $crate::TableId, $z> {
                self.0.entry(k.into())
            }

            pub fn remove(&mut self, k: $y) -> Option<$z> {
                self.0.remove(k.into())
            }

            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn merge(table1: $x, table2: $x) -> $x {
                $x($crate::Table::<$z>::merge(table1.0, table2.0))
            }

            pub fn merge_mut(&mut self, table: $x) {
                self.0.merge_mut(table.0)
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::define_wrapper_of_table_id;

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

    #[test]
    fn test_table_merge_mut_1() {
        let mut table = Table::<i64>::new();
        for i in 0..10000 {
            let mut table2 = Table::<i64>::new();
            for _ in 0..10 {
                let id = TableId::new();
                table2.insert(id, 0);
            }
            table.merge_mut(table2);
            assert_eq!(table.len(), 10 * (i + 1));
        }
    }

    #[test]
    fn test_define_wrapper_of_table() {
        define_wrapper_of_table_id!(TableTestId);
        define_wrapper_of_table!(TableTest, TableTestId, i64);
        let id = TableTestId::new();
        let mut table = TableTest::new();
        table.insert(id, 42);
        let v = *table.get(id).unwrap();
        assert_eq!(v, 42);
    }
}
