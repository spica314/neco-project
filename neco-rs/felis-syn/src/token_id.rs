#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenId(usize);

impl TokenId {
    pub fn id(&self) -> usize {
        self.0
    }
}

pub struct TokenIdGenerator {
    next_id: usize,
}

impl TokenIdGenerator {
    pub fn new() -> TokenIdGenerator {
        TokenIdGenerator { next_id: 0 }
    }
    pub fn generate(&mut self) -> TokenId {
        let res = TokenId(self.next_id);
        self.next_id += 1;
        res
    }
}
