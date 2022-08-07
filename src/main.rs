use crate::parser::parse;

mod parser;

fn main() {
    println!("Hello, world!");
    let tokens = parse("Create ridiculously fast lexers.");
    for token in tokens {
        println!("{}", token);
    }
}
