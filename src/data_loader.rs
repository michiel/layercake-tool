use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

pub fn load_tsv(filename: &str) -> Result<DataFrame> {
    let path = Path::new(filename);
    LazyCsvReader::new(path)
        .with_has_header(true)
        .with_separator(b'\t')
        .finish()?
        .collect()
        .map_err(Into::into)
}
