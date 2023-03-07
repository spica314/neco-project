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

#[macro_export]
macro_rules! define_wrapper_of_table_id {
    ( $x:ident ) => {
        #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
        pub struct $x(pub(crate) $crate::TableId);

        impl From<$x> for $crate::TableId {
            fn from(v: $x) -> $crate::TableId {
                v.0
            }
        }

        impl Default for $x {
            fn default() -> $x {
                $x::new()
            }
        }

        impl $x {
            pub fn new() -> $x {
                $x($crate::TableId::new())
            }
        }
    };
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

    #[test]
    fn test_define_wrapper_of_table_id() {
        define_wrapper_of_table_id!(TestId);
        let x = TestId::new();
        let y = TestId::new();
        assert_ne!(x, y);
    }
}
