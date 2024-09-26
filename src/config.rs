
struct ImportConfig {
    profiles: Vec<ImportProfile>,
}

enum Transformation {
    AddSQLColumn(String, String),
    FillColumnForward(String),
}

enum FileImportProfile {
    CSV(CSVImportParams),
}

struct CSVImportParams {
    skiprows: Option<usize>,
    separator: Option<char>,
}

struct ImportProfile {
    filename: String,
    tablename: String,
    transformations: Vec<Transformation>,
}