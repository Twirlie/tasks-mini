use crate::components::ColumnView;
use crate::domain::Board;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
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
