use clap::Parser;

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
    println!("{}", args.query);
}