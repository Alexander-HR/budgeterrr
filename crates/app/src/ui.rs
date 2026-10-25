//! Dioxus screens. They render what the app service returns and forward user
//! actions to it; no domain logic lives here.

use budgeterrr_core::{Account, AccountId, Bank, Transaction, format_transaction_date};
use dioxus::prelude::*;

use crate::AppService;

const STYLE: &str = r#"
body { font-family: -apple-system, system-ui, sans-serif; margin: 2rem; color: #1d1d1f; }
h1 { font-size: 1.5rem; }
ul { list-style: none; padding: 0; max-width: 32rem; }
li { display: flex; gap: .5rem; align-items: center; padding: .5rem 0; border-bottom: 1px solid #ddd; }
.name, .transaction-text { flex: 1; }
.bank { color: #666; }
.error { color: #b00020; }
.new-account, .new-transaction, .transaction-edit { display: flex; gap: .5rem; max-width: 48rem; }
.new-account input { flex: 1; }
.new-transaction input[type="text"] { flex: 1; }
.transaction-date, .transaction-amount { min-width: 9rem; }
"#;

#[component]
pub fn App() -> Element {
    let mut selected_account = use_signal(|| None::<Account>);

    rsx! {
        style { {STYLE} }
        if let Some(account) = selected_account() {
            TransactionsScreen {
                account,
                on_back: move |_| selected_account.set(None),
            }
        } else {
            AccountsScreen {
                on_select: move |account| selected_account.set(Some(account)),
            }
        }
    }
}

#[component]
fn AccountsScreen(on_select: EventHandler<Account>) -> Element {
    let app = use_context::<AppService>();
    // Bumped after every successful change so the list is fetched again.
    let mut changes = use_signal(|| 0u32);
    let mut error = use_signal(|| None::<String>);
    let mut new_name = use_signal(String::new);
    let mut new_bank = use_signal(|| Bank::ALL[0]);

    let accounts = use_memo({
        let app = app.clone();
        move || {
            changes();
            app.list_accounts().map_err(|e| e.to_string())
        }
    });

    let create = move |_| match app.create_account(&new_name(), new_bank()) {
        Ok(_) => {
            new_name.set(String::new());
            error.set(None);
            *changes.write() += 1;
        }
        Err(e) => error.set(Some(e.to_string())),
    };

    rsx! {
        h1 { "Accounts" }
        match accounts() {
            Ok(list) if list.is_empty() => rsx! { p { "No Accounts yet." } },
            Ok(list) => rsx! {
                ul {
                    for account in list {
                        AccountRow {
                            key: "{account.id.0}",
                            account,
                            error,
                            changes,
                            on_select,
                        }
                    }
                }
            },
            Err(e) => rsx! { p { class: "error", "{e}" } },
        }
        div { class: "new-account",
            input {
                placeholder: "Account name",
                value: "{new_name}",
                oninput: move |e| new_name.set(e.value()),
            }
            select {
                onchange: move |e| {
                    if let Some(bank) = e.value().parse::<usize>().ok().and_then(|i| Bank::ALL.get(i)) {
                        new_bank.set(*bank);
                    }
                },
                for (i, bank) in Bank::ALL.into_iter().enumerate() {
                    option { value: "{i}", selected: bank == new_bank(), "{bank.name()}" }
                }
            }
            button { onclick: create, "Add Account" }
        }
        if let Some(e) = error() {
            p { class: "error", "{e}" }
        }
    }
}

#[component]
fn AccountRow(
    account: Account,
    error: Signal<Option<String>>,
    changes: Signal<u32>,
    on_select: EventHandler<Account>,
) -> Element {
    let app = use_context::<AppService>();
    let mut error = error;
    let mut changes = changes;
    let mut draft = use_signal(|| None::<String>);
    let id = account.id;

    let Some(text) = draft() else {
        let name = account.name.clone();
        return rsx! {
            li {
                span { class: "name", "{account.name}" }
                span { class: "bank", "{account.bank.name()}" }
                button { onclick: move |_| on_select.call(account.clone()), "Transactions" }
                button { onclick: move |_| draft.set(Some(name.clone())), "Rename" }
            }
        };
    };

    let save = move |_| match app.rename_account(id, &draft().unwrap_or_default()) {
        Ok(_) => {
            draft.set(None);
            error.set(None);
            *changes.write() += 1;
        }
        Err(e) => error.set(Some(e.to_string())),
    };

    rsx! {
        li {
            input {
                class: "name",
                value: "{text}",
                oninput: move |e| draft.set(Some(e.value())),
            }
            button { onclick: save, "Save" }
            button { onclick: move |_| draft.set(None), "Cancel" }
        }
    }
}

#[component]
fn TransactionsScreen(account: Account, on_back: EventHandler<()>) -> Element {
    let app = use_context::<AppService>();
    let mut changes = use_signal(|| 0u32);
    let mut page_number = use_signal(|| 0u32);
    let mut error = use_signal(|| None::<String>);
    let mut new_date = use_signal(String::new);
    let mut new_amount = use_signal(String::new);
    let mut new_text = use_signal(String::new);

    let page = use_memo({
        let app = app.clone();
        let account_id = account.id;
        move || {
            changes();
            app.list_transactions(account_id, page_number())
                .map_err(|e| e.to_string())
        }
    });

    let create = move |_| {
        let amount = match new_amount().parse::<i64>() {
            Ok(amount) => amount,
            Err(_) => {
                error.set(Some("Amount must be a whole number of krónur.".to_owned()));
                return;
            }
        };
        match app.create_manual_transaction(account.id, &new_date(), amount, &new_text()) {
            Ok(_) => {
                new_date.set(String::new());
                new_amount.set(String::new());
                new_text.set(String::new());
                error.set(None);
                page_number.set(0);
                *changes.write() += 1;
            }
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    rsx! {
        button { onclick: move |_| on_back.call(()), "Back to Accounts" }
        h1 { "Transactions — {account.name}" }
        match page() {
            Ok(result) if result.transactions.is_empty() => rsx! {
                p { "No Transactions on this page." }
            },
            Ok(result) => rsx! {
                ul {
                    for transaction in result.transactions {
                        TransactionRow {
                            key: "{transaction.id.0}",
                            account_id: account.id,
                            transaction,
                            error,
                            changes,
                        }
                    }
                }
            },
            Err(e) => rsx! { p { class: "error", "{e}" } },
        }
        div { class: "new-transaction",
            input {
                class: "transaction-date",
                r#type: "date",
                value: "{new_date}",
                oninput: move |e| new_date.set(e.value()),
            }
            input {
                class: "transaction-amount",
                r#type: "number",
                step: "1",
                placeholder: "Amount in krónur",
                value: "{new_amount}",
                oninput: move |e| new_amount.set(e.value()),
            }
            input {
                r#type: "text",
                placeholder: "Description",
                value: "{new_text}",
                oninput: move |e| new_text.set(e.value()),
            }
            button { onclick: create, "Add Transaction" }
        }
        match page() {
            Ok(result) => rsx! {
                div {
                    button {
                        disabled: page_number() == 0,
                        onclick: move |_| page_number.set(page_number().saturating_sub(1)),
                        "Newer"
                    }
                    span { " Page {page_number() + 1} " }
                    button {
                        disabled: !result.has_next,
                        onclick: move |_| page_number.set(page_number().saturating_add(1)),
                        "Older"
                    }
                }
            },
            Err(_) => rsx! {},
        }
        if let Some(e) = error() {
            p { class: "error", "{e}" }
        }
    }
}

#[component]
fn TransactionRow(
    account_id: AccountId,
    transaction: Transaction,
    error: Signal<Option<String>>,
    changes: Signal<u32>,
) -> Element {
    let app = use_context::<AppService>();
    let mut error = error;
    let mut changes = changes;
    let mut editing = use_signal(|| false);
    let mut date = use_signal(|| format_transaction_date(transaction.date));
    let mut amount = use_signal(|| transaction.amount.to_string());
    let mut text = use_signal(|| transaction.text.clone());
    let transaction_id = transaction.id;

    if !editing() {
        let date_text = format_transaction_date(transaction.date);
        let amount_text = format_amount(transaction.amount);
        return rsx! {
            li {
                span { class: "transaction-date", "{date_text}" }
                span { class: "transaction-amount", "{amount_text}" }
                span { class: "transaction-text", "{transaction.text}" }
                button {
                    onclick: move |_| {
                        date.set(format_transaction_date(transaction.date));
                        amount.set(transaction.amount.to_string());
                        text.set(transaction.text.clone());
                        editing.set(true);
                    },
                    "Edit"
                }
                button {
                    onclick: move |_| match app.delete_manual_transaction(account_id, transaction_id) {
                        Ok(()) => {
                            error.set(None);
                            *changes.write() += 1;
                        }
                        Err(e) => error.set(Some(e.to_string())),
                    },
                    "Delete"
                }
            }
        };
    }

    let save = move |_| {
        let amount_value = match amount().parse::<i64>() {
            Ok(amount) => amount,
            Err(_) => {
                error.set(Some("Amount must be a whole number of krónur.".to_owned()));
                return;
            }
        };
        match app.update_manual_transaction(
            account_id,
            transaction_id,
            &date(),
            amount_value,
            &text(),
        ) {
            Ok(_) => {
                editing.set(false);
                error.set(None);
                *changes.write() += 1;
            }
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    rsx! {
        li { class: "transaction-edit",
            input {
                class: "transaction-date",
                r#type: "date",
                value: "{date}",
                oninput: move |e| date.set(e.value()),
            }
            input {
                class: "transaction-amount",
                r#type: "number",
                step: "1",
                value: "{amount}",
                oninput: move |e| amount.set(e.value()),
            }
            input {
                class: "transaction-text",
                r#type: "text",
                value: "{text}",
                oninput: move |e| text.set(e.value()),
            }
            button { onclick: save, "Save" }
            button { onclick: move |_| editing.set(false), "Cancel" }
        }
    }
}

fn format_amount(amount: i64) -> String {
    let digits = amount.unsigned_abs().to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    let grouped: String = grouped.chars().rev().collect();
    format!("{}{} kr.", if amount < 0 { "-" } else { "" }, grouped)
}

#[cfg(test)]
mod tests {
    use super::format_amount;

    #[test]
    fn amounts_are_grouped_with_periods_and_formatted_without_overflow() {
        assert_eq!(format_amount(1_234_567), "1.234.567 kr.");
        assert_eq!(format_amount(-1_234), "-1.234 kr.");
        assert_eq!(format_amount(i64::MIN), "-9.223.372.036.854.775.808 kr.");
    }
}
