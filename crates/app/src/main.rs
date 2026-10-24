use budgeterrr_app::{AppService, ui};
use dioxus::desktop::{Config, WindowBuilder};
use directories::ProjectDirs;

fn main() {
    let Some(dirs) = ProjectDirs::from("", "", "Budgeterrr") else {
        eprintln!("Budgeterrr could not find a home directory to keep its data in.");
        std::process::exit(1);
    };
    let data_dir = dirs.data_dir();
    let app = match AppService::open_local(data_dir) {
        Ok(app) => app,
        Err(e) => {
            eprintln!(
                "Budgeterrr could not open its database in {}: {e}",
                data_dir.display()
            );
            std::process::exit(1);
        }
    };

    let config = Config::new()
        .with_window(WindowBuilder::new().with_title("Budgeterrr"))
        .with_data_directory(data_dir.join("webview"));
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(app)
        .launch(ui::App);
}
