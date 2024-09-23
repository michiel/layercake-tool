use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    csv_file: String,
}

fn main() {

    use polars::prelude::*;
    use anyhow::{Result, Context};

    let args = Args::parse();
    println!("CSV file: {}", args.csv_file);
}

let df = LazyCsvReader::new(&args.csv_file)
    .finish()
    .context("Failed to read CSV file")?;

println!("DataFrame created successfully");
