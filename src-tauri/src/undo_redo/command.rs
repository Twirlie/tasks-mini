use crate::storage_port::Storage;
use crate::undo_redo::error::UndoRedoError;

#[async_trait::async_trait]
pub trait Command: Send {
    async fn execute(&self, storage: &dyn Storage) -> Result<(), UndoRedoError>;
    async fn undo(&self, storage: &dyn Storage) -> Result<(), UndoRedoError>;
}
