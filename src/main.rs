// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fmt::format, result};

use slint::SharedString;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let ui_handle = ui.as_weak();
    ui.on_divide_income(move |string: SharedString| {
            let ui = ui_handle.unwrap();
            let num: f64 = string.trim().parse().unwrap();
            let result: String = format!("Chicken tenders {}", {num});
            ui.set_results(result.into());
    });

    ui.run()?;

    Ok(())
}
