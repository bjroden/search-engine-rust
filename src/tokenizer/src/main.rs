use std::sync::{Mutex, Arc};
use std::thread;
use std::{fs, io::Error, env};

use util::data_models::{GlobHTBucket, MapRecord};
use encoding::{all::ISO_8859_1, Encoding, DecoderTrap};

use util::parser::parse;
use util::hashtable::HashTable;
use util::constants::*;
use util::write_query_files::write_output_files;

fn read_latin1_file(file_path: &str) -> Result<String, Error> {
    let bytes = fs::read(file_path)?;
    let contents = match ISO_8859_1.decode(&bytes, DecoderTrap::Ignore) {
        Ok(string) => string,
        Err(str) => str.into_owned()
    };
    Ok(contents)
}

fn create_stop_ht(stop_path: &str) -> Result<HashTable<usize>, Error> {
    let stop_words = parse(&read_latin1_file(stop_path)?);
    let mut stop_ht: HashTable<usize> = HashTable::new(stop_words.len() * 3);
    for word in stop_words {
        stop_ht.insert_combine(&word, 1);
    }
    Ok(stop_ht)
}

fn tokenize_file(glob_ht: Arc<Mutex<HashTable<GlobHTBucket>>>, stop_ht: &HashTable<usize>, file_path: &str, doc_id: usize) -> Result<(), Error> {
    let mut doc_ht: HashTable<usize> = HashTable::new(DOC_HT_SIZE);
    let file_contents = read_latin1_file(file_path)?;
    let tokens = parse(&file_contents);
    let mut token_count: usize = 0;
    for token in tokens {
        if !stop_ht.intable(&token) {
            doc_ht.insert_combine(token.as_str(), 1);
            token_count += 1;
        }
    }
    for bucket in doc_ht.get_buckets() {
        if let Some(entry) = bucket  {
            let raw_term_frequency: usize = entry.value;
            let relative_term_frequency: f64 = raw_term_frequency as f64 / token_count as f64;
            let file_record = GlobHTBucket::new(doc_id, raw_term_frequency, relative_term_frequency);
            let mut glob_ht = glob_ht.lock().unwrap();
            glob_ht.insert_combine(entry.key.as_str(), file_record);
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let indir = args.get(1).expect("Indir not given");
    let outdir = args.get(2).expect("Outdir not given");
    let stop_path = match args.get(3) {
        Some(path) => path,
        None => "./stopwords"
    };
    let glob_ht: Arc<Mutex<HashTable<GlobHTBucket>>> = Arc::new(Mutex::new(HashTable::new(GLOB_HT_SIZE)));
    let stop_ht: Arc<HashTable<usize>> = Arc::new(create_stop_ht(&stop_path).expect("Error opening stopfile"));
    let mut map_files: Vec<MapRecord> = vec![];
    let mut handles = vec![];
    for (doc_id, file_path) in fs::read_dir(indir).expect("Could not read indir").enumerate() {
        let file_name = file_path.as_ref().unwrap().file_name().into_string().unwrap();
        let file_path_str = file_path.unwrap().path().to_str().unwrap().to_owned();
        map_files.push(MapRecord { doc_id: doc_id, file_name: file_name.clone() });
        let glob_ht_clone = Arc::clone(&glob_ht);
        let stop_ht_clone = Arc::clone(&stop_ht);
        let handle = thread::spawn(move||{
            if let Err(e) = tokenize_file(glob_ht_clone, &stop_ht_clone, &file_path_str, doc_id) {
                println!("Could not read file {}: {}", &file_name, e);
            };
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    write_output_files(outdir, &glob_ht.lock().unwrap(), &map_files).unwrap();
}
