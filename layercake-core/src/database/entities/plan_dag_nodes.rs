use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plan_dag_nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub plan_id: i32,
    pub node_type: String,
    pub position_x: f64,
    pub position_y: f64,
    pub source_position: Option<String>,
    pub target_position: Option<String>,
    pub metadata_json: String,
    pub config_json: String,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::plans::Entity",
        from = "Column::PlanId",
        to = "super::plans::Column::Id"
    )]
    Plans,
}

impl Related<super::plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            plan_id: ActiveValue::NotSet,
            node_type: ActiveValue::NotSet,
            position_x: ActiveValue::NotSet,
            position_y: ActiveValue::NotSet,
            source_position: ActiveValue::NotSet,
            target_position: ActiveValue::NotSet,
            metadata_json: ActiveValue::NotSet,
            config_json: ActiveValue::NotSet,
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        }
    }

    pub fn set_updated_at(mut self) -> Self {
        self.updated_at = Set(chrono::Utc::now());
        self
    }
}