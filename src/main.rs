use std::{fs::{File, self}, io::Read, env};

use crate::parser::parse;

mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let indir = args.get(1).expect("Indir not given");
    for file_path in fs::read_dir(indir).expect("Could not read indir") {
        println!("----- File Path: {:?} -----", file_path.as_ref().unwrap().file_name());
        let mut file = File::open(file_path.unwrap().path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let tokens = parse(&contents.as_str());
        for token in tokens {
            println!("{}", token);
        }
    }
}
