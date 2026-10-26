//! The storage trait the app service talks to, and its SQLite implementation.
//!
//! The trait is shaped around what a screen needs (ADR 0002): each method answers
//! one screen-level question, and filtering or aggregation happens in the backend.

mod sqlite;

use budgeterrr_core::{
    Account, AccountId, AccountName, Bank, Transaction, TransactionDate, TransactionId,
    TransactionText,
};

pub use sqlite::SqliteStorage;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("database holds an unknown bank: {0}")]
    UnknownBank(String),
    #[error("database holds an invalid Transaction date: {0}")]
    InvalidTransactionDate(String),
    #[error("database holds invalid Transaction text: {0}")]
    InvalidTransactionText(String),
    #[error("database Transaction is missing its uncategorized Split")]
    InvalidTransactionSplit,
    #[error("Transaction pagination exceeds the supported range")]
    InvalidPagination,
}

pub type StorageResult<T> = Result<T, StorageError>;

pub trait Storage: Send + Sync {
    /// The Accounts list screen: every Account, oldest first.
    fn list_accounts(&self) -> StorageResult<Vec<Account>>;

    fn create_account(&self, name: &AccountName, bank: Bank) -> StorageResult<Account>;

    /// Renames the Account, returning it as it now stands, or `None` if it doesn't exist.
    fn rename_account(&self, id: AccountId, name: &AccountName) -> StorageResult<Option<Account>>;

    /// Returns one page of an Account's Transactions, newest date and id first.
    fn list_transactions(
        &self,
        account_id: AccountId,
        limit: u32,
        offset: u64,
    ) -> StorageResult<Vec<Transaction>>;

    /// Creates a Manual Transaction with one full-amount uncategorized Split.
    /// Returns `None` when the Account does not exist.
    fn create_manual_transaction(
        &self,
        account_id: AccountId,
        date: TransactionDate,
        amount: i64,
        text: &TransactionText,
    ) -> StorageResult<Option<Transaction>>;

    /// Updates a Manual Transaction belonging to the Account.
    fn update_manual_transaction(
        &self,
        account_id: AccountId,
        transaction_id: TransactionId,
        date: TransactionDate,
        amount: i64,
        text: &TransactionText,
    ) -> StorageResult<Option<Transaction>>;

    /// Deletes a Manual Transaction belonging to the Account.
    fn delete_manual_transaction(
        &self,
        account_id: AccountId,
        transaction_id: TransactionId,
    ) -> StorageResult<bool>;
}
