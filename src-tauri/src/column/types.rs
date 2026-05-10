use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_COLUMN_NAME_LEN: usize = 50;

#[derive(Error, Debug)]
pub enum ColumnError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub order: u32,
}

impl Column {
    pub fn new(name: String, order: u32) -> Result<Self, ColumnError> {
        if name.is_empty() {
            return Err(ColumnError::Validation(
                "Column name cannot be empty".to_string(),
            ));
        }
        if name.len() > MAX_COLUMN_NAME_LEN {
            return Err(ColumnError::Validation(format!(
                "Column name cannot exceed {} characters",
                MAX_COLUMN_NAME_LEN
            )));
        }
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            order,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_new_rejects_empty_name() {
        let result = Column::new("".to_string(), 0);
        assert!(result.is_err());
    }

    #[test]
    fn column_new_rejects_name_over_50_chars() {
        let long_name = "a".repeat(MAX_COLUMN_NAME_LEN + 1);
        let result = Column::new(long_name, 0);
        assert!(result.is_err());
    }

    #[test]
    fn column_new_with_valid_name_succeeds() {
        let col = Column::new("To Do".to_string(), 0).unwrap();
        assert!(!col.id.is_empty());
        assert_eq!(col.name, "To Do");
    }

    #[test]
    fn column_new_sets_correct_order_value() {
        let col = Column::new("To Do".to_string(), 5).unwrap();
        assert_eq!(col.order, 5);
    }
}
