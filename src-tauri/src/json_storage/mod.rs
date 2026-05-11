use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use thiserror::Error;

use crate::board::types::{Board, BoardError};
use crate::column::types::{Column, ColumnError};
use crate::storage_port::{Storage, StorageError};
use crate::task::types::{Task, TaskError};

pub struct JsonStorage {
    data_dir: PathBuf,
}

#[async_trait::async_trait]
impl Storage for JsonStorage {
    async fn load_board(&self) -> Result<Board, StorageError> {
        let tasks_file = self.data_dir.join("tasks.json");
        match tokio::fs::read_to_string(&tasks_file).await {
            Ok(content) => {
                let board: Board = serde_json::from_str(&content)?;
                Ok(board)
            }
            Err(_) => {
                // File doesn't exist, create and save default board
                let default_board = self.default_board();
                self.save_board(&default_board).await?;
                Ok(default_board)
            }
        }
    }

    async fn save_board(&self, board: &Board) -> Result<(), StorageError> {
        tokio::fs::create_dir_all(&self.data_dir).await?;
        let tasks_file = self.data_dir.join("tasks.json");

        // Create backup if file exists
        if tasks_file.exists() {
            let timestamp = Utc::now().timestamp();
            let backup_file = self
                .data_dir
                .join(format!("tasks.json.backup.{}", timestamp));
            tokio::fs::copy(&tasks_file, &backup_file).await?;

            // Rotate backups to keep only last 5
            self.rotate_backups().await?;
        }

        let content = serde_json::to_string_pretty(board)?;
        tokio::fs::write(&tasks_file, content).await?;
        Ok(())
    }

    async fn create_task(&self, task: &Task) -> Result<Task, StorageError> {
        let mut board = self.load_board().await?;
        let created_task = Task::new(
            task.title.clone(),
            task.description.clone(),
            task.column_id.clone(),
            task.order,
        )?;
        board.tasks.push(created_task.clone());
        self.save_board(&board).await?;
        Ok(created_task)
    }

    async fn update_task(&self, task: &Task) -> Result<Task, StorageError> {
        let mut board = self.load_board().await?;
        let task_index = board
            .tasks
            .iter()
            .position(|t| t.id == task.id)
            .ok_or_else(|| StorageError::NotFound(format!("Task {} not found", task.id)))?;
        board.tasks[task_index] = task.clone();
        self.save_board(&board).await?;
        Ok(task.clone())
    }

    async fn delete_task(&self, id: &str) -> Result<(), StorageError> {
        let mut board = self.load_board().await?;
        let initial_len = board.tasks.len();
        board.tasks.retain(|t| t.id != id);
        if board.tasks.len() == initial_len {
            return Err(StorageError::NotFound(format!("Task {} not found", id)));
        }
        self.save_board(&board).await?;
        Ok(())
    }

    async fn create_column(&self, column: &Column) -> Result<Column, StorageError> {
        let mut board = self.load_board().await?;
        let created_column = Column::new(column.name.clone(), column.order)?;
        board.columns.push(created_column.clone());
        self.save_board(&board).await?;
        Ok(created_column)
    }

    async fn update_column(&self, column: &Column) -> Result<Column, StorageError> {
        let mut board = self.load_board().await?;
        let column_index = board
            .columns
            .iter()
            .position(|c| c.id == column.id)
            .ok_or_else(|| StorageError::NotFound(format!("Column {} not found", column.id)))?;
        board.columns[column_index] = column.clone();
        self.save_board(&board).await?;
        Ok(column.clone())
    }

    async fn delete_column(&self, id: &str) -> Result<(), StorageError> {
        let mut board = self.load_board().await?;
        let initial_len = board.columns.len();
        board.columns.retain(|c| c.id != id);
        if board.columns.len() == initial_len {
            return Err(StorageError::NotFound(format!("Column {} not found", id)));
        }
        self.save_board(&board).await?;
        Ok(())
    }
}

