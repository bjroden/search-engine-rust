use std::{fs::{File, self, OpenOptions}, io::{Error, Write, BufWriter}, env};

use util::{data_models::{GlobHTBucket, DocFrequency, MapRecord}, hashtable::TableEntry};
use encoding::{all::ISO_8859_1, Encoding, DecoderTrap};

use crate::util::parser::parse;
use crate::util::hashtable::HashTable;
use crate::util::constants::*;

mod util;

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

fn tokenize_file(glob_ht: &mut HashTable<GlobHTBucket>, doc_ht: &mut HashTable<usize>, stop_ht: &HashTable<usize>, file_path: &str, doc_id: usize) -> Result<(), Error> {
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
            glob_ht.insert_combine(entry.key.as_str(), file_record);
        }
    }
    doc_ht.reset();
    Ok(())
}

fn write_dict(outdir: &str, glob_ht: &HashTable<GlobHTBucket>) -> Result<(), Error> {
    let dict_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/dict"))?;
    let mut writer = BufWriter::new(dict_file);
    let mut count: usize = 0;
    for bucket in glob_ht.get_buckets() {
        count = write_dict_line(&mut writer, bucket, count)?;
    }
    Ok(())
}

fn write_dict_line(writer: &mut BufWriter<File>, bucket: &Option<TableEntry<GlobHTBucket>>, count: usize) -> Result<usize, Error> {
    let mut new_count = count;
    let (term, num_docs, post_line_start) = match bucket {
        Some(entry) => {
            if entry.value.is_rare() {
                ("!DELETED", 0, 0)
            } 
            else {
                new_count += entry.value.get_num_docs();
                (entry.key.as_str(), entry.value.get_num_docs(), count)
            }
        }
        None => ("!NULL", 0, 0)
    };
    writeln!(writer, 
            "{:<term_length$.term_length$} {:<num_docs_length$.num_docs_length$} {:<start_length$.start_length$}",
            term, num_docs.to_string(), post_line_start.to_string(),
            term_length = TERM_LENGTH,
            num_docs_length = NUMDOCS_LENGTH,
            start_length = START_LENGTH
    )?;
    Ok(new_count)
}

fn write_post(outdir: &str, glob_ht: &HashTable<GlobHTBucket>, total_docs: usize) -> Result<(), Error> {
    let post_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/post"))?;
    let mut writer = BufWriter::new(post_file);
    for bucket in glob_ht.get_buckets() {
        if let Some(entry) = bucket {
            if !entry.value.is_rare() {
                let idf = 1.0 + (total_docs as f64 / entry.value.get_num_docs() as f64).log10();
                for file in entry.value.get_files() {
                    write_post_line(&mut writer, file, idf)?;
                }
            }
        }
    }
    Ok(())
}

fn write_post_line(writer: &mut BufWriter<File>, file: &DocFrequency, idf: f64) -> Result<(), Error> {
    let doc_id = file.doc_id;
    let weight = (file.relative_term_frequency * idf * WEIGHT_MULTIPLIER) as usize; 
    writeln!(writer,
        "{:<doc_id_length$.doc_id_length$} {:<weight_length$.weight_length$}",
        doc_id.to_string(), weight.to_string(),
        doc_id_length = DOC_ID_LENGTH,
        weight_length = WEIGHT_LENGTH
    )?;
    Ok(())
}


fn write_map(outdir: &str, docs: Vec<MapRecord>) -> Result<(), Error> {
    let map_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/map"))?;
    let mut writer = BufWriter::new(map_file);
    for doc in docs {
        write_map_line(&mut writer, &doc.file_name)?;
    }
    Ok(())
}

fn write_map_line(writer: &mut BufWriter<File>, name: &str) -> Result<(), Error> {
    writeln!(writer, "{:<length$.length$}", name, length=MAP_NAME_LENGTH)?;
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
    let mut doc_ht: HashTable<usize> = HashTable::new(DOC_HT_SIZE);
    let mut glob_ht: HashTable<GlobHTBucket> = HashTable::new(GLOB_HT_SIZE);
    let stop_ht: HashTable<usize> = create_stop_ht(&stop_path).expect("Error opening stopfile");
    let mut map_files: Vec<MapRecord> = vec![];
    for (doc_id, file_path) in fs::read_dir(indir).expect("Could not read indir").enumerate() {
        let file_name = file_path.as_ref().unwrap().file_name().into_string().unwrap();
        let file_path_str = file_path.unwrap().path().to_str().unwrap().to_owned();
        map_files.push(MapRecord { doc_id: doc_id, file_name: file_name.clone() });
        if let Err(e) = tokenize_file(&mut glob_ht, &mut doc_ht, &stop_ht, &file_path_str, doc_id) {
            println!("Could not read file {}: {}", &file_name, e);
            continue;
        };
    }
    write_dict(&outdir, &glob_ht).expect("Error writing dict file");
    write_post(&outdir, &glob_ht, map_files.len()).expect("Error writing post file");
    write_map(&outdir, map_files).expect("Error writing map file");
}
