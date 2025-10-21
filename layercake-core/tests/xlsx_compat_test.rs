#[cfg(test)]
mod tests {
    use calamine::{open_workbook_from_rs, Reader, Xlsx};
    use rust_xlsxwriter::*;
    use std::io::Cursor;

    #[test]
    fn test_rust_xlsxwriter_calamine_compatibility() {
        // Create a simple XLSX with rust_xlsxwriter
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        worksheet.set_name("test_sheet").unwrap();
        worksheet.write_string(0, 0, "key").unwrap();
        worksheet.write_string(0, 1, "value").unwrap();
        worksheet.write_string(1, 0, "id").unwrap();
        worksheet.write_number(1, 1, 123.0).unwrap();

        // Save to buffer
        let buffer = workbook.save_to_buffer().unwrap();

        println!("Generated XLSX buffer size: {} bytes", buffer.len());
        println!(
            "First 4 bytes: {:02x} {:02x} {:02x} {:02x}",
            buffer[0], buffer[1], buffer[2], buffer[3]
        );

        // Try to read with calamine
        let cursor = Cursor::new(&buffer);
        let mut xlsx: Xlsx<_> =
            open_workbook_from_rs(cursor).expect("Failed to open XLSX with calamine");

        // Get the sheet
        let sheet_names = xlsx.sheet_names();
        println!("Sheet names: {:?}", sheet_names);

        assert!(sheet_names.contains(&"test_sheet".to_string()));

        // Read the range
        let range = xlsx
            .worksheet_range("test_sheet")
            .expect("Failed to get worksheet");

        println!("Range dimensions: {}x{}", range.height(), range.width());

        // Verify data
        use calamine::Data;
        if let Some(cell) = range.get((0, 0)) {
            if let Data::String(s) = cell {
                assert_eq!(s, "key");
            } else {
                panic!("Expected string cell");
            }
        }
    }
}
