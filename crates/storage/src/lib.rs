//! The storage trait the app service talks to, and its SQLite implementation.
//!
//! The trait is shaped around what a screen needs (ADR 0002): each method answers
//! one screen-level question, and filtering or aggregation happens in the backend.

mod sqlite;

use budgeterrr_core::{Account, AccountId, AccountName, Bank};

pub use sqlite::SqliteStorage;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("database holds an unknown bank: {0}")]
    UnknownBank(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

pub trait Storage: Send + Sync {
    /// The Accounts list screen: every Account, oldest first.
    fn list_accounts(&self) -> StorageResult<Vec<Account>>;

    fn create_account(&self, name: &AccountName, bank: Bank) -> StorageResult<Account>;

    /// Renames the Account, returning it as it now stands, or `None` if it doesn't exist.
    fn rename_account(&self, id: AccountId, name: &AccountName) -> StorageResult<Option<Account>>;
}
