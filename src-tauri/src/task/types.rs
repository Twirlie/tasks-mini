use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_TITLE_LEN: usize = 200;
pub const MAX_DESCRIPTION_LEN: usize = 2000;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub column_id: String,
    pub order: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(
        title: String,
        description: Option<String>,
        column_id: String,
        order: u32,
    ) -> Result<Self, TaskError> {
        if title.is_empty() {
            return Err(TaskError::Validation("Title cannot be empty".to_string()));
        }
        if title.len() > MAX_TITLE_LEN {
            return Err(TaskError::Validation(format!(
                "Title cannot exceed {} characters",
                MAX_TITLE_LEN
            )));
        }
        if let Some(ref desc) = description {
            if desc.len() > MAX_DESCRIPTION_LEN {
                return Err(TaskError::Validation(format!(
                    "Description cannot exceed {} characters",
                    MAX_DESCRIPTION_LEN
                )));
            }
        }
        let now = Utc::now();
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description,
            column_id,
            order,
            created_at: now,
            updated_at: now,
            completed_at: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_new_rejects_empty_title() {
        let result = Task::new("".to_string(), None, "col-1".to_string(), 0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::Validation(_)));
    }

    #[test]
    fn task_new_rejects_title_over_200_chars() {
        let long_title = "a".repeat(MAX_TITLE_LEN + 1);
        let result = Task::new(long_title, None, "col-1".to_string(), 0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::Validation(_)));
    }

    #[test]
    fn task_new_rejects_description_over_2000_chars() {
        let long_desc = "a".repeat(MAX_DESCRIPTION_LEN + 1);
        let result = Task::new("Title".to_string(), Some(long_desc), "col-1".to_string(), 0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::Validation(_)));
    }

    #[test]
    fn task_new_with_valid_inputs_succeeds() {
        let task = Task::new(
            "Test task".to_string(),
            Some("Description".to_string()),
            "col-1".to_string(),
            0,
        )
        .unwrap();
        assert!(!task.id.is_empty());
        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, Some("Description".to_string()));
        assert!(task.created_at <= chrono::Utc::now());
        assert!(task.updated_at <= chrono::Utc::now());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn task_new_with_no_description_sets_none() {
        let task = Task::new("Test task".to_string(), None, "col-1".to_string(), 0).unwrap();
        assert_eq!(task.description, None);
    }

    #[test]
    fn task_new_sets_order_value_correctly() {
        let task = Task::new("Test".to_string(), None, "col-1".to_string(), 0).unwrap();
        assert_eq!(task.order, 0);
        let task2 = Task::new("Test2".to_string(), None, "col-1".to_string(), 100).unwrap();
        assert_eq!(task2.order, 100);
    }
}
