#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(usize);

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
