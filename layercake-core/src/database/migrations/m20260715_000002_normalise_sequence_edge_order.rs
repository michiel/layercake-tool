use sea_orm::Statement;
use sea_orm_migration::prelude::*;

/// Normalise `sequences.edge_order` to the canonical camelCase JSON shape.
///
/// The GraphQL boundary type used to serialise edge refs in snake_case
/// (`dataset_id`, `edge_id`, `note_position`) with PascalCase note positions
/// (`"Source"`), while the pipeline consumer expects camelCase (`datasetId`,
/// `edgeId`, `notePosition`) with lowercase positions (`"source"`). Rows written
/// before the fix parse to an empty vector, producing empty diagrams. This
/// rewrites the keys and note-position values to the canonical shape.
///
/// The key renames are unambiguous within `edge_order`. The value renames
/// (`"Source"`→`"source"` etc.) are scoped by first requiring the row to still
/// contain the old `note_position` key, so we only touch rows that were written
/// in the old shape.
#[derive(DeriveMigrationName)]
pub struct Migration;

const KEY_RENAMES: &[(&str, &str)] = &[
    ("\"dataset_id\"", "\"datasetId\""),
    ("\"edge_id\"", "\"edgeId\""),
];

const VALUE_RENAMES: &[(&str, &str)] = &[
    ("\"note_position\":\"Source\"", "\"notePosition\":\"source\""),
    ("\"note_position\":\"Target\"", "\"notePosition\":\"target\""),
    ("\"note_position\":\"Both\"", "\"notePosition\":\"both\""),
    // Spaced variant (serde_json default has no space, but be defensive).
    ("\"note_position\": \"Source\"", "\"notePosition\": \"source\""),
    ("\"note_position\": \"Target\"", "\"notePosition\": \"target\""),
    ("\"note_position\": \"Both\"", "\"notePosition\": \"both\""),
    // Any remaining note_position key (e.g. null) → notePosition.
    ("\"note_position\"", "\"notePosition\""),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        // Rename note-position values first (they still reference the old key),
        // then any leftover key, then the other keys.
        for (from, to) in VALUE_RENAMES.iter().chain(KEY_RENAMES.iter()) {
            let sql = format!(
                "UPDATE sequences SET edge_order = REPLACE(edge_order, '{from}', '{to}') WHERE edge_order LIKE '%{from}%'"
            );
            db.execute(Statement::from_string(backend, sql)).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Best-effort reverse to the old snake_case shape.
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        let reverses: &[(&str, &str)] = &[
            ("\"datasetId\"", "\"dataset_id\""),
            ("\"edgeId\"", "\"edge_id\""),
            ("\"notePosition\":\"source\"", "\"note_position\":\"Source\""),
            ("\"notePosition\":\"target\"", "\"note_position\":\"Target\""),
            ("\"notePosition\":\"both\"", "\"note_position\":\"Both\""),
            ("\"notePosition\"", "\"note_position\""),
        ];
        for (from, to) in reverses {
            let sql = format!(
                "UPDATE sequences SET edge_order = REPLACE(edge_order, '{from}', '{to}') WHERE edge_order LIKE '%{from}%'"
            );
            db.execute(Statement::from_string(backend, sql)).await?;
        }
        Ok(())
    }
}
