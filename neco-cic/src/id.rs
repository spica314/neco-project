#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(usize);

impl Id {
    #[cfg(test)]
    pub fn new() -> Id {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Id(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct IdGenerator {
    next_id: usize,
}

impl IdGenerator {
    pub fn new() -> IdGenerator {
        IdGenerator { next_id: 0 }
    }

    pub fn generate_id(&mut self) -> Id {
        let res = Id(self.next_id);
        self.next_id += 1;
        res
    }
}
