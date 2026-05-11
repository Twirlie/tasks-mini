use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BoardError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Duplicate column name: {0}")]
    DuplicateColumnName(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub columns: Vec<crate::column::types::Column>,
    pub tasks: Vec<crate::task::types::Task>,
    pub schema_version: u32,
}

impl Board {
    pub fn new(
        name: String,
        columns: Vec<crate::column::types::Column>,
    ) -> Result<Self, BoardError> {
        if name.is_empty() {
            return Err(BoardError::Validation(
                "Board name cannot be empty".to_string(),
            ));
        }
        let board = Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            columns,
            tasks: Vec::new(),
            schema_version: 1,
        };
        board.validate()?;
        Ok(board)
    }

    fn validate(&self) -> Result<(), BoardError> {
        let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for col in &self.columns {
            if !seen_names.insert(col.name.to_lowercase()) {
                return Err(BoardError::DuplicateColumnName(col.name.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_new_rejects_empty_name() {
        let result = Board::new("".to_string(), vec![]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BoardError::Validation(_)));
    }

    #[test]
    fn board_new_with_valid_name_and_columns_succeeds() {
        let col = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let board = Board::new("My Board".to_string(), vec![col]).unwrap();
        assert!(!board.id.is_empty());
        assert_eq!(board.name, "My Board");
        assert_eq!(board.columns.len(), 1);
        assert_eq!(board.schema_version, 1);
    }

    #[test]
    fn board_new_rejects_duplicate_column_names() {
        let col1 = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let col2 = crate::column::types::Column::new("To Do".to_string(), 1).unwrap();
        let result = Board::new("Board".to_string(), vec![col1, col2]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BoardError::DuplicateColumnName(_)
        ));
    }

    #[test]
    fn board_new_rejects_case_insensitive_duplicate_column_names() {
        let col1 = crate::column::types::Column::new("To Do".to_string(), 0).unwrap();
        let col2 = crate::column::types::Column::new("to do".to_string(), 1).unwrap();
        let result = Board::new("Board".to_string(), vec![col1, col2]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BoardError::DuplicateColumnName(_)
        ));
    }

    #[test]
    fn board_validate_returns_error_for_duplicate_names() {
        let col1 = crate::column::types::Column::new("Backlog".to_string(), 0).unwrap();
        let col2 = crate::column::types::Column::new("backlog".to_string(), 1).unwrap();
        let board = Board {
            id: "test".to_string(),
            name: "Test".to_string(),
            columns: vec![col1, col2],
            tasks: Vec::new(),
            schema_version: 1,
        };
        let result = board.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BoardError::DuplicateColumnName(_)
        ));
    }
}