impl JsonStorage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn default_board(&self) -> Board {
        let columns = vec![
            Column::new("Backlog".to_string(), 0).unwrap(),
            Column::new("Todo".to_string(), 1).unwrap(),
            Column::new("In Progress".to_string(), 2).unwrap(),
            Column::new("Done".to_string(), 3).unwrap(),
        ];
        Board::new("Tasks".to_string(), columns).unwrap()
    }

    async fn rotate_backups(&self) -> Result<(), StorageError> {
        let mut backup_files: Vec<_> = std::fs::read_dir(&self.data_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("tasks.json.backup.") {
                    // Extract timestamp from filename
                    let parts: Vec<&str> = file_name.split('.').collect();
                    if parts.len() >= 4 {
                        if let Ok(timestamp) = parts[3].parse::<i64>() {
                            Some((file_name, timestamp))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Sort by timestamp (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Keep only 5 newest backups
        if backup_files.len() > 5 {
            for (filename, _) in backup_files.iter().skip(5) {
                let file_path = self.data_dir.join(filename);
                tokio::fs::remove_file(&file_path).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::types::Column;
    use tempfile::TempDir;

    #[tokio::test]
    async fn save_board_then_load_board_roundtrips_board_data() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();

        // This should fail initially since methods are not implemented
        storage.save_board(&board).await.unwrap();
        let loaded_board = storage.load_board().await.unwrap();

        assert_eq!(loaded_board.name, board.name);
        assert_eq!(loaded_board.columns.len(), board.columns.len());
        assert_eq!(loaded_board.schema_version, board.schema_version);
    }

    #[tokio::test]
    async fn load_board_returns_default_board_when_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        // File doesn't exist, should return default board
        let default_board = storage.load_board().await.unwrap();

        assert_eq!(default_board.name, "Tasks");
        assert_eq!(default_board.columns.len(), 4);
        assert_eq!(default_board.columns[0].name, "Backlog");
        assert_eq!(default_board.columns[1].name, "Todo");
        assert_eq!(default_board.columns[2].name, "In Progress");
        assert_eq!(default_board.columns[3].name, "Done");
        assert_eq!(default_board.schema_version, 1);
    }

    #[tokio::test]
    async fn save_board_creates_backup_before_write() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();

        // Save the board first
        storage.save_board(&board).await.unwrap();

        // Save again to create backup
        storage.save_board(&board).await.unwrap();

        // Check that backup file exists
        let backup_files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("tasks.json.backup.") {
                    Some(file_name)
                } else {
                    None
                }
            })
            .collect();

        assert!(!backup_files.is_empty(), "No backup files found");
    }

    #[tokio::test]
    async fn save_board_rotates_backups_keeping_last_5() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();

        // Save board first
        storage.save_board(&board).await.unwrap();

        // Save 7 more times to create 7 backups (should only keep last 5)
        for _ in 0..7 {
            storage.save_board(&board).await.unwrap();
            // Add delay to ensure different timestamps (1 second to avoid collision)
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }

        // Count backup files
        let backup_files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("tasks.json.backup.") {
                    Some(file_name)
                } else {
                    None
                }
            })
            .collect();

        // Debug: print all backup files found
        println!("Found {} backup files:", backup_files.len());
        for file in &backup_files {
            println!("  {}", file);
        }

        assert_eq!(backup_files.len(), 5, "Should keep exactly 5 backup files");
    }

    #[tokio::test]
    async fn load_board_returns_serialization_error_on_corrupt_json() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let tasks_file = temp_dir.path().join("tasks.json");
        tokio::fs::write(&tasks_file, "{ invalid json }")
            .await
            .unwrap();

        let result = storage.load_board().await;

        assert!(result.is_err());
        match result {
            Err(StorageError::Serialization(_)) => {}
            _ => panic!("Expected StorageError::Serialization, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn schema_version_persisted_and_read_back_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Do".to_string(), 0).unwrap();
        let mut board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        board.schema_version = 42;

        storage.save_board(&board).await.unwrap();
        let loaded_board = storage.load_board().await.unwrap();

        assert_eq!(loaded_board.schema_version, 42);
    }

    #[tokio::test]
    async fn create_task_adds_task_to_board() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let task = Task::new(
            "Test Task".to_string(),
            None,
            board.columns[0].id.clone(),
            0,
        )
        .unwrap();
        let created_task = storage.create_task(&task).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.tasks.len(), 1);
        assert_eq!(loaded_board.tasks[0].title, "Test Task");
    }

    #[tokio::test]
    async fn update_task_modifies_task_in_board() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let task = Task::new(
            "Test Task".to_string(),
            None,
            board.columns[0].id.clone(),
            0,
        )
        .unwrap();
        let created_task = storage.create_task(&task).await.unwrap();

        let mut updated_task = created_task.clone();
        updated_task.title = "Updated Task".to_string();
        updated_task.description = Some("Updated Description".to_string());

        let _returned_task = storage.update_task(&updated_task).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.tasks.len(), 1);
        assert_eq!(loaded_board.tasks[0].title, "Updated Task");
        assert_eq!(
            loaded_board.tasks[0].description,
            Some("Updated Description".to_string())
        );
    }

    #[tokio::test]
    async fn delete_task_removes_task_from_board() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let task = Task::new(
            "Test Task".to_string(),
            None,
            board.columns[0].id.clone(),
            0,
        )
        .unwrap();
        let created_task = storage.create_task(&task).await.unwrap();

        storage.delete_task(&created_task.id).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.tasks.len(), 0);
    }

    #[tokio::test]
    async fn create_column_adds_column_to_board() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let board = Board::new("Test Board".to_string(), vec![]).unwrap();
        storage.save_board(&board).await.unwrap();

        let column = Column::new("New Column".to_string(), 0).unwrap();
        let _created_column = storage.create_column(&column).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 1);
        assert_eq!(loaded_board.columns[0].name, "New Column");
    }

    #[tokio::test]
    async fn update_column_modifies_column_in_board() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("Original Column".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let mut updated_column = board.columns[0].clone();
        updated_column.name = "Updated Column".to_string();
        updated_column.order = 1;

        let _returned_column = storage.update_column(&updated_column).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 1);
        assert_eq!(loaded_board.columns[0].name, "Updated Column");
        assert_eq!(loaded_board.columns[0].order, 1);
    }

    #[tokio::test]
    async fn delete_column_removes_column_from_board() {
        let temp_dir = TempDir::new().unwrap();
        let storage = JsonStorage::new(temp_dir.path().to_path_buf());

        let column = Column::new("To Delete".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        storage.delete_column(&board.columns[0].id).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 0);
    }
}
