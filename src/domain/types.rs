use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub columns: Vec<Column>,
    pub tasks: Vec<Task>,
    pub schema_version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub order: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub column_id: String,
    pub order: u32,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_board_serialization_round_trip() {
        let board = Board {
            id: "test-board".to_string(),
            name: "Test Board".to_string(),
            columns: vec![],
            tasks: vec![],
            schema_version: 1,
        };

        let json = serde_json::to_string(&board).expect("Failed to serialize Board");
        let deserialized: Board = serde_json::from_str(&json).expect("Failed to deserialize Board");

        assert_eq!(board.id, deserialized.id);
        assert_eq!(board.name, deserialized.name);
        assert_eq!(board.schema_version, deserialized.schema_version);
    }

    #[test]
    fn test_column_serialization_round_trip() {
        let column = Column {
            id: "test-column".to_string(),
            name: "Test Column".to_string(),
            order: 1,
        };

        let json = serde_json::to_string(&column).expect("Failed to serialize Column");
        let deserialized: Column =
            serde_json::from_str(&json).expect("Failed to deserialize Column");

        assert_eq!(column.id, deserialized.id);
        assert_eq!(column.name, deserialized.name);
        assert_eq!(column.order, deserialized.order);
    }

    #[test]
    fn test_task_serialization_round_trip() {
        let task = Task {
            id: "test-task".to_string(),
            title: "Test Task".to_string(),
            description: Some("Test description".to_string()),
            column_id: "test-column".to_string(),
            order: 1,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            completed_at: None,
        };

        let json = serde_json::to_string(&task).expect("Failed to serialize Task");
        let deserialized: Task = serde_json::from_str(&json).expect("Failed to deserialize Task");

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.title, deserialized.title);
        assert_eq!(task.description, deserialized.description);
        assert_eq!(task.column_id, deserialized.column_id);
        assert_eq!(task.order, deserialized.order);
        assert_eq!(task.created_at, deserialized.created_at);
        assert_eq!(task.updated_at, deserialized.updated_at);
        assert_eq!(task.completed_at, deserialized.completed_at);
    }

    #[test]
    fn test_module_imports() {
        // Test that we can import types from the domain module
        use crate::domain::{Board, Column, Task};

        let board = Board {
            id: "test-board".to_string(),
            name: "Test Board".to_string(),
            columns: vec![Column {
                id: "test-column".to_string(),
                name: "Test Column".to_string(),
                order: 1,
            }],
            tasks: vec![Task {
                id: "test-task".to_string(),
                title: "Test Task".to_string(),
                description: None,
                column_id: "test-column".to_string(),
                order: 1,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
                completed_at: None,
            }],
            schema_version: 1,
        };

        assert_eq!(board.id, "test-board");
        assert_eq!(board.columns.len(), 1);
        assert_eq!(board.tasks.len(), 1);
    }
}
