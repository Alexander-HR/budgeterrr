/// Identifies an Account within one database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(pub i64);

/// A bank whose statements the app can import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bank {
    Landsbankinn,
    Arion,
}

impl Bank {
    /// Every supported bank, in the order the app offers them.
    pub const ALL: [Bank; 2] = [Bank::Landsbankinn, Bank::Arion];

    pub fn name(self) -> &'static str {
        match self {
            Bank::Landsbankinn => "Landsbankinn",
            Bank::Arion => "Arion",
        }
    }
}

/// A real-world place money is held, whose statements are imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub name: String,
    pub bank: Bank,
}

/// A valid Account name: trimmed, and not blank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountName(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AccountNameError {
    #[error("an Account name can't be blank")]
    Blank,
}

impl AccountName {
    pub fn parse(raw: &str) -> Result<Self, AccountNameError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AccountNameError::Blank);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
