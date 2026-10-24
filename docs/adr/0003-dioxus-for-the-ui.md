# Dioxus for the UI

The UI is written in Rust with Dioxus (desktop renderer) so the entire codebase is one language, which is a hard project constraint. We rejected Tauri with a TypeScript frontend (better table and chart libraries, but a second language) and native Rust GUIs like iced, egui or Slint (weaker table and chart support for a table-heavy app). We accept a smaller ecosystem for data grids and charts and may need to build those ourselves.
