use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub struct At {
    pub line: usize,
    pub offset: usize
}

impl Display for At {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.offset)
    }
}

impl At {
    pub fn new(line: usize, offset: usize) -> Self {
        Self { line, offset }
    }
}
