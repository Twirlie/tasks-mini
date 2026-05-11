pub fn get_theme() -> String {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();

    match storage.get_item("tasks-mini-theme") {
        Ok(Some(theme)) => theme,
        _ => "dark".to_string(), // Default to dark mode
    }
}

pub fn set_theme(theme: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let document_element = document.document_element().unwrap();

    let storage = window.local_storage().unwrap().unwrap();
    storage.set_item("tasks-mini-theme", theme).unwrap();

    if theme == "dark" {
        document_element.class_list().add_1("dark").unwrap();
    } else {
        document_element.class_list().remove_1("dark").unwrap();
    }
}

pub fn apply_theme_on_load() {
    let theme = get_theme();
    set_theme(&theme);
}
