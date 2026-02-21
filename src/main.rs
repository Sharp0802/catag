use crate::iter::{IteratorConsumer, StringConsumer};
use crate::lex::std::parse_rules;
use crate::store::Store;

mod store;
mod lex;
mod error;
mod iter;

fn main() {
    let mut text: StringConsumer = r#"
    COMMA = ',';
    IP = ([0-9]{1,3} '.'){3,3}[0-9]{1,3};
    !WS = [ \t\n\f\v\r];
    "#.into();

    let mut store = Store::new();

    let rules = parse_rules(&mut text, &mut store).unwrap();

    for rule in rules.rules() {
        println!("{:?}", rule);
    }

    let mut iter: IteratorConsumer<char> = IteratorConsumer::new("hello, 192.168.0.1!".chars(), |at, _| at);
    for token in rules.lex(&mut iter) {
        println!("{:?}", token);
    }
}
