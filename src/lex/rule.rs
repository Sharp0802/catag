use crate::iter::Consumer;
use crate::lex::pattern::Pattern;
use crate::lex::token::Token;
use std::hash::Hash;

#[derive(Debug)]
pub struct Rule<'a, T: Eq + Hash> {
    id: usize,
    pattern: Pattern<'a, T>,
    ignore: bool,
}

impl<'a, T: Eq + Hash> Rule<'a, T> {
    pub fn new(id: usize, pattern: Pattern<'a, T>, ignore: bool) -> Self {
        Self {
            id,
            pattern,
            ignore,
        }
    }
}

pub struct Rules<'a, T: Eq + Hash> {
    rules: Vec<Rule<'a, T>>,
}

impl<'a, T: Eq + Hash> Rules<'a, T> {
    pub fn new(rules: Vec<Rule<'a, T>>) -> Self {
        Self { rules }
    }

    pub fn lex<Tx: Consumer<'a, T>>(&self, v: &mut Tx) -> Vec<Token<String>> {
        todo!()
    }

    pub fn rules(&self) -> &Vec<Rule<T>> {
        &self.rules
    }
}
