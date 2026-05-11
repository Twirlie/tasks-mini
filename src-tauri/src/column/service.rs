use crate::board::types::Board;
use crate::column::types::{Column, ColumnError, MAX_COLUMN_NAME_LEN};
use crate::storage_port::Storage;

fn validate_column_name(name: &str) -> Result<(), ColumnError> {
    if name.len() > MAX_COLUMN_NAME_LEN {
        return Err(ColumnError::Validation(format!(
            "Column name cannot exceed {} characters",
            MAX_COLUMN_NAME_LEN
        )));
    }
    Ok(())
}

async fn load_board_mutate_save<F>(storage: &dyn Storage, mutator: F) -> Result<Board, ColumnError>
where
    F: FnOnce(&mut Board) -> Result<(), ColumnError>,
{
    let mut board = storage
        .load_board()
        .await
        .map_err(|e| ColumnError::Storage(Box::new(e)))?;
    mutator(&mut board)?;
    storage
        .save_board(&board)
        .await
        .map_err(|e| ColumnError::Storage(Box::new(e)))?;
    Ok(board)
}

pub async fn add_column(storage: &dyn Storage, name: String) -> Result<Column, ColumnError> {
    validate_column_name(&name)?;

    let mut board = storage
        .load_board()
        .await
        .map_err(|e| ColumnError::Storage(Box::new(e)))?;

    // Check for duplicate name (case-insensitive)
    let name_lower = name.to_lowercase();
    for col in &board.columns {
        if col.name.to_lowercase() == name_lower {
            return Err(ColumnError::Validation(format!(
                "Column name '{}' already exists",
                name
            )));
        }
    }

    // Create new column with next order value
    let next_order = board.columns.len() as u32;
    let new_column = Column::new(name.clone(), next_order)?;

    // Add to board and persist
    board.columns.push(new_column.clone());
    storage
        .save_board(&board)
        .await
        .map_err(|e| ColumnError::Storage(Box::new(e)))?;

    Ok(new_column)
}

pub async fn rename_column(
    storage: &dyn Storage,
    id: &str,
    name: String,
) -> Result<Column, ColumnError> {
    validate_column_name(&name)?;

    let id = id.to_string();
    let name_clone = name.clone();
    let board = load_board_mutate_save(storage, |board| {
        // Find column by id
        let column_index = board
            .columns
            .iter()
            .position(|col| col.id == id)
            .ok_or_else(|| ColumnError::ColumnNotFound(id.clone()))?;

        // Check for duplicate name (case-insensitive), excluding current column
        let name_lower = name.to_lowercase();
        for (i, col) in board.columns.iter().enumerate() {
            if i != column_index && col.name.to_lowercase() == name_lower {
                return Err(ColumnError::Validation(format!(
                    "Column name '{}' already exists",
                    name
                )));
            }
        }

        // Update column name
        board.columns[column_index].name = name_clone.clone();
        Ok(())
    })
    .await?;

    let column_index = board
        .columns
        .iter()
        .position(|col| col.id == id)
        .ok_or_else(|| ColumnError::ColumnNotFound(id))?;
    Ok(board.columns[column_index].clone())
}

