use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};

const HISTORY_FILE: &str = "clipboard_history.json";
const MAX_HISTORY: usize = 10;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
enum ClipboardItem {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "image")]
    Image { data_url: String },
}

#[derive(Clone)]
struct AppState {
    history: Arc<Mutex<Vec<ClipboardItem>>>,
    history_path: PathBuf,
    last_text: Arc<Mutex<Option<String>>>,
    last_image: Arc<Mutex<Option<String>>>,
}

fn image_to_data_url(image: ImageData<'_>) -> Result<String, String> {
    let width = image.width as u32;
    let height = image.height as u32;
    let bytes = image.into_owned_bytes();
    let rgba = image::RgbaImage::from_raw(width, height, bytes.into_owned())
        .ok_or_else(|| "Invalid image dimensions".to_string())?;
    let mut png_bytes = Vec::new();
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let encoded = STANDARD.encode(&png_bytes);
    Ok(format!("data:image/png;base64,{encoded}"))
}

fn data_url_to_image(data_url: &str) -> Result<ImageData<'static>, String> {
    let b64 = data_url
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(|| "Invalid image data URL".to_string())?;
    let png_bytes = STANDARD.decode(b64).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&png_bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(ImageData {
        width: width as usize,
        height: height as usize,
        bytes: Cow::Owned(rgba.into_raw()),
    })
}

fn items_equal(a: &ClipboardItem, b: &ClipboardItem) -> bool {
    match (a, b) {
        (ClipboardItem::Text { content: c1 }, ClipboardItem::Text { content: c2 }) => c1 == c2,
        (ClipboardItem::Image { data_url: u1 }, ClipboardItem::Image { data_url: u2 }) => u1 == u2,
        _ => false,
    }
}

fn push_history(history: &mut Vec<ClipboardItem>, item: ClipboardItem) {
    if let Some(pos) = history.iter().position(|i| items_equal(i, &item)) {
        history.remove(pos);
    }
    history.insert(0, item);
    if history.len() > MAX_HISTORY {
        history.pop();
    }
}

fn load_history(path: &Path) -> Vec<ClipboardItem> {
    if !path.exists() {
        return vec![];
    }
    match std::fs::read_to_string(path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("Failed to parse history file: {e}");
            vec![]
        }),
        Err(e) => {
            eprintln!("Failed to read history file: {e}");
            vec![]
        }
    }
}

fn save_history(path: &Path, history: &[ClipboardItem]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())
}

fn persist_and_snapshot(state: &AppState) -> Result<Vec<ClipboardItem>, String> {
    let snapshot = state.history.lock().unwrap().clone();
    save_history(&state.history_path, &snapshot)?;
    Ok(snapshot)
}

/// Marks the current clipboard contents as "already seen" so the watcher does not
/// immediately re-add them after a clear or on startup.
fn sync_clipboard_dedupe_state(state: &AppState) {
    let Ok(mut clipboard) = Clipboard::new() else {
        return;
    };

    *state.last_text.lock().unwrap() = clipboard
        .get_text()
        .ok()
        .filter(|text| !text.is_empty());

    *state.last_image.lock().unwrap() = clipboard
        .get_image()
        .ok()
        .and_then(|image| image_to_data_url(image).ok());
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Vec<ClipboardItem> {
    state.history.lock().unwrap().clone()
}

#[tauri::command]
fn clear_history(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.history.lock().unwrap().clear();
    save_history(&state.history_path, &[])?;
    sync_clipboard_dedupe_state(&state);
    app.emit("clipboard_update", Vec::<ClipboardItem>::new())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn copy_to_clipboard(item: ClipboardItem) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    match item {
        ClipboardItem::Text { content } => clipboard.set_text(content).map_err(|e| e.to_string())?,
        ClipboardItem::Image { data_url } => {
            let image = data_url_to_image(&data_url)?;
            clipboard.set_image(image).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn start_clipboard_watcher(app: AppHandle, state: AppState) {
    std::thread::spawn(move || {
        let mut clipboard = Clipboard::new().expect("Failed to open clipboard");

        loop {
            let mut updated = false;

            // Get current clipboard state
            let current_text = clipboard.get_text().ok().filter(|t| !t.is_empty());
            let current_image = clipboard
                .get_image()
                .ok()
                .and_then(|img| image_to_data_url(img).ok());

            // Check what changed
            let mut last_text = state.last_text.lock().unwrap();
            let mut last_image = state.last_image.lock().unwrap();

            let text_changed = last_text.as_ref() != current_text.as_ref();
            let image_changed = last_image.as_ref() != current_image.as_ref();

            // Update the last known state
            *last_text = current_text.clone();
            *last_image = current_image.clone();

            drop(last_text);
            drop(last_image);

            // Add to history - if both changed, prefer the one that's not empty
            // (both being different suggests one was just set)
            if text_changed && current_text.is_some() {
                let mut history = state.history.lock().unwrap();
                push_history(&mut history, ClipboardItem::Text { content: current_text.unwrap() });
                updated = true;
            } else if image_changed && current_image.is_some() {
                let mut history = state.history.lock().unwrap();
                push_history(&mut history, ClipboardItem::Image { data_url: current_image.unwrap() });
                updated = true;
            }

            if updated {
                match persist_and_snapshot(&state) {
                    Ok(history) => {
                        let _ = app.emit("clipboard_update", history);
                    }
                    Err(e) => eprintln!("Failed to save history: {e}"),
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
            let history_path = app
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?
                .join(HISTORY_FILE);
            let history = load_history(&history_path);
            let state = AppState {
                history: Arc::new(Mutex::new(history)),
                history_path,
                last_text: Arc::new(Mutex::new(None)),
                last_image: Arc::new(Mutex::new(None)),
            };
            sync_clipboard_dedupe_state(&state);

            app.manage(state.clone());
            start_clipboard_watcher(app_handle.clone(), state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            copy_to_clipboard,
            clear_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running app");
}

fn main() {
    run();
}
