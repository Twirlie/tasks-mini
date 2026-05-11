use leptos::prelude::*;
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
    column_id: String,
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
    on_refresh: impl FnOnce(),
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

    // If dropping on same column with same order, no-op (cycle 4)
    if source_column_id == target_column_id && original_order == new_order {
        return Ok(());
    }

    let args = serde_wasm_bindgen::to_value(&MoveTaskArgs {
        id: task_id,
        column_id: target_column_id,
        order: new_order,
    })
    .map_err(|e| format!("Failed to serialize args: {:?}", e))?;

    invoke("move_task", args).await;
    on_refresh(); // Cycle 5: refresh board state after successful drop
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
                leptos::task::spawn_local(async move {
                    let _ = on_drop(col_id, order, ev, move || refresh()).await;
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
