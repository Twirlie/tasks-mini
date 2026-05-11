use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use crate::domain::Board;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub fn setup_keyboard_shortcuts(set_board: WriteSignal<Option<Board>>) {
    // Keyboard shortcuts handler
    let handle_keyboard = {
        let set_board = set_board.clone();
        move |e: web_sys::KeyboardEvent| {
            let set_board = set_board.clone();
            spawn_local(async move {
                let refresh_board = || {
                    let set_board = set_board.clone();
                    spawn_local(async move {
                        let result = invoke("get_board", JsValue::NULL).await;
                        match serde_wasm_bindgen::from_value::<Board>(result) {
                            Ok(board) => {
                                set_board.set(Some(board));
                            }
                            Err(e) => {
                                web_sys::console::log_1(
                                    &format!("Failed to refresh board: {}", e).into(),
                                );
                            }
                        }
                    });
                };

                // Check for Ctrl+Z (undo)
                if e.ctrl_key() && (e.key() == "z" || e.key() == "Z") {
                    web_sys::console::log_1(&"Ctrl+Z pressed - triggering undo".into());

                    // Call undo command
                    let _ = invoke("undo", JsValue::NULL).await;

                    // Refresh board after undo
                    refresh_board();
                }
                // Check for Ctrl+Y (redo)
                else if e.ctrl_key() && (e.key() == "y" || e.key() == "Y") {
                    web_sys::console::log_1(&"Ctrl+Y pressed - triggering redo".into());

                    // Call redo command
                    let _ = invoke("redo", JsValue::NULL).await;

                    // Refresh board after redo
                    refresh_board();
                }
            });
        }
    };

    // Set up keyboard event listener
    Effect::new(move |_| {
        use wasm_bindgen::JsCast;

        let window = web_sys::window().unwrap();
        
        // Add keyboard event listener to window instead of document for better capture
        let closure = wasm_bindgen::closure::Closure::wrap(
            Box::new(handle_keyboard) as Box<dyn Fn(web_sys::KeyboardEvent)>
        );

        window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        // Prevent cleanup for now (in a real app, you'd want proper cleanup)
        closure.forget();
    });
}
