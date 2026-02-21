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

        while !v.eof() {
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
                    break;
                }
            }

            if matched {
                continue;
            }

            let res = v.consume_while(|v| {
                if v.next().is_none() {
                    return Ok::<(), ()>(());
                }

                loop {
                    if v.eof() {
                        return Ok(());
                    }

                    let mut found = false;
                    for rule in &self.rules {
                        // Check if rule matches, but restore state if it does (by returning Err)
                        let matches = v.transact::<(), bool, _>(|v| {
                            if rule.pattern.consume(v) {
                                Err(true)
                            } else {
                                Ok(())
                            }
                        });

                        if matches == Err(true) {
                            found = true;
                            break;
                        }
                    }

                    if found {
                        return Ok(());
                    }

                    v.next();
                }
            });

            if let Ok(buffer) = res {
                tokens.push(Token::new(None, buffer));
            }
        }

        tokens
    }
}
