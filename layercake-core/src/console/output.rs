#![cfg(feature = "console")]

use std::cmp::max;

/// Table row wrapper for REPL display helpers.
#[derive(Debug)]
pub struct TableRow(pub Vec<String>);

impl TableRow {
    pub fn from(values: Vec<String>) -> Self {
        Self(values)
    }
}

/// Print a small ASCII table with column headers.
pub fn print_table(headers: &[&str], rows: &[TableRow]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

    for row in rows {
        for (idx, value) in row.0.iter().enumerate() {
            widths[idx] = max(widths[idx], value.len());
        }
    }

    let divider: String = widths
        .iter()
        .map(|w| "-".repeat(*w + 2))
        .collect::<Vec<_>>()
        .join("+");

    let render_row = |values: &[String]| {
        let mut parts = Vec::new();
        for (idx, value) in values.iter().enumerate() {
            parts.push(format!(" {:width$} ", value, width = widths[idx]));
        }
        parts.join("|")
    };

    println!("{}", divider);
    println!(
        "{}",
        render_row(&headers.iter().map(|h| h.to_string()).collect::<Vec<_>>())
    );
    println!("{}", divider);
    for row in rows {
        println!("{}", render_row(&row.0));
    }
    println!("{}", divider);
}

pub fn print_banner() {
    println!("Layercake console ready. Type 'help' for available commands, Ctrl+D to exit.");
}
