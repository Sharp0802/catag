#[derive(PartialEq, Eq, Debug)]
pub struct Token<T> {
    pub kind: Option<usize>,
    pub str: T,
}

impl<T> Token<T> {
    pub fn new(kind: Option<usize>, str: T) -> Self {
        Self { kind, str }
    }
}
