mod data_loader;

use anyhow::Result;

fn load_and_process_data(filename: &str) -> Result<()> {
    let df = data_loader::load_tsv(filename)?;
    println!("Loaded DataFrame:\n{}", df);
    Ok(())
}

fn main() -> Result<()> {
    let filename = "path/to/your/file.tsv";
    load_and_process_data(filename)?;
    Ok(())
}
