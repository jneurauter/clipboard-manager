use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::Cursor,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
enum ClipboardItem {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "image")]
    Image { data_url: String },
}

#[derive(Clone)]
struct AppState(Arc<Mutex<Vec<ClipboardItem>>>);

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
    let png_bytes = STANDARD
        .decode(b64)
        .map_err(|e| e.to_string())?;
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
    if history.len() > 10 {
        history.pop();
    }
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Vec<ClipboardItem> {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn clear_history(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.0.lock().unwrap().clear();
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
        let mut last_text: Option<String> = None;
        let mut last_image: Option<String> = None;

        loop {
            let mut updated = false;

            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() && last_text.as_ref() != Some(&text) {
                    last_text = Some(text.clone());
                    let mut history = state.0.lock().unwrap();
                    push_history(&mut history, ClipboardItem::Text { content: text });
                    updated = true;
                }
            }

            if let Ok(image) = clipboard.get_image() {
                if let Ok(data_url) = image_to_data_url(image) {
                    if last_image.as_ref() != Some(&data_url) {
                        last_image = Some(data_url.clone());
                        let mut history = state.0.lock().unwrap();
                        push_history(
                            &mut history,
                            ClipboardItem::Image { data_url },
                        );
                        updated = true;
                    }
                }
            }

            if updated {
                let history = state.0.lock().unwrap().clone();
                let _ = app.emit("clipboard_update", history);
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
