/// Common utilities for CSV export operations
///
/// This module provides generic helpers to eliminate duplication across
/// CSV export functions.
use csv::Writer;
use std::error::Error;

/// Generic CSV exporter that handles the common pattern of:
/// 1. Creating a writer
/// 2. Writing headers
/// 3. Writing rows from items
/// 4. Converting to string
///
/// # Example
///
/// ```rust,ignore
/// let csv = export_to_csv(
///     graph.nodes.iter(),
///     &["id", "label", "layer"],
///     |node| vec![
///         node.id.to_string(),
///         node.label.clone(),
///         node.layer.clone(),
///     ],
/// )?;
/// ```
pub fn export_to_csv<T, F>(
    items: impl IntoIterator<Item = T>,
    headers: &[&str],
    row_fn: F,
) -> Result<String, Box<dyn Error>>
where
    F: Fn(T) -> Vec<String>,
{
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(headers)?;

    for item in items {
        let row = row_fn(item);
        wtr.write_record(&row)?;
    }

    let data = wtr.into_inner()?;
    String::from_utf8(data).map_err(Into::into)
}

/// Export items to CSV with automatic sorting
///
/// This helper collects items into a Vec, sorts them, then exports.
/// Useful when you need to guarantee output order.
///
/// The sort key function should return an owned value (not a reference)
/// to avoid lifetime issues.
///
/// # Example
///
/// ```rust,ignore
/// let csv = export_to_csv_sorted(
///     &graph.nodes,
///     &["id", "label"],
///     |node| node.id.clone(),  // Sort key (owned)
///     |node| vec![node.id.to_string(), node.label.clone()],
/// )?;
/// ```
pub fn export_to_csv_sorted<T, K, F, S>(
    items: &[T],
    headers: &[&str],
    sort_key: S,
    row_fn: F,
) -> Result<String, Box<dyn Error>>
where
    K: Ord,
    S: Fn(&T) -> K,
    F: Fn(&T) -> Vec<String>,
{
    // Collect references and sort
    let mut refs: Vec<_> = items.iter().collect();
    refs.sort_by_key(|item| sort_key(item));

    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(headers)?;

    for item in refs {
        let row = row_fn(item);
        wtr.write_record(&row)?;
    }

    let data = wtr.into_inner()?;
    String::from_utf8(data).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestItem {
        id: i32,
        name: String,
    }

    #[test]
    fn test_export_to_csv() {
        let items = vec![
            TestItem {
                id: 2,
                name: "Second".to_string(),
            },
            TestItem {
                id: 1,
                name: "First".to_string(),
            },
        ];

        let result = export_to_csv(items.iter(), &["id", "name"], |item| {
            vec![item.id.to_string(), item.name.clone()]
        })
        .expect("CSV export should succeed");

        assert!(result.contains("id,name"));
        assert!(result.contains("2,Second"));
        assert!(result.contains("1,First"));
    }

    #[test]
    fn test_export_to_csv_sorted() {
        let items = vec![
            TestItem {
                id: 2,
                name: "Second".to_string(),
            },
            TestItem {
                id: 1,
                name: "First".to_string(),
            },
        ];

        let result = export_to_csv_sorted(
            &items,
            &["id", "name"],
            |item| item.id,
            |item| vec![item.id.to_string(), item.name.clone()],
        )
        .expect("Sorted CSV export should succeed");

        // Verify order: First should come before Second
        let first_pos = result.find("1,First").expect("Should contain First");
        let second_pos = result.find("2,Second").expect("Should contain Second");
        assert!(first_pos < second_pos, "Items should be sorted by ID");
    }

    #[test]
    fn test_export_empty() {
        let items: Vec<TestItem> = vec![];

        let result = export_to_csv(items.iter(), &["id", "name"], |item| {
            vec![item.id.to_string(), item.name.clone()]
        })
        .expect("Empty CSV export should succeed");

        assert_eq!(result, "id,name\n");
    }
}
