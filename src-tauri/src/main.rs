use tauri::{AppHandle, Emitter, Manager, State};
use serde::{Serialize, Deserialize};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use arboard::Clipboard;

#[derive(Serialize, Deserialize, Clone)]
struct ClipboardItem {
    content: String,
}

#[derive(Clone)]
struct AppState(Arc<Mutex<Vec<ClipboardItem>>>);

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Vec<ClipboardItem> {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text.clone()).map_err(|e| e.to_string())?;

    Ok(())
}

fn start_clipboard_watcher(app: AppHandle, state: AppState) {
    std::thread::spawn(move || {
        let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
        let mut last_text = None;

        loop {
            if let Ok(text) = clipboard.get_text() {
                let mut history = state.0.lock().unwrap();

                let should_add = match &last_text {
                    Some(prev) => prev != &text,
                    None => true,
                };

                if should_add {
                    last_text = Some(text.clone());

                    if let Some(pos) = history.iter().position(|item| item.content == text) {
                        history.remove(pos);
                    }

                    history.insert(0, ClipboardItem { content: text.clone() });
                    if history.len() > 10 {
                        history.pop();
                    }
                    if let Err(err) = app.emit("clipboard_update", history.clone()) {
                        eprintln!("Emit failed: {:?}", err);
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            let state = AppState(Arc::new(Mutex::new(vec![])));

            // register state and start watcher
            app.manage(state.clone());
            start_clipboard_watcher(app_handle.clone(), state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_history, copy_to_clipboard])
        .run(tauri::generate_context!())
        .expect("error while running app");
}

fn main() {
    run();
}
