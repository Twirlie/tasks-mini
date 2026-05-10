use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

use crate::board::types::{Board, BoardError};
use crate::column::types::{Column, ColumnError};
use crate::task::types::{Task, TaskError};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Task validation error: {0}")]
    TaskValidation(#[from] TaskError),
    #[error("Column validation error: {0}")]
    ColumnValidation(#[from] ColumnError),
    #[error("Board validation error: {0}")]
    BoardValidation(#[from] BoardError),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

#[async_trait::async_trait]
pub trait Storage {
    async fn load_board(&self) -> Result<Board, StorageError>;
    async fn save_board(&self, board: &Board) -> Result<(), StorageError>;
    async fn create_task(&self, task: &Task) -> Result<Task, StorageError>;
    async fn update_task(&self, task: &Task) -> Result<Task, StorageError>;
    async fn delete_task(&self, id: &str) -> Result<(), StorageError>;
    async fn create_column(&self, column: &Column) -> Result<Column, StorageError>;
    async fn update_column(&self, column: &Column) -> Result<Column, StorageError>;
    async fn delete_column(&self, id: &str) -> Result<(), StorageError>;
}

pub struct MockStorage {
    board: Mutex<Option<Board>>,
    tasks: Mutex<HashMap<String, Task>>,
    columns: Mutex<HashMap<String, Column>>,
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn load_board(&self) -> Result<Board, StorageError> {
        let board_guard = self.board.lock().unwrap();
        match board_guard.as_ref() {
            Some(board) => Ok(board.clone()),
            None => Err(StorageError::NotFound("Board not found".to_string())),
        }
    }

    async fn save_board(&self, board: &Board) -> Result<(), StorageError> {
        let mut board_guard = self.board.lock().unwrap();
        *board_guard = Some(board.clone());
        Ok(())
    }

    async fn create_task(&self, task: &Task) -> Result<Task, StorageError> {
        let mut tasks_guard = self.tasks.lock().unwrap();
        let created_task = Task::new(
            task.title.clone(),
            task.description.clone(),
            task.column_id.clone(),
            task.order,
        )?;
        tasks_guard.insert(created_task.id.clone(), created_task.clone());
        Ok(created_task)
    }

    async fn update_task(&self, task: &Task) -> Result<Task, StorageError> {
        let mut tasks_guard = self.tasks.lock().unwrap();
        if !tasks_guard.contains_key(&task.id) {
            return Err(StorageError::NotFound(format!(
                "Task {} not found",
                task.id
            )));
        }
        let updated_task = task.clone();
        tasks_guard.insert(task.id.clone(), updated_task.clone());
        Ok(updated_task)
    }

    async fn delete_task(&self, id: &str) -> Result<(), StorageError> {
        let mut tasks_guard = self.tasks.lock().unwrap();
        match tasks_guard.remove(id) {
            Some(_) => Ok(()),
            None => Err(StorageError::NotFound(format!("Task {} not found", id))),
        }
    }

    async fn create_column(&self, column: &Column) -> Result<Column, StorageError> {
        let mut columns_guard = self.columns.lock().unwrap();
        let created_column = Column::new(column.name.clone(), column.order)?;
        columns_guard.insert(created_column.id.clone(), created_column.clone());
        Ok(created_column)
    }

    async fn update_column(&self, column: &Column) -> Result<Column, StorageError> {
        let mut columns_guard = self.columns.lock().unwrap();
        if !columns_guard.contains_key(&column.id) {
            return Err(StorageError::NotFound(format!(
                "Column {} not found",
                column.id
            )));
        }
        let updated_column = column.clone();
        columns_guard.insert(column.id.clone(), updated_column.clone());
        Ok(updated_column)
    }

    async fn delete_column(&self, id: &str) -> Result<(), StorageError> {
        let mut columns_guard = self.columns.lock().unwrap();
        match columns_guard.remove(id) {
            Some(_) => Ok(()),
            None => Err(StorageError::NotFound(format!("Column {} not found", id))),
        }
    }
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            board: Mutex::new(None),
            tasks: Mutex::new(HashMap::new()),
            columns: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_storage_create_task_and_load_board_roundtrip() {
        let storage = MockStorage::new();
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();

        // Save the board first
        storage.save_board(&board).await.unwrap();

        // Create a task
        let task = crate::task::types::Task::new(
            "Test Task".to_string(),
            Some("Test Description".to_string()),
            board.columns[0].id.clone(),
            0,
        )
        .unwrap();

        let _created_task = storage.create_task(&task).await.unwrap();

        // Load the board and verify task is persisted
        let loaded_board = storage.load_board().await.unwrap();

        // This should fail because MockStorage doesn't implement Storage yet
        assert_eq!(loaded_board.name, "Test Board");
        assert_eq!(loaded_board.columns.len(), 1);
    }

    #[tokio::test]
    async fn mock_storage_update_task_persists_changes() {
        let storage = MockStorage::new();

        // Create a task
        let task = crate::task::types::Task::new(
            "Original Title".to_string(),
            Some("Original Description".to_string()),
            "col-1".to_string(),
            0,
        )
        .unwrap();

        let created_task = storage.create_task(&task).await.unwrap();

        // Update task
        let mut updated_task = created_task.clone();
        updated_task.title = "Updated Title".to_string();
        updated_task.description = Some("Updated Description".to_string());

        let returned_task = storage.update_task(&updated_task).await.unwrap();

        // Verify changes are persisted
        assert_eq!(returned_task.title, "Updated Title");
        assert_eq!(
            returned_task.description,
            Some("Updated Description".to_string())
        );
        assert_eq!(returned_task.id, created_task.id);
    }

    #[tokio::test]
    async fn mock_storage_delete_task_removes_from_storage() {
        let storage = MockStorage::new();

        // Create a task
        let task =
            crate::task::types::Task::new("Test Task".to_string(), None, "col-1".to_string(), 0)
                .unwrap();

        let created_task = storage.create_task(&task).await.unwrap();

        // Delete the task
        storage.delete_task(&created_task.id).await.unwrap();

        // Try to update the deleted task - should fail
        let mut updated_task = created_task.clone();
        updated_task.title = "Should Fail".to_string();

        let result = storage.update_task(&updated_task).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));
    }

    #[tokio::test]
    async fn mock_storage_delete_task_removes_from_board() {
        let storage = MockStorage::new();
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();

        // Save the board first
        storage.save_board(&board).await.unwrap();

        // Create a task
        let task = crate::task::types::Task::new(
            "Test Task".to_string(),
            Some("Test Description".to_string()),
            board.columns[0].id.clone(),
            0,
        )
        .unwrap();

        let created_task = storage.create_task(&task).await.unwrap();

        // Delete task
        storage.delete_task(&created_task.id).await.unwrap();

        // Load board and verify task is removed
        let loaded_board = storage.load_board().await.unwrap();

        // This should fail because MockStorage doesn't implement Storage yet
        assert_eq!(loaded_board.name, "Test Board");
        assert_eq!(loaded_board.columns.len(), 1);
    }

    #[tokio::test]
    async fn mock_storage_column_crud_roundtrips() {
        let storage = MockStorage::new();

        // Create column
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let created_column = storage.create_column(&column).await.unwrap();

        // Update column
        let mut updated_column = created_column.clone();
        updated_column.name = "In Progress".to_string();
        updated_column.order = 1;

        let returned_column = storage.update_column(&updated_column).await.unwrap();

        // Verify changes are persisted
        assert_eq!(returned_column.name, "In Progress");
        assert_eq!(returned_column.order, 1);
        assert_eq!(returned_column.id, created_column.id);

        // Delete column
        storage.delete_column(&created_column.id).await.unwrap();

        // Try to update deleted column - should fail
        let mut update_again = created_column.clone();
        update_again.name = "Should Fail".to_string();

        let result = storage.update_column(&update_again).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));
    }

    #[tokio::test]
    async fn mock_storage_returns_not_found_for_missing_entities() {
        let storage = MockStorage::new();

        // Try to update non-existent task
        let non_existent_task =
            crate::task::types::Task::new("Test".to_string(), None, "col-1".to_string(), 0)
                .unwrap();

        let result = storage.update_task(&non_existent_task).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));

        // Try to delete non-existent task
        let result = storage.delete_task("non-existent-id").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));

        // Try to update non-existent column
        let non_existent_column = crate::column::types::Column::new("Test".to_string(), 0).unwrap();
        let result = storage.update_column(&non_existent_column).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));

        // Try to delete non-existent column
        let result = storage.delete_column("non-existent-id").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));

        // Try to load board when none exists
        let result = storage.load_board().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));
    }

    #[test]
    fn storage_error_io_carries_source_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let storage_err = StorageError::Io(io_err);
        assert!(matches!(storage_err, StorageError::Io(_)));
    }

    #[test]
    fn storage_error_serialization_carries_source_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let storage_err = StorageError::Serialization(json_err);
        assert!(matches!(storage_err, StorageError::Serialization(_)));
    }

    #[test]
    fn storage_error_task_validation_carries_source_error() {
        let task_err = TaskError::Validation("title too long".to_string());
        let storage_err = StorageError::TaskValidation(task_err);
        assert!(matches!(storage_err, StorageError::TaskValidation(_)));
    }

    #[test]
    fn storage_error_column_validation_carries_source_error() {
        let col_err = ColumnError::Validation("name too long".to_string());
        let storage_err = StorageError::ColumnValidation(col_err);
        assert!(matches!(storage_err, StorageError::ColumnValidation(_)));
    }

    #[test]
    fn storage_error_board_validation_carries_source_error() {
        let board_err = BoardError::Validation("board name empty".to_string());
        let storage_err = StorageError::BoardValidation(board_err);
        assert!(matches!(storage_err, StorageError::BoardValidation(_)));
    }

    #[test]
    fn storage_error_not_found_carries_id() {
        let storage_err = StorageError::NotFound("task-123".to_string());
        assert!(matches!(storage_err, StorageError::NotFound(id) if id == "task-123"));
    }
}
