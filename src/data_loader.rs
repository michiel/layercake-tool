use anyhow::Result;

use anyhow::anyhow;

use polars::prelude::*;
use std::path::Path;

use polars::sql::SQLContext;

use polars::prelude::FillNullStrategy;

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
pub fn fill_forward_columns(lf: LazyFrame, columns: Vec<String>) -> Result<LazyFrame> {
    let mut lf = lf;

    for col_name in columns {
        // Apply the forward fill on each column
        lf = lf.with_column(
            col(col_name.as_str())
                .fill_null_with_strategy(FillNullStrategy::Backward(None))
                .alias(&col_name),
        );
    }

    Ok(lf)
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
    #[test]
    fn test_fill_forward_columns() -> Result<()> {
        let df = df! [
            "A" => ["1", "2", "", "4", "5"],
            "B" => ["a", "", "", "d", "e"],
            "C" => ["1.1", "2.2", "3.3", "", "5.5"]
        ]?;

        let lf = df.lazy();
        let columns = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        let result = fill_forward_columns(lf, columns)?.collect()?;

        let expected = df! [
            "A" => [1, 2, 2, 4, 5],
            "B" => ["a", "a", "a", "d", "e"],
            "C" => [1.1, 2.2, 3.3, 3.3, 5.5]
        ]?;

        assert_eq!(result, expected);

        Ok(())
    }
}
