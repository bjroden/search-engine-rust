use std::{fs::{File, self}, io::Read, env};

use hashtable::HashTable;

use crate::parser::parse;

mod parser;
mod hashtable;

fn main() {
    let args: Vec<String> = env::args().collect();
    let indir = args.get(1).expect("Indir not given");
    let mut doc_ht: HashTable<usize> = HashTable::new(50000);
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
