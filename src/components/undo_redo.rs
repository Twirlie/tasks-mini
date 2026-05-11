use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use crate::domain::Board;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn UndoButton(set_board: WriteSignal<Option<Board>>) -> impl IntoView {
    let on_click = move |_| {
        spawn_local(async move {
            let _ = invoke("undo", JsValue::NULL).await;
            let result = invoke("get_board", JsValue::NULL).await;
            match serde_wasm_bindgen::from_value::<Board>(result) {
                Ok(board) => set_board.set(Some(board)),
                Err(e) => web_sys::console::log_1(&format!("Failed to refresh after undo: {}", e).into()),
            }
        });
    };

    view! {
        <button
            on:click=on_click
            class="px-3 py-2 rounded-lg bg-gray-800 dark:bg-gray-200 text-gray-100 dark:text-gray-800 hover:bg-gray-700 dark:hover:bg-gray-300 transition-colors duration-300 ease-in-out disabled:opacity-50 disabled:cursor-not-allowed"
            title="Undo (Ctrl+Z)"
        >
            "↶ Undo"
        </button>
    }
}

#[component]
pub fn RedoButton(set_board: WriteSignal<Option<Board>>) -> impl IntoView {
    let on_click = move |_| {
        spawn_local(async move {
            let _ = invoke("redo", JsValue::NULL).await;
            let result = invoke("get_board", JsValue::NULL).await;
            match serde_wasm_bindgen::from_value::<Board>(result) {
                Ok(board) => set_board.set(Some(board)),
                Err(e) => web_sys::console::log_1(&format!("Failed to refresh after redo: {}", e).into()),
            }
        });
    };

    view! {
        <button
            on:click=on_click
            class="px-3 py-2 rounded-lg bg-gray-800 dark:bg-gray-200 text-gray-100 dark:text-gray-800 hover:bg-gray-700 dark:hover:bg-gray-300 transition-colors duration-300 ease-in-out disabled:opacity-50 disabled:cursor-not-allowed"
            title="Redo (Ctrl+Y)"
        >
            "↷ Redo"
        </button>
    }
}

#[component]
pub fn UndoRedoControls(set_board: WriteSignal<Option<Board>>) -> impl IntoView {
    view! {
        <div class="flex gap-2">
            <UndoButton set_board />
            <RedoButton set_board />
        </div>
    }
}