pub async fn delete_column(storage: &dyn Storage, id: &str) -> Result<(), ColumnError> {
    let id = id.to_string();
    load_board_mutate_save(storage, |board| {
        // Find column by id first
        let column_index = board
            .columns
            .iter()
            .position(|col| col.id == id)
            .ok_or_else(|| ColumnError::ColumnNotFound(id.clone()))?;

        // Guard: cannot delete last column
        if board.columns.len() == 1 {
            return Err(ColumnError::CannotDeleteLastColumn);
        }

        // Get first column id for task relocation
        let first_column_id = board.columns[0].id.clone();

        // Move all tasks from deleted column to first column
        for task in &mut board.tasks {
            if task.column_id == id {
                task.column_id = first_column_id.clone();
            }
        }

        // Remove the column
        board.columns.remove(column_index);
        Ok(())
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::Board;
    use crate::column::types::Column;
    use crate::storage_port::MockStorage;

    #[tokio::test]
    async fn add_column_rejects_name_over_50_chars() {
        let storage = MockStorage::new();
        let long_name = "a".repeat(MAX_COLUMN_NAME_LEN + 1);
        let result = add_column(&storage, long_name).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ColumnError::Validation(_)));
    }

    #[tokio::test]
    async fn add_column_rejects_duplicate_name() {
        let storage = MockStorage::new();
        let existing_column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![existing_column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let result = add_column(&storage, "To Do".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ColumnError::Validation(_)));
    }

    #[tokio::test]
    async fn add_column_with_valid_name_persists_and_returns_column() {
        let storage = MockStorage::new();
        let existing_column = Column::new("Backlog".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![existing_column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let result = add_column(&storage, "To Do".to_string()).await;
        assert!(result.is_ok());
        let column = result.unwrap();
        assert_eq!(column.name, "To Do");
        assert_eq!(column.order, 1);

        // Verify persistence
        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 2);
        assert_eq!(loaded_board.columns[1].name, "To Do");
    }

    #[tokio::test]
    async fn rename_column_updates_name_and_persists() {
        let storage = MockStorage::new();
        let column = Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let column_id = board.columns[0].id.clone();
        let result = rename_column(&storage, &column_id, "In Progress".to_string()).await;
        assert!(result.is_ok());
        let updated_column = result.unwrap();
        assert_eq!(updated_column.name, "In Progress");

        // Verify persistence
        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns[0].name, "In Progress");
    }

    #[tokio::test]
    async fn delete_column_moves_tasks_to_first_column() {
        use crate::task::types::Task;
        let storage = MockStorage::new();
        let col1 = Column::new("Backlog".to_string(), 0).unwrap();
        let col2 = Column::new("To Do".to_string(), 1).unwrap();
        let mut board = Board::new("Test Board".to_string(), vec![col1, col2]).unwrap();

        // Add tasks to second column
        let task1 = Task::new("Task 1".to_string(), None, board.columns[1].id.clone(), 0).unwrap();
        let task2 = Task::new("Task 2".to_string(), None, board.columns[1].id.clone(), 1).unwrap();
        board.tasks.push(task1);
        board.tasks.push(task2);
        storage.save_board(&board).await.unwrap();

        let col2_id = board.columns[1].id.clone();
        let result = delete_column(&storage, &col2_id).await;
        assert!(result.is_ok());

        // Verify tasks moved to first column
        let loaded_board = storage.load_board().await.unwrap();
        assert_eq!(loaded_board.columns.len(), 1);
        assert_eq!(loaded_board.tasks.len(), 2);
        assert_eq!(loaded_board.tasks[0].column_id, loaded_board.columns[0].id);
        assert_eq!(loaded_board.tasks[1].column_id, loaded_board.columns[0].id);
    }

    #[tokio::test]
    async fn delete_column_on_last_column_returns_cannot_delete_last_column() {
        let storage = MockStorage::new();
        let column = Column::new("Backlog".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let column_id = board.columns[0].id.clone();
        let result = delete_column(&storage, &column_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ColumnError::CannotDeleteLastColumn
        ));
    }

    #[tokio::test]
    async fn rename_column_on_missing_id_returns_column_not_found() {
        let storage = MockStorage::new();
        let column = Column::new("Backlog".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let result = rename_column(&storage, "non-existent-id", "New Name".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ColumnError::ColumnNotFound(_)
        ));
    }

    #[tokio::test]
    async fn delete_column_on_missing_id_returns_column_not_found() {
        let storage = MockStorage::new();
        let column = Column::new("Backlog".to_string(), 0).unwrap();
        let board = Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let result = delete_column(&storage, "non-existent-id").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ColumnError::ColumnNotFound(_)
        ));
    }
}
