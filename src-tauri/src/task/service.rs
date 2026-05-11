use crate::board::types::Board;
use crate::storage_port::Storage;
use crate::task::types::{Task, TaskError, MAX_DESCRIPTION_LEN, MAX_TITLE_LEN};
use chrono::Utc;

async fn load_and_modify_board<F>(storage: &dyn Storage, modifier: F) -> Result<Task, TaskError>
where
    F: FnOnce(&mut Board) -> Result<Task, TaskError>,
{
    let mut board = storage
        .load_board()
        .await
        .map_err(|e| TaskError::Storage(Box::new(e)))?;

    let task = modifier(&mut board)?;

    storage
        .save_board(&board)
        .await
        .map_err(|e| TaskError::Storage(Box::new(e)))?;

    Ok(task)
}

fn is_last_column(board: &Board, column_id: &str) -> bool {
    board
        .columns
        .last()
        .map_or(false, |col| col.id == column_id)
}

fn validate_task_input(title: &str, description: &Option<String>) -> Result<(), TaskError> {
    if title.is_empty() {
        return Err(TaskError::Validation("Title cannot be empty".to_string()));
    }
    if title.len() > MAX_TITLE_LEN {
        return Err(TaskError::Validation(format!(
            "Title cannot exceed {} characters",
            MAX_TITLE_LEN
        )));
    }
    if let Some(desc) = description {
        if desc.len() > MAX_DESCRIPTION_LEN {
            return Err(TaskError::Validation(format!(
                "Description cannot exceed {} characters",
                MAX_DESCRIPTION_LEN
            )));
        }
    }
    Ok(())
}

pub async fn create_task(
    storage: &dyn Storage,
    title: String,
    description: Option<String>,
    column_id: String,
) -> Result<Task, TaskError> {
    validate_task_input(&title, &description)?;

    load_and_modify_board(storage, |board| {
        // Validate column exists
        if !board.columns.iter().any(|c| c.id == column_id) {
            return Err(TaskError::ColumnNotFound(column_id));
        }

        // Find max order in target column
        let max_order = board
            .tasks
            .iter()
            .filter(|t| t.column_id == column_id)
            .map(|t| t.order)
            .max()
            .unwrap_or(0);

        // Create task with next order
        let task = Task::new(title, description, column_id, max_order + 1)?;

        // Add task to board
        board.tasks.push(task.clone());

        Ok(task)
    })
    .await
}

pub async fn read_tasks(storage: &dyn Storage) -> Result<Board, TaskError> {
    storage
        .load_board()
        .await
        .map_err(|e| TaskError::Storage(Box::new(e)))
}

pub async fn update_task(
    storage: &dyn Storage,
    id: &str,
    title: Option<String>,
    description: Option<String>,
) -> Result<Task, TaskError> {
    load_and_modify_board(storage, |board| {
        // Find task by id
        let task_index = board
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| TaskError::TaskNotFound(id.to_string()))?;

        // Get mutable reference to task
        let task = &mut board.tasks[task_index];

        // Validate and update title if provided
        if let Some(new_title) = title {
            validate_task_input(&new_title, &description)?;
            task.title = new_title;
        }

        // Update description if provided
        if let Some(new_description) = description {
            validate_task_input(&task.title, &Some(new_description.clone()))?;
            task.description = Some(new_description);
        }

        // Update timestamp
        task.updated_at = Utc::now();

        Ok(task.clone())
    })
    .await
}

pub async fn delete_task(storage: &dyn Storage, id: &str) -> Result<(), TaskError> {
    let mut board = storage
        .load_board()
        .await
        .map_err(|e| TaskError::Storage(Box::new(e)))?;

    let initial_len = board.tasks.len();
    board.tasks.retain(|t| t.id != id);
    if board.tasks.len() == initial_len {
        return Err(TaskError::TaskNotFound(id.to_string()));
    }

    storage
        .save_board(&board)
        .await
        .map_err(|e| TaskError::Storage(Box::new(e)))?;

    Ok(())
}

