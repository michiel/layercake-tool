use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    csv_file: String,
}

fn main() {
    let args = Args::parse();
    println!("CSV file: {}", args.csv_file);
}
