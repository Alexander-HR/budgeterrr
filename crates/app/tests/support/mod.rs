//! Backend factory for the app service suite.
//!
//! Every test opens the app service through [`open_app`], so the same suite can
//! run against another storage backend by adding it here and selecting it with
//! `BUDGETERRR_TEST_BACKEND`.

use std::path::Path;

use budgeterrr_app::AppService;
use tempfile::TempDir;

/// Opens the app service on the selected backend, keeping its data in `data_dir`.
pub fn open_app(data_dir: &Path) -> AppService {
    match std::env::var("BUDGETERRR_TEST_BACKEND").as_deref() {
        Ok("sqlite") | Err(_) => AppService::open_local(data_dir).expect("open app service"),
        Ok(other) => panic!("unknown test backend: {other}"),
    }
}

/// Opens the app service on a fresh, empty app-data folder.
pub fn fresh_app() -> (TempDir, AppService) {
    let dir = TempDir::new().expect("create temp dir");
    let app = open_app(dir.path());
    (dir, app)
}
