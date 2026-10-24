use std::path::Path;
use std::sync::Arc;

use budgeterrr_core::{Account, AccountId, AccountName, AccountNameError, Bank};
use budgeterrr_storage::{SqliteStorage, Storage, StorageError};

/// File name of the local database inside the app-data folder.
const DATABASE_FILE: &str = "budgeterrr.sqlite3";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("could not create the app-data folder: {0}")]
    DataDir(#[from] std::io::Error),
    #[error(transparent)]
    InvalidAccountName(#[from] AccountNameError),
    #[error("no such Account")]
    AccountNotFound(AccountId),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

pub type AppResult<T> = Result<T, AppError>;

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
}
