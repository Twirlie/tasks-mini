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
                // File doesn't exist, return default board
                Ok(self.default_board())
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
        todo!("Implement create_task")
    }

    async fn update_task(&self, task: &Task) -> Result<Task, StorageError> {
        todo!("Implement update_task")
    }

    async fn delete_task(&self, id: &str) -> Result<(), StorageError> {
        todo!("Implement delete_task")
    }

    async fn create_column(&self, column: &Column) -> Result<Column, StorageError> {
        todo!("Implement create_column")
    }

    async fn update_column(&self, column: &Column) -> Result<Column, StorageError> {
        todo!("Implement update_column")
    }

    async fn delete_column(&self, id: &str) -> Result<(), StorageError> {
        todo!("Implement delete_column")
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
                    if parts.len() >= 3 {
                        if let Ok(timestamp) = parts[2].parse::<i64>() {
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
        for i in 0..7 {
            storage.save_board(&board).await.unwrap();
            // Add small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(10));
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
}