pub async fn move_task(
    storage: &dyn Storage,
    id: &str,
    column_id: String,
    order: u32,
) -> Result<Task, TaskError> {
    let mut board = storage
        .load_board()
        .await
        .map_err(|e| TaskError::Storage(Box::new(e)))?;

    let is_done = is_last_column(&board, &column_id);

    let task_index = board
        .tasks
        .iter()
        .position(|t| t.id == id)
        .ok_or_else(|| TaskError::TaskNotFound(id.to_string()))?;

    let task = &mut board.tasks[task_index];
    task.column_id = column_id.clone();
    task.order = order;

    // Set completed_at if moved to last column
    if is_done {
        task.completed_at = Some(Utc::now());
    }

    let moved_task = task.clone();

    storage
        .save_board(&board)
        .await
        .map_err(|e| TaskError::Storage(Box::new(e)))?;

    Ok(moved_task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_port::MockStorage;

    #[tokio::test]
    async fn create_task_rejects_empty_title() {
        let storage = MockStorage::new();
        let result = create_task(&storage, "".to_string(), None, "col-1".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::Validation(_)));
    }

    #[tokio::test]
    async fn create_task_with_valid_input_persists_and_returns_task() {
        let storage = MockStorage::new();

        // Set up a board with a column first
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let result = create_task(
            &storage,
            "Test Task".to_string(),
            Some("Description".to_string()),
            board.columns[0].id.clone(),
        )
        .await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, Some("Description".to_string()));
        assert_eq!(task.column_id, board.columns[0].id);
    }

    #[tokio::test]
    async fn read_tasks_returns_board_from_storage() {
        let storage = MockStorage::new();

        // Set up a board with a column first
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        let result = read_tasks(&storage).await;
        assert!(result.is_ok());
        let loaded_board = result.unwrap();
        assert_eq!(loaded_board.name, "Test Board");
        assert_eq!(loaded_board.columns.len(), 1);
    }

    #[tokio::test]
    async fn update_task_changes_title_and_sets_updated_at() {
        let storage = MockStorage::new();

        // Set up a board with a column first
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        // Create a task
        let task = create_task(
            &storage,
            "Original Title".to_string(),
            None,
            board.columns[0].id.clone(),
        )
        .await
        .unwrap();

        // Update the task
        let result = update_task(&storage, &task.id, Some("Updated Title".to_string()), None).await;
        assert!(result.is_ok());
        let updated_task = result.unwrap();
        assert_eq!(updated_task.title, "Updated Title");
        assert!(updated_task.updated_at > task.updated_at);
    }

    #[tokio::test]
    async fn delete_task_removes_task_from_board() {
        let storage = MockStorage::new();

        // Set up a board with a column first
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        // Create a task
        let task = create_task(
            &storage,
            "Test Task".to_string(),
            None,
            board.columns[0].id.clone(),
        )
        .await
        .unwrap();

        // Delete the task
        let result = delete_task(&storage, &task.id).await;
        assert!(result.is_ok());

        // Verify task is removed
        let loaded_board = read_tasks(&storage).await.unwrap();
        assert_eq!(loaded_board.tasks.len(), 0);
    }

    #[tokio::test]
    async fn move_task_to_done_column_sets_completed_at() {
        let storage = MockStorage::new();

        // Set up a board with columns including Done
        let col1 = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let col2 = crate::column::types::Column::new("Done".to_string(), 1).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![col1, col2]).unwrap();
        storage.save_board(&board).await.unwrap();

        // Create a task in the first column
        let task = create_task(
            &storage,
            "Test Task".to_string(),
            None,
            board.columns[0].id.clone(),
        )
        .await
        .unwrap();

        // Move task to Done column (last column)
        let result = move_task(&storage, &task.id, board.columns[1].id.clone(), 0).await;
        assert!(result.is_ok());
        let moved_task = result.unwrap();
        assert_eq!(moved_task.column_id, board.columns[1].id);
        assert!(moved_task.completed_at.is_some());
    }

    #[tokio::test]
    async fn update_task_on_non_existent_id_returns_task_not_found() {
        let storage = MockStorage::new();

        // Set up a board with a column
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        // Try to update non-existent task
        let result = update_task(
            &storage,
            "non-existent-id",
            Some("New Title".to_string()),
            None,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::TaskNotFound(_)));
    }

    #[tokio::test]
    async fn create_task_with_column_not_found_returns_column_not_found() {
        let storage = MockStorage::new();

        // Set up a board with a column
        let column = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board =
            crate::board::types::Board::new("Test Board".to_string(), vec![column]).unwrap();
        storage.save_board(&board).await.unwrap();

        // Try to create task with non-existent column id
        let result = create_task(
            &storage,
            "Test Task".to_string(),
            None,
            "non-existent-column".to_string(),
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::ColumnNotFound(_)));
    }
}
