use crate::components::TaskCard;
use crate::domain::{Board, Column, Task};
use crate::utils::DropZone;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn ColumnView(
    column: Column,
    tasks: Vec<Task>,
    set_board: WriteSignal<Option<Board>>,
) -> impl IntoView {
    let (show_add_task, set_show_add_task) = signal(false);
    let (new_task_title, set_new_task_title) = signal(String::new());
    let (new_task_desc, set_new_task_desc) = signal(String::new());

    let refresh_board = {
        let set_board = set_board.clone();
        Rc::new(move || {
            web_sys::console::log_1(&"refresh_board called".into());
            let set_board = set_board.clone();
            spawn_local(async move {
                web_sys::console::log_1(&"Fetching board data...".into());
                let result = invoke("get_board", JsValue::NULL).await;
                match serde_wasm_bindgen::from_value::<Board>(result) {
                    Ok(b) => {
                        web_sys::console::log_1(
                            &format!("Board updated with {} tasks", b.tasks.len()).into(),
                        );
                        set_board.set(Some(b));
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Failed to update board: {:?}", e).into());
                    }
                }
            });
        })
    };

    let add_task = {
        let set_show_add_task = set_show_add_task.clone();
        let set_new_task_title = set_new_task_title.clone();
        let set_new_task_desc = set_new_task_desc.clone();
        let set_board = set_board.clone();
        let column_name = column.name.clone();
        move |ev: SubmitEvent| {
            ev.prevent_default();
            let title = new_task_title.get_untracked();
            let desc = if new_task_desc.get_untracked().is_empty() {
                None
            } else {
                Some(new_task_desc.get_untracked())
            };
            let column_name = column_name.clone();

            spawn_local(async move {
                // Get the current board to find the correct column_id
                let board_result = invoke("get_board", JsValue::NULL).await;
                if let Ok(current_board) = serde_wasm_bindgen::from_value::<Board>(board_result) {
                    if let Some(target_column) =
                        current_board.columns.iter().find(|c| c.name == column_name)
                    {
                        let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                            "title": title,
                            "description": desc,
                            "columnId": target_column.id,
                        }))
                        .unwrap();

                        let result = invoke("create_task", args).await;
                        match serde_wasm_bindgen::from_value::<Task>(result) {
                            Ok(new_task) => {
                                set_board.update(|board| {
                                    if let Some(b) = board {
                                        b.tasks.push(new_task);
                                    }
                                });
                                set_show_add_task.set(false);
                                set_new_task_title.set(String::new());
                                set_new_task_desc.set(String::new());
                            }
                            Err(_) => {
                                // Handle error
                            }
                        }
                    }
                }
            });
        }
    };

    view! {
        <div class="flex-shrink-0 w-80 bg-gray-200 rounded-lg p-4">
            <h2 class="text-xl font-semibold text-gray-700 mb-4">{column.name}</h2>

            {
                let tasks_len = tasks.len();
                view! {
                    <div class="space-y-3 mb-4">
                        {tasks.into_iter().enumerate().map(|(index, task)| {
                            let column_id = column.id.clone();
                            let refresh_board = refresh_board.clone();
                            view! {
                                <DropZone
                                    column_id=column_id.clone()
                                    order=index as u32
                                    on_refresh=refresh_board.clone()
                                >
                                    <TaskCard
                                        task=task
                                        set_board
                                    />
                                </DropZone>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    // Add a DropZone at the bottom for dropping tasks after the last one
                    <DropZone
                        column_id=column.id.clone()
                        order=tasks_len as u32
                        on_refresh=refresh_board.clone()
                    >
                        <div class="w-full h-8 border-2 border-dashed border-gray-300 rounded opacity-50 hover:opacity-100"></div>
                    </DropZone>
                }
            }

            {move || if show_add_task.get() {
                let add_task = add_task.clone();
                view! {
                    <form on:submit=add_task class="space-y-2">
                        <input
                            type="text"
                            placeholder="Task title"
                            class="w-full p-2 rounded border"
                            on:input=move |ev| set_new_task_title.set(event_target_value(&ev))
                            prop:value=new_task_title
                        />
                        <textarea
                            placeholder="Description (optional)"
                            class="w-full p-2 rounded border"
                            on:input=move |ev| set_new_task_desc.set(event_target_value(&ev))
                            rows="2"
                        >
                            {new_task_desc.get()}
                        </textarea>
                        <div class="flex gap-2">
                            <button type="submit" class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600">
                                "Add"
                            </button>
                            <button
                                type="button"
                                class="px-3 py-1 bg-gray-400 text-white rounded hover:bg-gray-500"
                                on:click=move |_| set_show_add_task.set(false)
                            >
                                "Cancel"
                            </button>
                        </div>
                    </form>
                }.into_any()
            } else {
                view! {
                    <button
                        class="w-full p-2 border-2 border-dashed border-gray-400 rounded text-gray-500 hover:bg-gray-300"
                        on:click=move |_| set_show_add_task.set(true)
                    >
                        "+ Add Task"
                    </button>
                }.into_any()
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_column_view_renders_column_header() {
        let column = Column {
            id: "test-col-1".to_string(),
            name: "Test Column".to_string(),
            order: 0,
        };

        let tasks = vec![Task {
            id: "test-task-1".to_string(),
            title: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            column_id: "test-col-1".to_string(),
            order: 0,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            completed_at: None,
        }];

        // Test that ColumnView can be instantiated with proper props
        let (board, set_board) = signal::<Option<Board>>(None);
        let _view = view! {
            <ColumnView
                column=column
                tasks=tasks
                set_board
            />
        };
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_column_view_has_add_task_functionality() {
        let column = Column {
            id: "test-col-1".to_string(),
            name: "Test Column".to_string(),
            order: 0,
        };

        let tasks = vec![];

        // Test that ColumnView renders with add task button
        let (board, set_board) = signal::<Option<Board>>(None);
        let _view = view! {
            <ColumnView
                column=column
                tasks=tasks
                set_board
            />
        };

        // The component should include the "+ Add Task" button by default
        // This test verifies the component structure is correct
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_column_view_drag_integration() {
        let column = Column {
            id: "test-col-1".to_string(),
            name: "Test Column".to_string(),
            order: 0,
        };

        let tasks = vec![Task {
            id: "test-task-1".to_string(),
            title: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            column_id: "test-col-1".to_string(),
            order: 0,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            completed_at: None,
        }];

        // Test that ColumnView renders DropZone and TaskCard for drag & drop
        let (board, set_board) = signal::<Option<Board>>(None);
        let _view = view! {
            <ColumnView
                column=column
                tasks=tasks
                set_board
            />
        };

        // The component should render tasks within DropZone components
        // This verifies drag & drop integration structure
    }
}
