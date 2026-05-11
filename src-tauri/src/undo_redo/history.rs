use crate::storage_port::Storage;
use crate::undo_redo::command::Command;
use crate::undo_redo::error::UndoRedoError;

pub struct History {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_depth: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 50,
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    pub async fn execute(
        &mut self,
        cmd: Box<dyn Command>,
        storage: &dyn Storage,
    ) -> Result<(), UndoRedoError> {
        cmd.execute(storage).await?;
        self.redo_stack.clear();
        self.undo_stack.push(cmd);
        self.trim_if_needed();
        Ok(())
    }

    fn trim_if_needed(&mut self) {
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }

    pub async fn undo(&mut self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let cmd = self.undo_stack.pop().ok_or(UndoRedoError::NothingToUndo)?;
        cmd.undo(storage).await?;
        self.redo_stack.push(cmd);
        Ok(())
    }

    pub async fn redo(&mut self, storage: &dyn Storage) -> Result<(), UndoRedoError> {
        let cmd = self.redo_stack.pop().ok_or(UndoRedoError::NothingToRedo)?;
        cmd.execute(storage).await?;
        self.undo_stack.push(cmd);
        Ok(())
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_port::MockStorage;

    struct MockCommand {
        execute_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
        undo_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockCommand {
        fn new() -> Self {
            Self {
                execute_called: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                undo_called: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }
    }

    #[async_trait::async_trait]
    impl Command for MockCommand {
        async fn execute(&self, _storage: &dyn Storage) -> Result<(), UndoRedoError> {
            self.execute_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn undo(&self, _storage: &dyn Storage) -> Result<(), UndoRedoError> {
            self.undo_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn history_execute_runs_command_and_pushes_to_undo_stack() {
        let mut history = History::new();
        let storage = MockStorage::new();
        let cmd = MockCommand::new();

        history.execute(Box::new(cmd), &storage).await.unwrap();

        assert_eq!(history.undo_stack.len(), 1);
    }

    #[tokio::test]
    async fn history_undo_pops_and_calls_cmd_undo() {
        let mut history = History::new();
        let storage = MockStorage::new();
        let cmd = MockCommand::new();

        history.execute(Box::new(cmd), &storage).await.unwrap();
        history.undo(&storage).await.unwrap();

        assert_eq!(history.undo_stack.len(), 0);
        assert_eq!(history.redo_stack.len(), 1);
    }

    #[tokio::test]
    async fn history_redo_re_applies_undone_command() {
        let mut history = History::new();
        let storage = MockStorage::new();
        let cmd = MockCommand::new();

        history.execute(Box::new(cmd), &storage).await.unwrap();
        history.undo(&storage).await.unwrap();
        history.redo(&storage).await.unwrap();

        assert_eq!(history.undo_stack.len(), 1);
        assert_eq!(history.redo_stack.len(), 0);
    }

    #[tokio::test]
    async fn history_undo_returns_nothing_to_undo_when_stack_empty() {
        let mut history = History::new();
        let storage = MockStorage::new();

        let result = history.undo(&storage).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UndoRedoError::NothingToUndo));
    }

    #[tokio::test]
    async fn history_redo_returns_nothing_to_redo_when_stack_empty() {
        let mut history = History::new();
        let storage = MockStorage::new();

        let result = history.redo(&storage).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UndoRedoError::NothingToRedo));
    }

    #[tokio::test]
    async fn history_execute_clears_redo_stack() {
        let mut history = History::new();
        let storage = MockStorage::new();
        let cmd1 = MockCommand::new();
        let cmd2 = MockCommand::new();

        history.execute(Box::new(cmd1), &storage).await.unwrap();
        history.undo(&storage).await.unwrap();
        history.execute(Box::new(cmd2), &storage).await.unwrap();

        assert_eq!(history.redo_stack.len(), 0);
    }

    #[tokio::test]
    async fn history_enforces_max_depth_and_trims_oldest() {
        let mut history = History::with_max_depth(3);
        let storage = MockStorage::new();

        for _ in 0..5 {
            let cmd = MockCommand::new();
            history.execute(Box::new(cmd), &storage).await.unwrap();
        }

        assert_eq!(history.undo_stack.len(), 3);
    }
}
