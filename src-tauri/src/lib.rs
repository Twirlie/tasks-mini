pub mod board;
pub mod column;
pub mod domain;
pub mod json_storage;
pub mod storage_port;
pub mod task;
pub mod undo_redo;

use crate::board::types::Board;
use crate::json_storage::JsonStorage;
use crate::task::service;
use crate::task::types::Task;
use std::path::PathBuf;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_board(storage: State<'_, Mutex<JsonStorage>>) -> Result<Board, String> {
    let storage = storage.lock().await;
    service::read_tasks(&*storage)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_task(
    title: String,
    description: Option<String>,
    column_id: String,
    storage: State<'_, Mutex<JsonStorage>>,
) -> Result<Task, String> {
    let storage = storage.lock().await;
    service::create_task(&*storage, title, description, column_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_task(
    id: String,
    title: Option<String>,
    description: Option<String>,
    storage: State<'_, Mutex<JsonStorage>>,
) -> Result<Task, String> {
    let storage = storage.lock().await;
    service::update_task(&*storage, &id, title, description)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_task(id: String, storage: State<'_, Mutex<JsonStorage>>) -> Result<(), String> {
    let storage = storage.lock().await;
    service::delete_task(&*storage, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn move_task(
    id: String,
    column_id: String,
    order: u32,
    storage: State<'_, Mutex<JsonStorage>>,
) -> Result<Task, String> {
    let storage = storage.lock().await;
    service::move_task(&*storage, &id, column_id, order)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let storage = JsonStorage::new(PathBuf::from("tasks.json"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(storage))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_board,
            create_task,
            update_task,
            delete_task,
            move_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
