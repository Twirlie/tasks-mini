//! Main application component for the Tasks Mini board.
//!
//! This component handles the overall application structure including:
//! - Board state management
//! - Loading and error states
//! - Keyboard shortcuts setup
//! - Theme switching
//! - Undo/redo functionality

use crate::components::{ColumnView, ThemeButton, UndoRedoControls};
use crate::domain::Board;
use crate::utils::keyboard;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

/// Tauri FFI bindings for communicating with the Rust backend.
///
/// These bindings allow the frontend to invoke commands on the Tauri backend
/// using the `invoke` function with the specified command and arguments.
#[wasm_bindgen]
extern "C" {
    /// Invoke a command on the Tauri backend
    ///
    /// # Arguments
    /// * `cmd` - The command name to invoke
    /// * `args` - JSON arguments to pass to the command
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Main application component.
///
/// This is the root component that manages the entire application state.
/// It handles board loading, error management, and renders the main UI.
#[component]
pub fn App() -> impl IntoView {
    // Reactive signals for application state
    let (board, set_board) = signal::<Option<Board>>(None); // Current board data
    let (loading, set_loading) = signal(true); // Loading state
    let (error, set_error) = signal::<Option<String>>(None); // Error state

    // Load board data from the Tauri backend.
    //
    // This function spawns an async task that:
    // 1. Sets loading state to true
    // 2. Clears any existing errors
    // 3. Invokes the "get_board" command on the backend
    // 4. Parses the response or sets an error state
    // 5. Sets loading state to false
    let load_board = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            let result = invoke("get_board", JsValue::NULL).await;
            match serde_wasm_bindgen::from_value::<Board>(result) {
                Ok(b) => set_board.set(Some(b)),
                Err(e) => set_error.set(Some(format!("Failed to parse board: {}", e))),
            }
            set_loading.set(false);
        });
    };

    // Load board data when the component mounts
    Effect::new(move |_| {
        load_board();
    });

    // Initialize keyboard shortcuts for board operations
    keyboard::setup_keyboard_shortcuts(set_board);

    view! {
        // Main application container with dark/light theme support
        <main class="min-h-screen bg-gray-900 dark:bg-gray-100 p-8 transition-colors duration-300 ease-in-out">
            // Header section with board title and controls
            <div class="max-w-7xl mx-auto mb-8">
                <div class="flex justify-between items-center">
                    // Dynamic board title - shows board name or default
                    {move || {
                        if let Some(b) = board.get() {
                            view! {
                                <h1 class="text-3xl font-bold text-gray-100 dark:text-gray-800 transition-colors duration-300 ease-in-out">{b.name}</h1>
                            }.into_any()
                        } else {
                            view! {
                                <h1 class="text-3xl font-bold text-gray-100 dark:text-gray-800 transition-colors duration-300 ease-in-out">"Tasks Mini"</h1>
                            }.into_any()
                        }
                    }}
                    // Control buttons for undo/redo and theme switching
                    <div class="flex gap-2">
                        <UndoRedoControls set_board />
                        <ThemeButton />
                    </div>
                </div>
            </div>
            // Main content area with conditional rendering based on state
            {move || {
                if loading.get() {
                    // Show loading spinner/message
                    view! { <div class="text-center text-gray-400 dark:text-gray-600">"Loading board..."</div> }.into_any()
                } else if let Some(err) = error.get() {
                    // Show error message
                    view! { <div class="text-center text-red-400 dark:text-red-600">{err}</div> }.into_any()
                } else if let Some(b) = board.get() {
                    // Render the board with columns and tasks
                    view! {
                        <div class="max-w-7xl mx-auto">
                            // Horizontal scrollable container for columns
                            <div class="flex gap-6 overflow-x-auto pb-4">
                                // Render each column with its associated tasks
                                {b.columns.into_iter().map(|column| {
                                    let column_id = column.id.clone();
                                    // Filter tasks that belong to this column
                                    let tasks = b.tasks.clone()
                                        .into_iter()
                                        .filter(|t| t.column_id == column_id)
                                        .collect::<Vec<_>>();
                                    view! {
                                        <ColumnView
                                            column=column
                                            tasks=tasks
                                            set_board
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    // Show message when no board is found
                    view! { <div class="text-center text-gray-400 dark:text-gray-600">"No board found"</div> }.into_any()
                }
            }}
        </main>
    }
}
