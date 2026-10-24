//! The Budgeterrr desktop app: the app service the UI calls, and the UI itself.

mod service;
pub mod ui;

pub use service::{AppError, AppResult, AppService};
