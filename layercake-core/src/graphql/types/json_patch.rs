use async_graphql::*;
use serde::{Deserialize, Serialize};

/// JSON Patch operation types (RFC 6902)
#[derive(Clone, Debug, Enum, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchOp {
    Add,
    Remove,
    Replace,
    Move,
    Copy,
    Test,
}

/// A single JSON Patch operation
#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
pub struct PatchOperation {
    /// Operation type
    pub op: PatchOp,

    /// JSON Pointer path to the target location
    pub path: String,

    /// Value for add, replace, and test operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Source path for move and copy operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

/// Input type for JSON Patch operations
#[derive(Clone, Debug, InputObject, Serialize, Deserialize)]
pub struct PatchOperationInput {
    pub op: PatchOp,
    pub path: String,
    #[graphql(default)]
    pub value: Option<serde_json::Value>,
    #[graphql(default)]
    pub from: Option<String>,
}

impl From<PatchOperationInput> for PatchOperation {
    fn from(input: PatchOperationInput) -> Self {
        PatchOperation {
            op: input.op,
            path: input.path,
            value: input.value,
            from: input.from,
        }
    }
}

/// Delta event containing JSON Patch operations for Plan DAG changes
#[derive(Clone, Debug, SimpleObject)]
pub struct PlanDagDeltaEvent {
    /// Project ID
    pub project_id: i32,

    /// New version after applying patch
    pub version: i32,

    /// User ID who made the change
    pub user_id: String,

    /// Timestamp of the change
    pub timestamp: String,

    /// JSON Patch operations describing the changes
    pub operations: Vec<PatchOperation>,
}

/// Result of applying a JSON Patch
#[derive(Clone, Debug, SimpleObject)]
pub struct PatchResult {
    /// Whether the patch was successfully applied
    pub success: bool,

    /// New version number if successful
    pub new_version: Option<i32>,

    /// Error messages if failed
    pub errors: Vec<String>,

    /// Conflicts detected (for version mismatch)
    pub conflicts: Vec<PatchConflict>,
}

/// Represents a conflict when applying a patch
#[derive(Clone, Debug, SimpleObject)]
pub struct PatchConflict {
    /// The operation that conflicted
    pub operation: PatchOperation,

    /// Expected version
    pub expected_version: i32,

    /// Actual current version
    pub actual_version: i32,

    /// Reason for the conflict
    pub reason: String,
}

/// Helper function to convert json-patch operations to our GraphQL types
/// Currently unused but kept for potential future use with manual patch construction
#[allow(dead_code)]
pub fn convert_json_patch_to_operations(patch: &json_patch::Patch) -> Vec<PatchOperation> {
    patch.0.iter().map(|op| {
        match op {
            json_patch::PatchOperation::Add(add_op) => PatchOperation {
                op: PatchOp::Add,
                path: add_op.path.to_string(),
                value: Some(add_op.value.clone()),
                from: None,
            },
            json_patch::PatchOperation::Remove(remove_op) => PatchOperation {
                op: PatchOp::Remove,
                path: remove_op.path.to_string(),
                value: None,
                from: None,
            },
            json_patch::PatchOperation::Replace(replace_op) => PatchOperation {
                op: PatchOp::Replace,
                path: replace_op.path.to_string(),
                value: Some(replace_op.value.clone()),
                from: None,
            },
            json_patch::PatchOperation::Move(move_op) => PatchOperation {
                op: PatchOp::Move,
                path: move_op.path.to_string(),
                value: None,
                from: Some(move_op.from.to_string()),
            },
            json_patch::PatchOperation::Copy(copy_op) => PatchOperation {
                op: PatchOp::Copy,
                path: copy_op.path.to_string(),
                value: None,
                from: Some(copy_op.from.to_string()),
            },
            json_patch::PatchOperation::Test(test_op) => PatchOperation {
                op: PatchOp::Test,
                path: test_op.path.to_string(),
                value: Some(test_op.value.clone()),
                from: None,
            },
        }
    }).collect()
}

/// Helper function to convert our GraphQL types to json-patch operations
/// Currently unused but kept for potential future use with manual patch construction
#[allow(dead_code)]
pub fn convert_operations_to_json_patch(operations: Vec<PatchOperation>) -> Result<json_patch::Patch, String> {
    let patch_ops: Result<Vec<json_patch::PatchOperation>, String> = operations.into_iter().map(|op| {
        match op.op {
            PatchOp::Add => {
                let value = op.value.ok_or_else(|| "Add operation requires a value".to_string())?;
                Ok(json_patch::PatchOperation::Add(json_patch::AddOperation {
                    path: op.path.parse().map_err(|e| format!("Invalid path: {}", e))?,
                    value,
                }))
            },
            PatchOp::Remove => {
                Ok(json_patch::PatchOperation::Remove(json_patch::RemoveOperation {
                    path: op.path.parse().map_err(|e| format!("Invalid path: {}", e))?,
                }))
            },
            PatchOp::Replace => {
                let value = op.value.ok_or_else(|| "Replace operation requires a value".to_string())?;
                Ok(json_patch::PatchOperation::Replace(json_patch::ReplaceOperation {
                    path: op.path.parse().map_err(|e| format!("Invalid path: {}", e))?,
                    value,
                }))
            },
            PatchOp::Move => {
                let from = op.from.ok_or_else(|| "Move operation requires a from path".to_string())?;
                Ok(json_patch::PatchOperation::Move(json_patch::MoveOperation {
                    path: op.path.parse().map_err(|e| format!("Invalid path: {}", e))?,
                    from: from.parse().map_err(|e| format!("Invalid from path: {}", e))?,
                }))
            },
            PatchOp::Copy => {
                let from = op.from.ok_or_else(|| "Copy operation requires a from path".to_string())?;
                Ok(json_patch::PatchOperation::Copy(json_patch::CopyOperation {
                    path: op.path.parse().map_err(|e| format!("Invalid path: {}", e))?,
                    from: from.parse().map_err(|e| format!("Invalid from path: {}", e))?,
                }))
            },
            PatchOp::Test => {
                let value = op.value.ok_or_else(|| "Test operation requires a value".to_string())?;
                Ok(json_patch::PatchOperation::Test(json_patch::TestOperation {
                    path: op.path.parse().map_err(|e| format!("Invalid path: {}", e))?,
                    value,
                }))
            },
        }
    }).collect();

    Ok(json_patch::Patch(patch_ops?))
}
