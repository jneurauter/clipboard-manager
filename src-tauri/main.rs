use tauri::State;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use arboard::Clipboard;

#[derive(Serialize, Deserialize, Clone)]
struct ClipboardItem {
    content: String,
}

struct AppState(Mutex<Vec<ClipboardItem>>);

#[tauri::command]
fn get_history(state: State<AppState>) -> Vec<ClipboardItem> {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(vec![])))
        .invoke_handler(tauri::generate_handler![get_history, copy_to_clipboard])
        .run(tauri::generate_context!())
        .expect("error while running Tauri app");
}
