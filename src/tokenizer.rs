use core::num;
use std::{fs::{File, self, write, OpenOptions}, io::{Read, Error, Write, BufWriter}, env, iter::Map};

use util::data_models::{GlobHTBucket, DocFrequency, DictRecord, MapRecord};
use encoding::{all::ISO_8859_1, Encoding, DecoderTrap};

use crate::util::parser::parse;
use crate::util::hashtable::HashTable;
use crate::util::constants::*;

mod util;

fn tokenize_file(glob_ht: &mut HashTable<GlobHTBucket>, doc_ht: &mut HashTable<usize>, file_contents: &str, doc_id: usize) {
    let tokens = parse(file_contents);
    let mut token_count: usize = 0;
    for token in tokens {
        doc_ht.insert_combine(token.as_str(), 1);
        token_count += 1;
    }
    for bucket in doc_ht.get_buckets() {
        if let Some(entry) = bucket  {
            let raw_term_frequency: usize = entry.value;
            let relative_term_frequency: f64 = raw_term_frequency as f64 / token_count as f64;
            let file_record = GlobHTBucket::new(doc_id, raw_term_frequency, relative_term_frequency);
            glob_ht.insert_combine(entry.key.as_str(), file_record);
        }
    }
    doc_ht.reset();
}

fn write_dict(outdir: &str, glob_ht: &HashTable<GlobHTBucket>) -> Result<(), Error> {
    let dict_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/dict"))?;
    let mut writer = BufWriter::new(dict_file);
    let mut count: usize = 0;
    for bucket in glob_ht.get_buckets() {
        match bucket {
            Some(entry) => {
                let term = &entry.key;
                let num_docs = entry.value.get_num_docs();
                let post_line_start = count;
                let raw_term_frequency = entry.value.get_total_frequency();
                if num_docs <= 1 && raw_term_frequency <= 1 {
                    write_dict_line(&mut writer, "!DELETED", 0, 0)?; 
                } 
                else {
                    write_dict_line(&mut writer, term, num_docs, post_line_start)?; 
                    count += num_docs
                }
            }
            None => write_dict_line(&mut writer, "!NULL", 0, 0)?
        }
    }
    Ok(())
}

fn write_dict_line(writer: &mut BufWriter<File>, term: &str, num_docs: usize, start: usize) -> Result<(), Error> {
    writeln!(writer, 
            "{:<term_length$.term_length$} {:<num_docs_length$.num_docs_length$} {:<start_length$.start_length$}",
            term, num_docs.to_string(), start.to_string(),
            term_length = TERM_LENGTH,
            num_docs_length = NUMDOCS_LENGTH,
            start_length = START_LENGTH
    )?;
    Ok(())
}

fn write_post(outdir: &str, glob_ht: &HashTable<GlobHTBucket>, total_docs: usize) -> Result<(), Error> {
    let post_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/post"))?;
    let mut writer = BufWriter::new(post_file);
    for bucket in glob_ht.get_buckets() {
        if let Some(entry) = bucket {
            let idf = 1.0 + (total_docs as f64 / entry.value.get_num_docs() as f64).log10();
            for file in entry.value.get_files() {
                let doc_id = file.doc_id;
                let weight = (file.relative_term_frequency * idf * WEIGHT_MULTIPLIER) as usize; 
                writeln!(writer,
                    "{:<doc_id_length$.doc_id_length$} {:<weight_length$.weight_length$}",
                    doc_id.to_string(), weight.to_string(),
                    doc_id_length = DOC_ID_LENGTH,
                    weight_length = WEIGHT_LENGTH
                )?;
            }
        }
    }
    Ok(())
}

fn write_map(outdir: &str, docs: Vec<MapRecord>) -> Result<(), Error> {
    let map_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/map"))?;
    let mut writer = BufWriter::new(map_file);
    for doc in docs {
        writeln!(writer, "{:<length$.length$}", doc.file_name, length=MAP_NAME_LENGTH)?;
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
        let file_name = file_path.as_ref().unwrap().file_name().into_string().unwrap();
        let bytes = match fs::read(file_path.as_ref().unwrap().path()) {
            Ok(contents) => contents,
            Err(e) => {
                println!("Could not open file {:?}: {}", file_name, e);
                continue;
            }
        };
        let contents = match ISO_8859_1.decode(&bytes, DecoderTrap::Ignore) {
            Ok(string) => string,
            Err(str) => str.into_owned()
        };
        tokenize_file(&mut glob_ht, &mut doc_ht, &contents, doc_id);
        map_files.push(MapRecord { doc_id: doc_id, file_name: file_name });
    }
    write_dict(&outdir, &glob_ht).expect("Error writing dict file");
    write_post(&outdir, &glob_ht, map_files.len()).expect("Error writing post file");
    write_map(&outdir, map_files).expect("Error writing map file");
}
