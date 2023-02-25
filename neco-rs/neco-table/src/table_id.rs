use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct TableId {
    id: u64,
}

impl Default for TableId {
    fn default() -> Self {
        Self::new()
    }
}

impl TableId {
    pub fn new() -> TableId {
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let mut last = COUNTER.load(Ordering::Relaxed);
        loop {
            let Some(id) = last.checked_add(1) else {
                panic!();
            };

            match COUNTER.compare_exchange_weak(last, id, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => return TableId { id },
                Err(id) => last = id,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gen_table_id_1() {
        let id1 = TableId::new();
        let id2 = TableId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_gen_table_id_2() {
        let id1 = TableId::new();
        let thread = std::thread::spawn(|| TableId::new());
        let id2 = thread.join().unwrap();
        assert_ne!(id1, id2);
    }
}
