use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use budgeterrr_core::{Account, AccountId, AccountName, Bank};
use rusqlite::{Connection, OptionalExtension};

use crate::{Storage, StorageError, StorageResult};

/// Schema migrations, applied in order. A migration is never edited once released.
///
/// DDL may be SQLite-specific; queries elsewhere in this file stay portable to Postgres.
const MIGRATIONS: &[&str] = &[
    // 1: Accounts
    "CREATE TABLE accounts (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        bank TEXT NOT NULL CHECK (bank IN ('landsbankinn', 'arion'))
    )",
];

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    /// Opens the database file at `path`, creating it if needed, and applies pending migrations.
    pub fn open(path: &Path) -> StorageResult<Self> {
        let mut conn = Connection::open(path)?;
        migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        // A panic while holding the lock leaves no partial state behind: every
        // write runs in a statement or transaction that SQLite rolls back.
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn migrate(conn: &mut Connection) -> StorageResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY)",
        [],
    )?;
    let applied: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    for (version, sql) in (1..).zip(MIGRATIONS) {
        if version <= applied {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute(sql, [])?;
        tx.execute(
            "INSERT INTO schema_migrations (version) VALUES ($1)",
            [version],
        )?;
        tx.commit()?;
    }
    Ok(())
}

fn bank_to_sql(bank: Bank) -> &'static str {
    match bank {
        Bank::Landsbankinn => "landsbankinn",
        Bank::Arion => "arion",
    }
}

fn bank_from_sql(value: String) -> StorageResult<Bank> {
    match value.as_str() {
        "landsbankinn" => Ok(Bank::Landsbankinn),
        "arion" => Ok(Bank::Arion),
        _ => Err(StorageError::UnknownBank(value)),
    }
}

impl Storage for SqliteStorage {
    fn list_accounts(&self) -> StorageResult<Vec<Account>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name, bank FROM accounts ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        rows.map(|row| {
            let (id, name, bank) = row?;
            Ok(Account {
                id: AccountId(id),
                name,
                bank: bank_from_sql(bank)?,
            })
        })
        .collect()
    }

    fn create_account(&self, name: &AccountName, bank: Bank) -> StorageResult<Account> {
        let id = self.conn().query_row(
            "INSERT INTO accounts (name, bank) VALUES ($1, $2) RETURNING id",
            (name.as_str(), bank_to_sql(bank)),
            |row| row.get(0),
        )?;
        Ok(Account {
            id: AccountId(id),
            name: name.as_str().to_owned(),
            bank,
        })
    }

    fn rename_account(&self, id: AccountId, name: &AccountName) -> StorageResult<Option<Account>> {
        let bank = self
            .conn()
            .query_row(
                "UPDATE accounts SET name = $1 WHERE id = $2 RETURNING bank",
                (name.as_str(), id.0),
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        bank.map(|bank| {
            Ok(Account {
                id,
                name: name.as_str().to_owned(),
                bank: bank_from_sql(bank)?,
            })
        })
        .transpose()
    }
}
