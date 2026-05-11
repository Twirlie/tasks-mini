use leptos::prelude::*;
use crate::utils::theme;

#[component]
pub fn ThemeButton() -> impl IntoView {
    let (current_theme, set_current_theme) = signal(theme::get_theme());

    // Apply theme on first load
    Effect::new(move |_| {
        theme::apply_theme_on_load();
        set_current_theme.set(theme::get_theme());
    });

    let on_click = move |_: leptos::ev::MouseEvent| {
        let new_theme = if current_theme.get() == "dark" {
            "light"
        } else {
            "dark"
        };
        theme::set_theme(new_theme);
        set_current_theme.set(new_theme.to_string());
    };

    view! {
        <button
            on:click=on_click
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
    }
}
