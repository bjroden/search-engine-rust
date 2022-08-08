use core::num;
use std::{fs::{File, self, write, OpenOptions}, io::{Read, Error, Write, BufWriter}, env, iter::Map};

use util::data_models::{GlobHTBucket, PostRecord, DictRecord, MapRecord};

use crate::parser::parse;
use crate::util::hashtable::HashTable;
use crate::util::constants::*;

mod parser;
mod util;

fn tokenize_file(glob_ht: &mut HashTable<GlobHTBucket>, doc_ht: &mut HashTable<usize>, file_contents: &str, doc_id: usize) {
    let tokens = parse(file_contents);
    for token in tokens {
        doc_ht.insert_combine(token.as_str(), 1)
    }
    for bucket in doc_ht.get_buckets() {
        if let Some(entry) = bucket  {
            let file_record = GlobHTBucket::new(doc_id, entry.value);
            glob_ht.insert_combine(entry.key.as_str(), file_record);
        }
    }
    doc_ht.reset();
}

fn write_dict(outdir: &str, glob_ht: &HashTable<GlobHTBucket>) -> Result<(), Error> {
    let dict_file = OpenOptions::new().write(true).create(true).open(format!("{outdir}/dict"))?;
    let mut writer = BufWriter::new(dict_file);
    let mut count = 0;
    for bucket in glob_ht.get_buckets() {
        if let Some(entry) = bucket {
            let term = &entry.key;
            let num_docs = entry.value.get_num_docs();
            let post_line_start = count;
            writeln!(writer, "{} {} {}", term, num_docs, post_line_start)?;
            count += num_docs;
        }
    }
    Ok(())
}

fn write_post(outdir: &str, glob_ht: &HashTable<GlobHTBucket>, total_docs: usize) -> Result<(), Error> {
    let post_file = OpenOptions::new().write(true).create(true).open(format!("{outdir}/post"))?;
    let mut writer = BufWriter::new(post_file);
    for bucket in glob_ht.get_buckets() {
        if let Some(entry) = bucket {
            let idf = 1.0 + (total_docs as f64 / entry.value.get_num_docs() as f64).log10();
            for file in entry.value.get_files() {
                let doc_id = file.doc_id;
                let weight = (file.rtf as f64 * idf * WEIGHT_MULTIPLIER) as usize; 
                writeln!(writer, "{} {}", doc_id, weight)?;
            }
        }
    }
    Ok(())
}

fn write_map(outdir: &str, docs: Vec<MapRecord>) -> Result<(), Error> {
    let map_file = OpenOptions::new().write(true).create(true).open(format!("{outdir}/map"))?;
    let mut writer = BufWriter::new(map_file);
    for doc in docs {
        writeln!(writer, "{}", doc.file_name)?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let indir = args.get(1).expect("Indir not given");
    let outdir = args.get(2).expect("Outdir not given");
    let mut doc_ht: HashTable<usize> = HashTable::new(DOC_HT_SIZE);
    let mut glob_ht: HashTable<GlobHTBucket> = HashTable::new(GLOB_HT_SIZE);
    let mut map_files: Vec<MapRecord> = vec![];
    for (doc_id, file_path) in fs::read_dir(indir).expect("Could not read indir").enumerate() {
        let mut file = File::open(file_path.as_ref().unwrap().path()).unwrap();
        let file_name = file_path.as_ref().unwrap().file_name().into_string().unwrap();
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {
                tokenize_file(&mut glob_ht, &mut doc_ht, &contents, doc_id);
                map_files.push(MapRecord { doc_id: doc_id, file_name: file_name })
            }
            Err(e) => {
                println!("Error while opening file: {}", e);
                continue
            }
        };
    }
    write_dict(&outdir, &glob_ht).expect("Error writing dict file");
    write_post(&outdir, &glob_ht, map_files.len()).expect("Error writing post file");
    write_map(&outdir, map_files).expect("Error writing map file");
}
