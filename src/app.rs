use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
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

    let delete_task = {
        let set_board = set_board.clone();
        move |task_id: String| {
            spawn_local(async move {
                let args =
                    serde_wasm_bindgen::to_value(&serde_json::json!({ "id": task_id })).unwrap();
                let _ = invoke("delete_task", args).await;
                set_board.update(|board| {
                    if let Some(b) = board {
                        b.tasks.retain(|t| t.id != task_id);
                    }
                });
            });
        }
    };

    view! {
        <div class="flex-shrink-0 w-80 bg-gray-200 rounded-lg p-4">
            <h2 class="text-xl font-semibold text-gray-700 mb-4">{column.name}</h2>

            <div class="space-y-3 mb-4">
                {tasks.into_iter().map(|task| {
                    let task_id = task.id.clone();
                    let delete_task = delete_task.clone();
                    view! {
                        <TaskCard
                            task=task
                            on_delete=move || delete_task(task_id.clone())
                        />
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
fn TaskCard(task: Task, on_delete: impl Fn() + 'static) -> impl IntoView {
    view! {
        <div class="bg-white p-3 rounded shadow hover:shadow-md transition-shadow">
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
                    on:click=move |_| on_delete()
                >
                    "Delete"
                </button>
            </div>
        </div>
    }
}
