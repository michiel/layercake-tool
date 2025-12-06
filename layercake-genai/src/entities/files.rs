use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub project_id: i32,
    pub filename: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub blob: Vec<u8>,
    pub checksum: String,
    pub created_by: Option<i32>,
    pub created_at: ChronoDateTimeUtc,
    pub indexed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
