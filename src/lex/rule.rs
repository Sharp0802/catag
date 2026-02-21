use crate::iter::{Buffer, Consume, Consumer};
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

    pub fn rules(&self) -> &Vec<Rule<'_, T>> {
        &self.rules
    }
}

impl<'a, T: Eq + Hash + 'static> Rules<'a, T> {
    pub fn lex<Tx: Consumer<'a, T>>(&self, v: &mut Tx) -> Vec<Token<Buffer<'a, T>>> {
        let mut tokens = Vec::new();
        
        'outer: while !v.eof() {
            let mut matched = false;
            for rule in &self.rules {
                let res = v.consume_while(|v| {
                    if rule.pattern.consume(v) {
                        Ok(())
                    } else {
                        Err(())
                    }
                });

                if let Ok(buffer) = res {
                    if !rule.ignore {
                        tokens.push(Token::new(Some(rule.id), buffer));
                    }

                    matched = true;
                    continue 'outer;
                }
            }

            if !matched {
                panic!("No rule matched at {:?}", v.at());
            }
        }

        tokens
    }
}
