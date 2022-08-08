use std::fs::File;
use std::io::{Error, BufReader, Seek, SeekFrom, Read, BufRead};

use clap::Parser;
use util::parser::parse;
use util::data_models::DictRecord;
use util::hashtable::{hash_function, rehash};
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

fn main() {
    let args = Args::parse();
    let tokens = get_query_tokens(&args.query);
    let dict_records = get_dict_records(&args.directory, &tokens).expect("Error reading dict file");
    println!("")
}