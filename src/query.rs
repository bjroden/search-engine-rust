use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Error, BufReader, Seek, SeekFrom, Read, BufRead};
use std::{vec, num};

use clap::Parser;
use util::parser::parse;
use util::data_models::{DictRecord, PostRecord};
use util::hashtable::{hash_function, rehash, HashTable};
use util::constants::*;

mod util;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    directory: String,

    #[clap(short, long, value_parser)]
    query: String,

    #[clap(short, long, value_parser, default_value_t = 10)]
    num_results: usize
}

fn get_query_tokens(query: &str) -> Vec<String> {
    return parse(query);
}

fn get_dict_records(filedir: &str, tokens: &Vec<String>) -> Result<Vec<DictRecord>, Error> {
    let mut records = vec![];
    let file = File::open(format!("{filedir}/dict"))?;
    let mut reader = BufReader::new(file);
    for token in tokens {
        records.push(get_one_dict_record(&mut reader, token)?);
    }
    Ok(records)
}

fn get_one_dict_record(reader: &mut BufReader<File>, token: &str) -> Result<DictRecord, Error> {
    let mut hash = hash_function(token, &GLOB_HT_SIZE).unwrap();
    let mut record = read_one_dict_line_from_hash(reader, hash)?;
    while record.term != "!NULL" && record.term != token { 
        hash = rehash(&hash, &GLOB_HT_SIZE);
        record = read_one_dict_line_from_hash(reader, hash)?;
    }
    Ok(record)
}

fn read_one_dict_line_from_hash(reader: &mut BufReader<File>, hash: usize) -> Result<DictRecord, Error> {
    reader.seek(SeekFrom::Start((hash * DICT_RECORD_SIZE).try_into().unwrap()))?;
    let mut record_str = String::new();
    reader.read_line(&mut record_str)?;
    let split_record: Vec<&str> = record_str.split_whitespace().collect();
    let term = split_record[0];
    let num_docs = split_record[1].parse().unwrap();
    let start = split_record[2].parse().unwrap();
    Ok(DictRecord { term: term.to_string(), num_docs: num_docs, post_line_start: start })
}

fn make_query_ht(post_records: &Vec<PostRecord>, expected_docs: usize) -> HashTable<usize> {
    let mut query_ht = HashTable::new(expected_docs * 3);
    for record in post_records {
        query_ht.insert_combine(&record.doc_id.to_string(), record.weight);
    }
    query_ht
}

fn get_all_post_records(filedir: &str, dict_records: &Vec<DictRecord>) -> Result<Vec<PostRecord>, Error> {
    let file = File::open(format!("{filedir}/post"))?;
    let mut reader = BufReader::new(file);
    let mut post_records = vec![];
    for dict_record in dict_records {
        post_records.append(&mut get_term_post_records(&mut reader, &dict_record)?);
    }
    Ok(post_records)
}

fn get_term_post_records(reader: &mut BufReader<File>, dict_record: &DictRecord) -> Result<Vec<PostRecord>, Error> {
    let mut post_records = vec![];
    reader.seek(SeekFrom::Start((dict_record.post_line_start * POST_RECORD_SIZE).try_into().unwrap()))?;
    for _ in 0..dict_record.num_docs {
        let mut record_str = String::new();
        reader.read_line(&mut record_str)?;
        let split_record: Vec<&str> = record_str.split_whitespace().collect();
        let doc_id = split_record[0].to_string();
        let weight: usize = split_record[1].parse().unwrap();
        post_records.push(PostRecord { doc_id: doc_id, weight: weight })
    }
    Ok(post_records)
}

fn get_sorted_results(query_ht: HashTable<usize>, num_results: usize) -> Vec<PostRecord> {
    let mut heap: BinaryHeap<Reverse<PostRecord>> = BinaryHeap::new();
    for bucket in query_ht.get_buckets() {
        if let Some(entry) = bucket {
            match heap.peek() {
                Some(Reverse(heap_head)) => {
                    if heap_head.weight < entry.value {
                        if heap.len() + 1 > num_results { heap.pop(); }
                        heap.push(Reverse(PostRecord { doc_id: entry.key.to_string(), weight: entry.value }));
                    }
                }
                None => heap.push(Reverse(PostRecord { doc_id: entry.key.to_string(), weight: entry.value }))
            }
        }
    }
    let rev_sorted = heap.into_sorted_vec();
    let mut sorted = vec![];
    for rev_elem in rev_sorted {
        if let Reverse(elem) = rev_elem {
            sorted.push(elem.clone());
        }
    }
    sorted
}

fn main() {
    let args = Args::parse();
    let tokens = get_query_tokens(&args.query);
    let dict_records = get_dict_records(&args.directory, &tokens).expect("Error reading dict file");
    let expected_docs = dict_records.iter().fold(0, |sum, record| sum + record.num_docs);
    let post_records = get_all_post_records(&args.directory, &dict_records).expect("Error reading post file");
    let query_ht = make_query_ht(&post_records, expected_docs);
    let sorted_results = get_sorted_results(query_ht, args.num_results);
}