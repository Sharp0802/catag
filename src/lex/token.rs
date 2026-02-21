#[derive(PartialEq, Eq, Debug)]
pub struct Token<T> {
    kind: Option<usize>,
    str: T,
}

impl<'a> Token<&'a str> {
    pub fn new(kind: Option<usize>, str: &'a str) -> Self {
        Self { kind, str }
    }
}
