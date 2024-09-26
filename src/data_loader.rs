use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use polars::sql::SQLContext;

pub fn load_tsv(filename: &str) -> Result<DataFrame> {
    let path = Path::new(filename);
    LazyCsvReader::new(path)
        .with_has_header(true)
        .with_separator(b'\t')
        .finish()?
        .collect()
        .map_err(Into::into)
}

pub fn add_column_with_sql(df: &DataFrame, sql_query: &str, col_name: &str) -> Result<DataFrame> {
    let mut ctx = SQLContext::new();
    ctx.register("df", df.clone().lazy());

    // Execute the SQL query and get the resulting LazyFrame
    let result_lf = ctx.execute(sql_query)?;

    // Collect the LazyFrame into a DataFrame
    let result_df = result_lf.collect()?;

    // Extract the new column from the result
    let new_col = result_df.column(col_name)?;

    // Add the new column to the original DataFrame
    let df_with_new_col = df.hstack(&[new_col.clone()])?;

    Ok(df_with_new_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_column_with_sql() -> Result<()> {
        // Create a sample DataFrame
        let df = df! [
            "A" => [1, 2, 3],
            "B" => [4, 5, 6]
        ]?;

        // SQL query to add a new column
        let sql_query = "SELECT *, A + B AS sum FROM df";

        // Add the new column
        let result = add_column_with_sql(&df, sql_query, "sum")?;

        // Check if the new column was added
        assert!(result.column("sum").is_ok());

        // Check if the values in the new column are correct
        let sum_col = result.column("sum")?;
        assert_eq!(sum_col.get(0).unwrap(), AnyValue::Int32(5));
        assert_eq!(sum_col.get(1).unwrap(), AnyValue::Int32(7));
        assert_eq!(sum_col.get(2).unwrap(), AnyValue::Int32(9));

        Ok(())
    }
}
