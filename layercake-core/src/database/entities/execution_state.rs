use serde::{Deserialize, Serialize};

/// Execution state for pipeline entities (datasources and graphs)
///
/// This enum tracks the progress of data processing through the DAG pipeline.
/// It is used by both datasource and graph nodes to maintain consistent
/// execution state across the system.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionState {
    /// Node has not been executed yet
    NotStarted,
    /// Node is queued for execution
    Pending,
    /// Node is currently being processed
    Processing,
    /// Node execution completed successfully
    Completed,
    /// Node execution failed with an error
    Error,
}

impl ExecutionState {
    /// Convert ExecutionState to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionState::NotStarted => "not_started",
            ExecutionState::Pending => "pending",
            ExecutionState::Processing => "processing",
            ExecutionState::Completed => "completed",
            ExecutionState::Error => "error",
        }
    }

    /// Parse ExecutionState from database string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "not_started" => Some(ExecutionState::NotStarted),
            "pending" => Some(ExecutionState::Pending),
            "processing" => Some(ExecutionState::Processing),
            "completed" => Some(ExecutionState::Completed),
            "error" => Some(ExecutionState::Error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_state_conversion() {
        assert_eq!(ExecutionState::NotStarted.as_str(), "not_started");
        assert_eq!(ExecutionState::Completed.as_str(), "completed");
        assert_eq!(
            ExecutionState::from_str("processing"),
            Some(ExecutionState::Processing)
        );
        assert_eq!(ExecutionState::from_str("invalid"), None);
    }

    #[test]
    fn test_all_states_roundtrip() {
        let states = vec![
            ExecutionState::NotStarted,
            ExecutionState::Pending,
            ExecutionState::Processing,
            ExecutionState::Completed,
            ExecutionState::Error,
        ];

        for state in states {
            let str_repr = state.as_str();
            let parsed = ExecutionState::from_str(str_repr);
            assert_eq!(parsed, Some(state.clone()));
        }
    }
}
