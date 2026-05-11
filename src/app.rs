use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::DragEvent;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct MoveTaskArgs {
    id: String,
    columnId: String,
    order: u32,
}

/// Stores task id, source column id, and original order in DataTransfer on drag start
pub fn on_drag_start(task_id: String, column_id: String, order: u32, ev: DragEvent) {
    if let Some(dt) = ev.data_transfer() {
        let _ = dt.set_data(
            "text/plain",
            &format!("{}|{}|{}", task_id, column_id, order),
        );
    }
}

/// Handles drop event - calls move_task Tauri command and refresh callback
pub async fn on_drop(
    target_column_id: String,
    new_order: u32,
    ev: DragEvent,
    on_refresh: Rc<dyn Fn()>,
) -> Result<(), String> {
    let dt = ev.data_transfer().ok_or("No data transfer")?;

    let data = dt
        .get_data("text/plain")
        .map_err(|e| format!("Failed to get data: {:?}", e))?;
    let parts: Vec<&str> = data.split('|').collect();
    if parts.len() != 3 {
        return Err("Invalid data format".to_string());
    }

    let task_id = parts[0].to_string();
    let source_column_id = parts[1].to_string();
    let original_order: u32 = parts[2].parse().map_err(|_| "Invalid order".to_string())?;

    // If dropping on same column with same order, no-op
    if source_column_id == target_column_id && original_order == new_order {
        return Ok(());
    }

    let args = serde_wasm_bindgen::to_value(&MoveTaskArgs {
        id: task_id,
        columnId: target_column_id,
        order: new_order,
    })
    .map_err(|e| format!("Failed to serialize args: {:?}", e))?;

    invoke("move_task", args).await;
    on_refresh();
    Ok(())
}

/// Simplified version that works with extracted data instead of DragEvent
async fn move_task_with_data(
    target_column_id: String,
    new_order: u32,
    data: &str,
    on_refresh: Rc<dyn Fn()>,
) -> Result<(), String> {
    let parts: Vec<&str> = data.split('|').collect();
    if parts.len() != 3 {
        return Err("Invalid data format".to_string());
    }

    let task_id = parts[0].to_string();
    let source_column_id = parts[1].to_string();
    let original_order: u32 = parts[2].parse().map_err(|_| "Invalid order".to_string())?;

    // If dropping on same column with same order, no-op
    if source_column_id == target_column_id && original_order == new_order {
        return Ok(());
    }

    let args = serde_wasm_bindgen::to_value(&MoveTaskArgs {
        id: task_id,
        columnId: target_column_id,
        order: new_order,
    })
    .map_err(|e| format!("Failed to serialize args: {:?}", e))?;

    invoke("move_task", args).await;
    on_refresh();
    Ok(())
}

#[component]
pub fn DraggableTask(
    task_id: String,
    column_id: String,
    order: u32,
    children: Children,
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = signal(false);

    view! {
        <div
            class={move || if is_dragging.get() { "dragging" } else { "" }}
            draggable="true"
            on:dragstart=move |ev: DragEvent| {
                set_is_dragging.set(true);
                on_drag_start(task_id.clone(), column_id.clone(), order, ev)
            }
            on:dragend=move |_| set_is_dragging.set(false)
        >
            {children()}
        </div>
    }
}

