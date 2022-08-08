use std::{fs::{File, self}, io::Read, env, hash::Hash};

use crate::parser::parse;
use crate::util::hashtable::HashTable;
use crate::util::constants::*;

mod parser;
mod util;

fn main() {
    let args: Vec<String> = env::args().collect();
    let indir = args.get(1).expect("Indir not given");
    let mut doc_ht: HashTable<usize> = HashTable::new(DOC_HT_SIZE);
    for file_path in fs::read_dir(indir).expect("Could not read indir") {
        println!("----- File Path: {:?} -----", file_path.as_ref().unwrap().file_name());
        let mut file = File::open(file_path.unwrap().path()).unwrap();
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {
                let tokens = parse(&contents.as_str());
                for token in tokens {
                    doc_ht.insert_combine(token.as_str(), 1)
                }
                for bucket in doc_ht.get_buckets() {
                    if let Some(entry) = bucket  {
                        println!("{}: {}", entry.key, entry.value);
                    }
                }
                doc_ht.reset();
            }
            Err(e) => {
                println!("Error while opening file: {}", e);
                continue
            }
        };
    }
}
