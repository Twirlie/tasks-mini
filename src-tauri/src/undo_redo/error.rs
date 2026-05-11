use crate::column::types::ColumnError;
use crate::storage_port::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UndoRedoError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Column service error: {0}")]
    ColumnService(#[from] ColumnError),
    #[error("Nothing to undo")]
    NothingToUndo,
    #[error("Nothing to redo")]
    NothingToRedo,
}
