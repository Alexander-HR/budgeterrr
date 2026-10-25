use std::path::Path;
use std::sync::Arc;

use budgeterrr_core::{
    Account, AccountId, AccountName, AccountNameError, Bank, Transaction, TransactionDateError,
    TransactionId, TransactionText, TransactionTextError, parse_transaction_date,
};
use budgeterrr_storage::{SqliteStorage, Storage, StorageError};

/// File name of the local database inside the app-data folder.
const DATABASE_FILE: &str = "budgeterrr.sqlite3";
pub const TRANSACTIONS_PER_PAGE: u32 = 25;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("could not create the app-data folder: {0}")]
    DataDir(#[from] std::io::Error),
    #[error(transparent)]
    InvalidAccountName(#[from] AccountNameError),
    #[error(transparent)]
    InvalidTransactionDate(#[from] TransactionDateError),
    #[error(transparent)]
    InvalidTransactionText(#[from] TransactionTextError),
    #[error("no such Account")]
    AccountNotFound(AccountId),
    #[error("no such Transaction")]
    TransactionNotFound(TransactionId),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionPage {
    pub transactions: Vec<Transaction>,
    pub page: u32,
    pub has_next: bool,
}

/// The one API the UI calls. Holds the domain flow; the UI only renders and forwards.
#[derive(Clone)]
pub struct AppService {
    storage: Arc<dyn Storage>,
}

impl AppService {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Opens the local SQLite database in `data_dir`, creating the folder and
    /// database on first launch and applying any pending migrations.
    pub fn open_local(data_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(data_dir)?;
        let storage = SqliteStorage::open(&data_dir.join(DATABASE_FILE))?;
        Ok(Self::new(Arc::new(storage)))
    }

    pub fn list_accounts(&self) -> AppResult<Vec<Account>> {
        Ok(self.storage.list_accounts()?)
    }

    pub fn create_account(&self, name: &str, bank: Bank) -> AppResult<Account> {
        let name = AccountName::parse(name)?;
        Ok(self.storage.create_account(&name, bank)?)
    }

    pub fn rename_account(&self, id: AccountId, name: &str) -> AppResult<Account> {
        let name = AccountName::parse(name)?;
        self.storage
            .rename_account(id, &name)?
            .ok_or(AppError::AccountNotFound(id))
    }

    pub fn list_transactions(
        &self,
        account_id: AccountId,
        page: u32,
    ) -> AppResult<TransactionPage> {
        let offset = u64::from(page) * u64::from(TRANSACTIONS_PER_PAGE);
        let mut transactions =
            self.storage
                .list_transactions(account_id, TRANSACTIONS_PER_PAGE + 1, offset)?;
        let has_next = transactions.len() > TRANSACTIONS_PER_PAGE as usize;
        if has_next {
            transactions.pop();
        }
        Ok(TransactionPage {
            transactions,
            page,
            has_next,
        })
    }

    pub fn create_manual_transaction(
        &self,
        account_id: AccountId,
        date: &str,
        amount: i64,
        text: &str,
    ) -> AppResult<Transaction> {
        let date = parse_transaction_date(date)?;
        let text = TransactionText::parse(text)?;
        self.storage
            .create_manual_transaction(account_id, date, amount, &text)?
            .ok_or(AppError::AccountNotFound(account_id))
    }

    pub fn update_manual_transaction(
        &self,
        account_id: AccountId,
        transaction_id: TransactionId,
        date: &str,
        amount: i64,
        text: &str,
    ) -> AppResult<Transaction> {
        let date = parse_transaction_date(date)?;
        let text = TransactionText::parse(text)?;
        self.storage
            .update_manual_transaction(account_id, transaction_id, date, amount, &text)?
            .ok_or(AppError::TransactionNotFound(transaction_id))
    }

    pub fn delete_manual_transaction(
        &self,
        account_id: AccountId,
        transaction_id: TransactionId,
    ) -> AppResult<()> {
        if self
            .storage
            .delete_manual_transaction(account_id, transaction_id)?
        {
            Ok(())
        } else {
            Err(AppError::TransactionNotFound(transaction_id))
        }
    }
}
