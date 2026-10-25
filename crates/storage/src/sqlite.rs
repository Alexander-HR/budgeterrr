use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use budgeterrr_core::{
    Account, AccountId, AccountName, Bank, CategoryId, Split, Transaction, TransactionDate,
    TransactionId, TransactionText, format_transaction_date, parse_transaction_date,
};
use rusqlite::{Connection, OptionalExtension, params};

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
    // 2: Manual Transactions and their uncategorized Splits
    "CREATE TABLE transactions (
        id INTEGER PRIMARY KEY,
        account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
        date TEXT NOT NULL,
        amount INTEGER NOT NULL CHECK (typeof(amount) = 'integer'),
        text TEXT NOT NULL CHECK (length(trim(text)) > 0)
    );
    CREATE INDEX transactions_by_account_date
        ON transactions (account_id, date DESC, id DESC);
    CREATE TABLE splits (
        id INTEGER PRIMARY KEY,
        transaction_id INTEGER NOT NULL UNIQUE REFERENCES transactions(id) ON DELETE CASCADE,
        amount INTEGER NOT NULL CHECK (typeof(amount) = 'integer'),
        category_id INTEGER
    )",
];

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    /// Opens the database file at `path`, creating it if needed, and applies pending migrations.
    pub fn open(path: &Path) -> StorageResult<Self> {
        let mut conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
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
        tx.execute_batch(sql)?;
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

    fn list_transactions(
        &self,
        account_id: AccountId,
        limit: u32,
        offset: u64,
    ) -> StorageResult<Vec<Transaction>> {
        let offset = i64::try_from(offset).map_err(|_| StorageError::InvalidPagination)?;
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.account_id, t.date, t.amount, t.text, s.amount, s.category_id
             FROM transactions t
             JOIN splits s ON s.transaction_id = t.id
             WHERE t.account_id = $1
             ORDER BY t.date DESC, t.id DESC
             LIMIT $2 OFFSET $3",
        )?;
        let rows = stmt.query_map(params![account_id.0, i64::from(limit), offset], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<i64>>(6)?,
            ))
        })?;

        rows.map(|row| {
            let (id, account_id, date, amount, text, split_amount, category_id) = row?;
            let date = parse_transaction_date(&date)
                .map_err(|_| StorageError::InvalidTransactionDate(date))?;
            let text = TransactionText::parse(&text)
                .map_err(|_| StorageError::InvalidTransactionText(text))?;
            Ok(Transaction {
                id: TransactionId(id),
                account_id: AccountId(account_id),
                date,
                amount,
                text: text.as_str().to_owned(),
                split: Split {
                    amount: split_amount,
                    category_id: category_id.map(CategoryId),
                },
            })
        })
        .collect()
    }

    fn create_manual_transaction(
        &self,
        account_id: AccountId,
        date: TransactionDate,
        amount: i64,
        text: &TransactionText,
    ) -> StorageResult<Option<Transaction>> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let account_exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1)",
            [account_id.0],
            |row| row.get(0),
        )?;
        if !account_exists {
            return Ok(None);
        }

        let date_text = format_transaction_date(date);
        let transaction_id = tx.query_row(
            "INSERT INTO transactions (account_id, date, amount, text)
             VALUES ($1, $2, $3, $4) RETURNING id",
            (account_id.0, &date_text, amount, text.as_str()),
            |row| row.get::<_, i64>(0),
        )?;
        tx.execute(
            "INSERT INTO splits (transaction_id, amount, category_id) VALUES ($1, $2, NULL)",
            (transaction_id, amount),
        )?;
        tx.commit()?;

        Ok(Some(Transaction {
            id: TransactionId(transaction_id),
            account_id,
            date,
            amount,
            text: text.as_str().to_owned(),
            split: Split {
                amount,
                category_id: None,
            },
        }))
    }

    fn update_manual_transaction(
        &self,
        account_id: AccountId,
        transaction_id: TransactionId,
        date: TransactionDate,
        amount: i64,
        text: &TransactionText,
    ) -> StorageResult<Option<Transaction>> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let date_text = format_transaction_date(date);
        let updated = tx
            .query_row(
                "UPDATE transactions SET date = $1, amount = $2, text = $3
                 WHERE id = $4 AND account_id = $5 RETURNING id",
                (
                    &date_text,
                    amount,
                    text.as_str(),
                    transaction_id.0,
                    account_id.0,
                ),
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        let Some(id) = updated else {
            return Ok(None);
        };

        let updated_splits = tx.execute(
            "UPDATE splits SET amount = $1
             WHERE transaction_id = $2 AND category_id IS NULL",
            (amount, id),
        )?;
        if updated_splits != 1 {
            return Err(StorageError::InvalidTransactionSplit);
        }
        tx.commit()?;

        Ok(Some(Transaction {
            id: transaction_id,
            account_id,
            date,
            amount,
            text: text.as_str().to_owned(),
            split: Split {
                amount,
                category_id: None,
            },
        }))
    }

    fn delete_manual_transaction(
        &self,
        account_id: AccountId,
        transaction_id: TransactionId,
    ) -> StorageResult<bool> {
        let affected = self.conn().execute(
            "DELETE FROM transactions WHERE id = $1 AND account_id = $2",
            (transaction_id.0, account_id.0),
        )?;
        Ok(affected == 1)
    }
}
