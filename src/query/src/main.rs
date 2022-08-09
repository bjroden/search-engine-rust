
use clap::Parser;
use util::read_query_files::make_query;
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

fn main() {
    let args = Args::parse();
    for (num, result) in make_query(&args.query, &args.directory, args.num_results).expect("Error reading files").iter().enumerate() {
        println!("{}: {} (weight: {})", num + 1, result.name, result.weight);
    }
}