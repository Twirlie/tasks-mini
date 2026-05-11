use crate::column::service;
use crate::storage_port::Storage;
use crate::task::types::Task;
use crate::undo_redo::command::Command;
use crate::undo_redo::error::UndoRedoError;
use std::sync::Arc;

pub struct AddColumn {
    pub name: String,
    pub column_id: Arc<tokio::sync::Mutex<Option<String>>>,
}

impl AddColumn {
    pub fn new(name: String) -> Self {
        Self {
            name,
            column_id: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Command for AddColumn {
    async fn execute(&self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let column = service::add_column(storage, self.name.clone()).await?;
        *self.column_id.lock().await = Some(column.id);
        Ok(())
    }

    async fn undo(&self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let id = self.column_id.lock().await.clone();
        if let Some(id) = id {
            service::delete_column(storage, &id).await?;
        }
        Ok(())
    }
}

pub struct RenameColumn {
    pub column_id: String,
    pub old_name: Arc<tokio::sync::Mutex<Option<String>>>,
    pub new_name: String,
}

impl RenameColumn {
    pub fn new(column_id: String, new_name: String) -> Self {
        Self {
            column_id,
            old_name: Arc::new(tokio::sync::Mutex::new(None)),
            new_name,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Command for RenameColumn {
    async fn execute(&self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let board = storage.load_board().await?;
        let column = board
            .columns
            .iter()
            .find(|c| c.id == self.column_id)
            .ok_or(UndoRedoError::ColumnService(
                crate::column::types::ColumnError::ColumnNotFound(self.column_id.clone()),
            ))?;
        *self.old_name.lock().await = Some(column.name.clone());
        service::rename_column(storage, &self.column_id, self.new_name.clone()).await?;
        Ok(())
    }

    async fn undo(&self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let old_name = self.old_name.lock().await.clone();
        if let Some(old_name) = old_name {
            service::rename_column(storage, &self.column_id, old_name).await?;
        }
        Ok(())
    }
}
pub struct DeleteColumn {
    pub column_id: String,
    pub column_data: Arc<tokio::sync::Mutex<Option<(crate::column::types::Column, Vec<Task>)>>>,
}

impl DeleteColumn {
    pub fn new(column_id: String) -> Self {
        Self {
            column_id,
            column_data: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Command for DeleteColumn {
    async fn execute(&self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let mut board = storage.load_board().await?;
        let column = board
            .columns
            .iter()
            .find(|c| c.id == self.column_id)
            .ok_or(UndoRedoError::ColumnService(
                crate::column::types::ColumnError::ColumnNotFound(self.column_id.clone()),
            ))?;
        let tasks: Vec<Task> = board
            .tasks
            .iter()
            .filter(|t| t.column_id == self.column_id)
            .cloned()
            .collect();
        *self.column_data.lock().await = Some((column.clone(), tasks));

        board.columns.retain(|c| c.id != self.column_id);
        board.tasks.retain(|t| t.column_id != self.column_id);

        storage.save_board(&board).await?;
        Ok(())
    }

    async fn undo(&self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let data = self.column_data.lock().await.clone();
        if let Some((column, tasks)) = data {
            let mut board = storage.load_board().await?;

            board.columns.push(column);

            for task in tasks {
                board.tasks.push(task);
            }

            storage.save_board(&board).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::Board;
    use crate::column::types::Column;
    use crate::storage_port::MockStorage;

    #[tokio::test]
    async fn add_column_execute_adds() {
        let storage = MockStorage::new();
        let existing_column = Column::new("Backlog".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![existing_column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let cmd = AddColumn::new("To Do".to_string());
        cmd.execute(&storage).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 2);
        assert_eq!(loaded_board.columns[1].name, "To Do");
    }

    #[tokio::test]
    async fn add_column_undo_deletes() {
        let storage = MockStorage::new();
        let existing_column = Column::new("Backlog".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![existing_column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let cmd = AddColumn::new("To Do".to_string());
        cmd.execute(&storage).await.unwrap();
        cmd.undo(&storage).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 1);
    }

    #[tokio::test]
    async fn rename_column_execute_renames() {
        let storage = MockStorage::new();
        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let column_id = board.columns[0].id.clone();
        let cmd = RenameColumn::new(column_id, "In Progress".to_string());
        cmd.execute(&storage).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns[0].name, "In Progress");
    }

    #[tokio::test]
    async fn rename_column_undo_reverts() {
        let storage = MockStorage::new();
        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let column_id = board.columns[0].id.clone();
        let cmd = RenameColumn::new(column_id, "In Progress".to_string());
        cmd.execute(&storage).await.unwrap();
        cmd.undo(&storage).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns[0].name, "To Do");
    }

    #[tokio::test]
    async fn delete_column_execute_deletes_and_undo_restores() {
        use crate::task::types::Task;
        let storage = MockStorage::new();
        let col1 = Column::new("Backlog".to_string(), 0).unwrap();
        let col2 = Column::new("To Do".to_string(), 1).unwrap();
        let mut board = Board::new("Test Board".to_string(), vec![col1, col2]).unwrap();

        let task1 = Task::new("Task 1".to_string(), None, board.columns[1].id.clone(), 0).unwrap();
        board.tasks.push(task1.clone());
        storage.save_board(&board).await.unwrap();

        let col2_id = board.columns[1].id.clone();
        let cmd = DeleteColumn::new(col2_id);
        cmd.execute(&storage).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 1);
        assert_eq!(loaded_board.columns[0].name, "Backlog");

        cmd.undo(&storage).await.unwrap();

        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 2);
        assert!(loaded_board.columns.iter().any(|c| c.name == "To Do"));
        assert_eq!(loaded_board.tasks.len(), 1);
        assert_eq!(loaded_board.tasks[0].title, "Task 1");
    }
}
