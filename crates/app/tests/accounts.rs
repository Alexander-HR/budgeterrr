mod support;

use budgeterrr_app::AppError;
use budgeterrr_core::{AccountId, Bank};
use support::{fresh_app, open_app};
use tempfile::TempDir;

#[test]
fn first_launch_starts_with_no_accounts() {
    let (_dir, app) = fresh_app();

    assert_eq!(app.list_accounts().unwrap(), vec![]);
}

#[test]
fn created_accounts_are_listed_with_their_name_and_bank() {
    let (_dir, app) = fresh_app();

    let checking = app.create_account("Checking", Bank::Landsbankinn).unwrap();
    let card = app.create_account("Credit card", Bank::Arion).unwrap();

    assert_eq!(checking.name, "Checking");
    assert_eq!(checking.bank, Bank::Landsbankinn);
    assert_ne!(checking.id, card.id);
    assert_eq!(app.list_accounts().unwrap(), vec![checking, card]);
}

#[test]
fn accounts_survive_relaunch_from_a_new_app_data_folder() {
    let root = TempDir::new().unwrap();
    let data_dir = root.path().join("not").join("yet").join("created");

    let created = open_app(&data_dir)
        .create_account("Savings", Bank::Arion)
        .unwrap();
    let relaunched = open_app(&data_dir);

    assert_eq!(relaunched.list_accounts().unwrap(), vec![created]);
}

#[test]
fn renaming_an_account_changes_only_its_name() {
    let (_dir, app) = fresh_app();
    let checking = app.create_account("Checking", Bank::Landsbankinn).unwrap();
    let card = app.create_account("Credit card", Bank::Arion).unwrap();

    let renamed = app.rename_account(checking.id, "Household").unwrap();

    assert_eq!(renamed.id, checking.id);
    assert_eq!(renamed.name, "Household");
    assert_eq!(renamed.bank, Bank::Landsbankinn);
    assert_eq!(app.list_accounts().unwrap(), vec![renamed, card]);
}

#[test]
fn renaming_an_unknown_account_is_an_error() {
    let (_dir, app) = fresh_app();

    let result = app.rename_account(AccountId(42), "Household");

    assert!(matches!(
        result,
        Err(AppError::AccountNotFound(AccountId(42)))
    ));
}

#[test]
fn account_names_are_trimmed_and_must_not_be_blank() {
    let (_dir, app) = fresh_app();

    let blank = app.create_account("   ", Bank::Arion);
    let account = app
        .create_account("  Checking  ", Bank::Landsbankinn)
        .unwrap();
    let blank_rename = app.rename_account(account.id, "");

    assert!(matches!(blank, Err(AppError::InvalidAccountName(_))));
    assert!(matches!(blank_rename, Err(AppError::InvalidAccountName(_))));
    assert_eq!(account.name, "Checking");
    assert_eq!(app.list_accounts().unwrap(), vec![account]);
}