#[component]
pub fn DropZone(
    column_id: String,
    order: u32,
    on_refresh: Rc<dyn Fn()>,
    children: Children,
) -> impl IntoView {
    let column_id_clone = column_id.clone();
    let (is_drag_over, set_is_drag_over) = signal(false);

    view! {
        <div
            class={move || if is_drag_over.get() { "drag-over" } else { "" }}
            on:drop=move |ev: DragEvent| {
                ev.prevent_default();
                set_is_drag_over.set(false);
                let col_id = column_id_clone.clone();
                let refresh = on_refresh.clone();

                // Extract data transfer data before async
                let dt = match ev.data_transfer() {
                    Some(dt) => dt,
                    None => {
                        web_sys::console::log_1(&"No data transfer".into());
                        return;
                    }
                };

                let data = match dt.get_data("text/plain") {
                    Ok(data) => data,
                    Err(e) => {
                        web_sys::console::log_1(&format!("Failed to get data: {:?}", e).into());
                        return;
                    }
                };

                // Spawn async task with the extracted data
                leptos::task::spawn_local(async move {
                    // Call a simplified version that works with the extracted data
                    match move_task_with_data(col_id, order, &data, refresh).await {
                        Ok(_) => {
                            // Success - refresh is already called in move_task_with_data
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("Drop error: {}", e).into());
                        }
                    }
                });
            }
            on:dragover=move |ev: DragEvent| {
                ev.prevent_default();
                set_is_drag_over.set(true);
            }
            on:dragleave=move |_| set_is_drag_over.set(false)
        >
            {children()}
        </div>
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Column {
    id: String,
    name: String,
    order: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Task {
    id: String,
    title: String,
    description: Option<String>,
    column_id: String,
    order: u32,
    created_at: String,
    updated_at: String,
    completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Board {
    id: String,
    name: String,
    columns: Vec<Column>,
    tasks: Vec<Task>,
    schema_version: u32,
}

#[component]
pub fn App() -> impl IntoView {
    let (board, set_board) = signal::<Option<Board>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);

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

    // Load board on mount
    Effect::new(move |_| {
        load_board();
    });

    view! {
        <main class="min-h-screen bg-gray-100 p-8">
            {move || {
                if loading.get() {
                    view! { <div class="text-center text-gray-600">"Loading board..."</div> }.into_any()
                } else if let Some(err) = error.get() {
                    view! { <div class="text-center text-red-600">{err}</div> }.into_any()
                } else if let Some(b) = board.get() {
                    view! {
                        <div class="max-w-7xl mx-auto">
                            <h1 class="text-3xl font-bold text-gray-800 mb-8">{b.name}</h1>
                            <div class="flex gap-6 overflow-x-auto pb-4">
                                {b.columns.into_iter().map(|column| {
                                    let column_id = column.id.clone();
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
                    view! { <div class="text-center text-gray-600">"No board found"</div> }.into_any()
                }
            }}
        </main>
    }
}

#[component]
fn ColumnView(
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
            let set_board = set_board.clone();
            spawn_local(async move {
                let result = invoke("get_board", JsValue::NULL).await;
                match serde_wasm_bindgen::from_value::<Board>(result) {
                    Ok(b) => set_board.set(Some(b)),
                    Err(_) => {}
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
                            />
                        </DropZone>
                    }
                }).collect::<Vec<_>>()}
            </div>

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

#[component]
fn TaskCard(task: Task) -> impl IntoView {
    let task_id_for_delete = task.id.clone();
    view! {
        <DraggableTask
            task_id=task.id.clone()
            column_id=task.column_id.clone()
            order=task.order
        >
            <div class="bg-white p-3 rounded shadow hover:shadow-md transition-shadow cursor-move">
                <h3 class="font-medium text-gray-800">{task.title}</h3>
                {move || {
                    if let Some(desc) = task.description.clone() {
                        view! { <p class="text-sm text-gray-600 mt-1">{desc}</p> }.into_any()
                    } else {
                        view! { }.into_any()
                    }
                }}
                <div class="flex justify-end gap-2 mt-2">
                    <button
                        class="text-sm text-red-500 hover:text-red-700"
                        on:click=move |_| {
                            let task_id = task_id_for_delete.clone();
                            spawn_local(async move {
                                let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": task_id })).unwrap();
                                let _ = invoke("delete_task", args).await;
                                // Refresh board after deletion
                                let result = invoke("get_board", JsValue::NULL).await;
                                let _ = serde_wasm_bindgen::from_value::<Board>(result);
                            });
                        }
                    >
                        "Delete"
                    </button>
                </div>
            </div>
        </DraggableTask>
    }
}
