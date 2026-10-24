//! Dioxus screens. They render what the app service returns and forward user
//! actions to it; no domain logic lives here.

use budgeterrr_core::{Account, Bank};
use dioxus::prelude::*;

use crate::AppService;

const STYLE: &str = r#"
body { font-family: -apple-system, system-ui, sans-serif; margin: 2rem; color: #1d1d1f; }
h1 { font-size: 1.5rem; }
ul { list-style: none; padding: 0; max-width: 32rem; }
li { display: flex; gap: .5rem; align-items: center; padding: .5rem 0; border-bottom: 1px solid #ddd; }
.name { flex: 1; }
.bank { color: #666; }
.error { color: #b00020; }
.new-account { display: flex; gap: .5rem; max-width: 32rem; }
.new-account input { flex: 1; }
"#;

#[component]
pub fn App() -> Element {
    rsx! {
        style { {STYLE} }
        AccountsScreen {}
    }
}

#[component]
fn AccountsScreen() -> Element {
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
                        AccountRow { key: "{account.id.0}", account, error, changes }
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
fn AccountRow(account: Account, error: Signal<Option<String>>, changes: Signal<u32>) -> Element {
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
