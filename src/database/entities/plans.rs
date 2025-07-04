use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveValue};
use serde::{Deserialize, Serialize};

// Simplified to String for now - will improve to enum later
pub type PlanStatus = String;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub plan_content: String, // JSON content (or YAML for backward compatibility)
    pub plan_schema_version: String, // Schema version for validation
    pub plan_format: String, // "json" or "yaml"
    pub dependencies: Option<String>, // JSON array of plan IDs
    pub status: PlanStatus,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id"
    )]
    Projects,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Parse plan content as JSON
    pub fn get_plan_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        match self.plan_format.as_str() {
            "json" => serde_json::from_str(&self.plan_content),
            "yaml" => {
                // Convert YAML to JSON for backward compatibility
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(&self.plan_content)
                    .map_err(|e| serde_json::Error::custom(format!("YAML parse error: {}", e)))?;
                let json_value = serde_json::to_value(yaml_value)
                    .map_err(|e| serde_json::Error::custom(format!("YAML to JSON conversion error: {}", e)))?;
                Ok(json_value)
            },
            _ => Err(serde_json::Error::custom(format!("Unsupported plan format: {}", self.plan_format)))
        }
    }

    /// Set plan content from JSON
    pub fn set_plan_json(&mut self, json_value: &serde_json::Value) -> Result<(), serde_json::Error> {
        self.plan_content = serde_json::to_string_pretty(json_value)?;
        self.plan_format = "json".to_string();
        Ok(())
    }

    /// Validate plan content against schema
    pub fn validate_plan_schema(&self) -> Result<(), String> {
        let json_value = self.get_plan_json()
            .map_err(|e| format!("Failed to parse plan content: {}", e))?;
        
        // TODO: Implement actual JSON schema validation
        // For now, just validate basic structure
        self.validate_basic_plan_structure(&json_value)
    }

    /// Basic validation for plan structure
    fn validate_basic_plan_structure(&self, plan: &serde_json::Value) -> Result<(), String> {
        let obj = plan.as_object()
            .ok_or("Plan must be a JSON object")?;

        // Check for required fields
        if !obj.contains_key("meta") {
            return Err("Plan must contain 'meta' field".to_string());
        }

        if let Some(meta) = obj.get("meta") {
            let meta_obj = meta.as_object()
                .ok_or("'meta' field must be an object")?;
            
            if !meta_obj.contains_key("name") {
                return Err("Plan meta must contain 'name' field".to_string());
            }
        }

        // Validate import section if present
        if let Some(import) = obj.get("import") {
            self.validate_import_section(import)?;
        }

        // Validate export section if present
        if let Some(export) = obj.get("export") {
            self.validate_export_section(export)?;
        }

        Ok(())
    }

    fn validate_import_section(&self, import: &serde_json::Value) -> Result<(), String> {
        let import_obj = import.as_object()
            .ok_or("'import' field must be an object")?;

        if let Some(profiles) = import_obj.get("profiles") {
            let profiles_array = profiles.as_array()
                .ok_or("'import.profiles' must be an array")?;

            for (i, profile) in profiles_array.iter().enumerate() {
                let profile_obj = profile.as_object()
                    .ok_or(format!("Import profile {} must be an object", i))?;

                if !profile_obj.contains_key("filename") {
                    return Err(format!("Import profile {} must contain 'filename'", i));
                }

                if !profile_obj.contains_key("filetype") {
                    return Err(format!("Import profile {} must contain 'filetype'", i));
                }
            }
        }

        Ok(())
    }

    fn validate_export_section(&self, export: &serde_json::Value) -> Result<(), String> {
        let export_obj = export.as_object()
            .ok_or("'export' field must be an object")?;

        if let Some(profiles) = export_obj.get("profiles") {
            let profiles_array = profiles.as_array()
                .ok_or("'export.profiles' must be an array")?;

            for (i, profile) in profiles_array.iter().enumerate() {
                let profile_obj = profile.as_object()
                    .ok_or(format!("Export profile {} must be an object", i))?;

                if !profile_obj.contains_key("filename") {
                    return Err(format!("Export profile {} must contain 'filename'", i));
                }

                if !profile_obj.contains_key("exporter") {
                    return Err(format!("Export profile {} must contain 'exporter'", i));
                }
            }
        }

        Ok(())
    }

    /// Convert YAML content to JSON format (for migration)
    pub fn migrate_yaml_to_json(&mut self) -> Result<(), String> {
        if self.plan_format == "json" {
            return Ok(()); // Already JSON
        }

        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&self.plan_content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let json_value = serde_json::to_value(yaml_value)
            .map_err(|e| format!("Failed to convert YAML to JSON: {}", e))?;

        self.set_plan_json(&json_value)
            .map_err(|e| format!("Failed to set JSON content: {}", e))?;

        self.plan_schema_version = "1.0.0".to_string();

        Ok(())
    }
}