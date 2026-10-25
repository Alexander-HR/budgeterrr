use crate::AccountId;
use time::{Date, macros::format_description};

const ISO_DATE_FORMAT: &[time::format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day]");

/// Identifies a Transaction within one database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(pub i64);

/// Identifies a Category within one database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryId(pub i64);

/// A calendar date with no time or time zone.
pub type TransactionDate = Date;

#[derive(Debug, thiserror::Error)]
pub enum TransactionDateError {
    #[error("date must be a valid calendar date in YYYY-MM-DD format: {0}")]
    Parse(#[from] time::error::Parse),
    #[error("year must be greater than 0000")]
    YearZero,
}

pub fn parse_transaction_date(raw: &str) -> Result<TransactionDate, TransactionDateError> {
    let date = Date::parse(raw, ISO_DATE_FORMAT)?;
    if date.year() == 0 {
        return Err(TransactionDateError::YearZero);
    }
    Ok(date)
}

pub fn format_transaction_date(date: TransactionDate) -> String {
    date.format(ISO_DATE_FORMAT)
        .expect("a valid calendar date can be formatted as ISO")
}

/// A valid Transaction text: trimmed, and not blank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionText(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Transaction text can't be blank")]
pub struct TransactionTextError;

impl TransactionText {
    pub fn parse(raw: &str) -> Result<Self, TransactionTextError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(TransactionTextError);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The portion of a Transaction assigned to one Category, or left uncategorized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub amount: i64,
    pub category_id: Option<CategoryId>,
}

/// A single movement of money in or out of one Account.
///
/// Manual Transactions have no bank-original fields; their first Split is
/// uncategorized and covers the full amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: TransactionId,
    pub account_id: AccountId,
    pub date: TransactionDate,
    pub amount: i64,
    pub text: String,
    pub split: Split,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_parse_and_display_as_iso_calendar_dates() {
        let date = parse_transaction_date("2024-02-29").unwrap();

        assert_eq!(format_transaction_date(date), "2024-02-29");
    }

    #[test]
    fn dates_reject_invalid_calendar_values_and_formats() {
        for raw in [
            "2023-02-29",
            "2024-04-31",
            "2024-13-01",
            "0000-01-01",
            "2024-1-01",
        ] {
            assert!(parse_transaction_date(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn transaction_text_is_trimmed_and_cannot_be_blank() {
        assert_eq!(
            TransactionText::parse("  Coffee  ").unwrap().as_str(),
            "Coffee"
        );
        assert!(TransactionText::parse("   ").is_err());
    }
}
