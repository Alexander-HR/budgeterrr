//! Domain types and pure rules for Budgeterrr. No I/O lives here.

mod account;

pub use account::{Account, AccountId, AccountName, AccountNameError, Bank};
