use crate::components::ColumnView;
use crate::domain::Board;
use crate::utils::theme;
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
    let (current_theme, set_current_theme) = signal(theme::get_theme());

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

    // Theme toggle function
    let toggle_theme = move |_: leptos::ev::MouseEvent| {
        let new_theme = if current_theme.get() == "dark" {
            "light"
        } else {
            "dark"
        };
        theme::set_theme(new_theme);
        set_current_theme.set(new_theme.to_string());
    };

    // Apply theme on first load
    Effect::new(move |_| {
        theme::apply_theme_on_load();
        set_current_theme.set(theme::get_theme());
    });

    // Load board on mount
    Effect::new(move |_| {
        load_board();
    });

    view! {
        <main class="min-h-screen bg-gray-900 dark:bg-gray-100 p-8 transition-colors duration-300 ease-in-out">
            <div class="max-w-7xl mx-auto mb-8">
                <div class="flex justify-between items-center">
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
                    <button
                        on:click=toggle_theme
                        class="px-4 py-2 rounded-lg bg-gray-800 dark:bg-gray-200 text-gray-100 dark:text-gray-800 hover:bg-gray-700 dark:hover:bg-gray-300 transition-colors duration-300 ease-in-out"
                    >
                        {move || {
                            if current_theme.get() == "dark" {
                                "🌙 Dark"
                            } else {
                                "☀️ Light"
                            }
                        }}
                    </button>
                </div>
            </div>
            {move || {
                if loading.get() {
                    view! { <div class="text-center text-gray-400 dark:text-gray-600">"Loading board..."</div> }.into_any()
                } else if let Some(err) = error.get() {
                    view! { <div class="text-center text-red-400 dark:text-red-600">{err}</div> }.into_any()
                } else if let Some(b) = board.get() {
                    view! {
                        <div class="max-w-7xl mx-auto">
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
                    view! { <div class="text-center text-gray-400 dark:text-gray-600">"No board found"</div> }.into_any()
                }
            }}
        </main>
    }
}
