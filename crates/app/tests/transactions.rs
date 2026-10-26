mod support;

use budgeterrr_app::{AppError, TRANSACTIONS_PER_PAGE};
use budgeterrr_core::{AccountId, Bank, TransactionId, format_transaction_date};
use support::{fresh_app, open_app};
use tempfile::TempDir;

#[test]
fn manual_transactions_have_one_uncategorized_full_amount_split() {
    let (_dir, app) = fresh_app();
    let account = app.create_account("Cash", Bank::Arion).unwrap();

    let transaction = app
        .create_manual_transaction(account.id, "2024-02-29", -1_234_567, " Coffee ")
        .unwrap();

    assert_eq!(transaction.account_id, account.id);
    assert_eq!(format_transaction_date(transaction.date), "2024-02-29");
    assert_eq!(transaction.amount, -1_234_567);
    assert_eq!(transaction.text, "Coffee");
    assert_eq!(transaction.split.amount, transaction.amount);
    assert_eq!(transaction.split.category_id, None);
    assert_eq!(
        app.list_transactions(account.id, 0).unwrap().transactions,
        vec![transaction]
    );
}

#[test]
fn manual_transactions_can_be_edited_and_deleted() {
    let (_dir, app) = fresh_app();
    let account = app.create_account("Cash", Bank::Arion).unwrap();
    let created = app
        .create_manual_transaction(account.id, "2024-01-01", 500, "Lunch")
        .unwrap();

    let updated = app
        .update_manual_transaction(account.id, created.id, "2024-01-02", -650, "Dinner")
        .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(format_transaction_date(updated.date), "2024-01-02");
    assert_eq!(updated.amount, -650);
    assert_eq!(updated.text, "Dinner");
    assert_eq!(updated.split.amount, -650);
    assert_eq!(updated.split.category_id, None);

    app.delete_manual_transaction(account.id, created.id)
        .unwrap();
    assert!(
        app.list_transactions(account.id, 0)
            .unwrap()
            .transactions
            .is_empty()
    );
}

#[test]
fn transaction_inputs_are_validated_and_unknown_accounts_are_rejected() {
    let (_dir, app) = fresh_app();
    let account = app.create_account("Cash", Bank::Arion).unwrap();

    assert!(matches!(
        app.create_manual_transaction(account.id, "2023-02-29", 10, "Purchase"),
        Err(AppError::InvalidTransactionDate(_))
    ));
    assert!(matches!(
        app.create_manual_transaction(account.id, "2024-01-01", 10, "  "),
        Err(AppError::InvalidTransactionText(_))
    ));
    assert!(matches!(
        app.create_manual_transaction(AccountId(99), "2024-01-01", 10, "Purchase"),
        Err(AppError::AccountNotFound(AccountId(99)))
    ));
    assert!(
        app.list_transactions(account.id, 0)
            .unwrap()
            .transactions
            .is_empty()
    );
}

#[test]
fn transaction_edits_and_deletes_are_scoped_to_the_account() {
    let (_dir, app) = fresh_app();
    let account = app.create_account("Cash", Bank::Arion).unwrap();
    let other_account = app.create_account("Card", Bank::Landsbankinn).unwrap();
    let transaction = app
        .create_manual_transaction(account.id, "2024-01-01", 10, "Purchase")
        .unwrap();

    assert!(matches!(
        app.update_manual_transaction(other_account.id, transaction.id, "2024-01-02", 20, "Other"),
        Err(AppError::TransactionNotFound(id)) if id == transaction.id
    ));
    assert!(matches!(
        app.delete_manual_transaction(other_account.id, transaction.id),
        Err(AppError::TransactionNotFound(id)) if id == transaction.id
    ));
    assert_eq!(
        app.list_transactions(account.id, 0).unwrap().transactions,
        vec![transaction]
    );
}

#[test]
fn account_transactions_are_newest_first_and_paged_in_storage() {
    let (_dir, app) = fresh_app();
    let account = app.create_account("Cash", Bank::Arion).unwrap();
    let other_account = app.create_account("Card", Bank::Landsbankinn).unwrap();
    let mut created_ids = Vec::new();

    for amount in 0..(TRANSACTIONS_PER_PAGE + 2) {
        let transaction = app
            .create_manual_transaction(account.id, "2024-04-05", i64::from(amount), "Purchase")
            .unwrap();
        created_ids.push(transaction.id);
    }
    app.create_manual_transaction(other_account.id, "2025-01-01", 99, "Separate")
        .unwrap();

    let first_page = app.list_transactions(account.id, 0).unwrap();
    let second_page = app.list_transactions(account.id, 1).unwrap();
    let expected_first: Vec<_> = created_ids
        .iter()
        .rev()
        .take(TRANSACTIONS_PER_PAGE as usize)
        .copied()
        .collect();
    let expected_second: Vec<_> = created_ids
        .iter()
        .rev()
        .skip(TRANSACTIONS_PER_PAGE as usize)
        .copied()
        .collect();

    assert_eq!(
        first_page.transactions.len(),
        TRANSACTIONS_PER_PAGE as usize
    );
    assert!(first_page.has_next);
    assert_eq!(
        first_page
            .transactions
            .iter()
            .map(|transaction| transaction.id)
            .collect::<Vec<_>>(),
        expected_first
    );
    assert_eq!(second_page.page, 1);
    assert!(!second_page.has_next);
    assert_eq!(
        second_page
            .transactions
            .iter()
            .map(|transaction| transaction.id)
            .collect::<Vec<_>>(),
        expected_second
    );
}

#[test]
fn transaction_pages_sort_by_calendar_date_before_tiebreaking_by_id() {
    let (_dir, app) = fresh_app();
    let account = app.create_account("Cash", Bank::Arion).unwrap();
    let older = app
        .create_manual_transaction(account.id, "2024-01-01", 10, "Older")
        .unwrap();
    let newer = app
        .create_manual_transaction(account.id, "2024-01-03", 30, "Newer")
        .unwrap();
    let same_day_newer_id = app
        .create_manual_transaction(account.id, "2024-01-03", 20, "Same day")
        .unwrap();

    let transactions = app.list_transactions(account.id, 0).unwrap().transactions;

    assert_eq!(
        transactions
            .iter()
            .map(|transaction| transaction.id)
            .collect::<Vec<_>>(),
        vec![same_day_newer_id.id, newer.id, older.id]
    );
}

#[test]
fn manual_transactions_survive_relaunch() {
    let root = TempDir::new().unwrap();
    let data_dir = root.path().join("app-data");
    let account = open_app(&data_dir)
        .create_account("Cash", Bank::Arion)
        .unwrap();
    let created = open_app(&data_dir)
        .create_manual_transaction(account.id, "2024-12-31", i64::MIN, "Cash purchase")
        .unwrap();

    let relaunched = open_app(&data_dir);

    assert_eq!(
        relaunched
            .list_transactions(account.id, 0)
            .unwrap()
            .transactions,
        vec![created]
    );
}

#[test]
fn transaction_not_found_errors_include_the_requested_id() {
    let (_dir, app) = fresh_app();
    let account = app.create_account("Cash", Bank::Arion).unwrap();

    assert!(matches!(
        app.delete_manual_transaction(account.id, TransactionId(42)),
        Err(AppError::TransactionNotFound(TransactionId(42)))
    ));
}
