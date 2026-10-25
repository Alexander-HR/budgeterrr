// Prevent the app from opening a terminal window on launch
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

/// Budgeterrr is a finance management application that helps users track their income, expenses
/// and budgets in a simple overview. It provides a user-friendly interface for managing
/// financial data to help users make informed financial decisions.
///
/// Author: Budgeterrr team
/// Date created: 2026-10-24

// Crate imports
// The `budgeterrr_app` crate contains the core logic and data management for the Budgeterrr application.
use budgeterrr_app::{AppService, ui};
// Dioxus is the framework used to build the UI of the application.
use dioxus::desktop::{Config, WindowBuilder, tao::window::Icon};
// The `directories` crate is used to find platform-specific directories for storing application data.
use directories::ProjectDirs;

fn main() {
    // Find a directory to store the app's data in.
    let Some(dirs) = ProjectDirs::from("", "", "Budgeterrr") else {
        eprintln!("Budgeterrr could not find a home directory to keep its data in.");
        std::process::exit(1);
    };
    // Create the data directory if it doesn't exist.
    let data_dir = dirs.data_dir();
    let app = match AppService::open_local(data_dir) {
        Ok(app) => app,
        Err(e) => {
            eprintln!(
                "Budgeterrr could not open its database in {}: {e}",
                data_dir.display()
            );
            // Close app with exit code 1 to indicate an error to the OS.
            std::process::exit(1);
        }
    };

    // Load app icon
    let icon = dioxus::desktop::icon_from_memory::<Icon>(
        include_bytes!("../../../logos/budgeterrr-icons-parchment/budgeterrr-icon-parchment.png"),
    )
    .expect("load Budgeterrr icon");

    // Launch the app with a webview.
    let config = Config::new()
        .with_window(WindowBuilder::new().with_title("Budgeterrr"))
        .with_icon(icon)
        .with_data_directory(data_dir.join("webview"));
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(app)
        .launch(ui::App);
}
