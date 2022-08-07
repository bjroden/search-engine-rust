use crate::parser::parse;

mod parser;

fn main() {
    println!("Hello, world!");
    let tokens = parse("Create ridiculously fast lexers <body>1 1a2 13.5 <b>E</b>lephants.");
    for token in tokens {
        println!("{}", token);
    }
}
