# Single-entry bookkeeping with explicit Transfers

Despite running personal finances "like a company", we record each Transaction once against one Account (single-entry) rather than using a double-entry ledger. Bank statements are already single-entry, the reports we need (income and expenses by Category per Quarter, Net Income) don't need a balance sheet, and double-entry would add concepts and UI cost for little gain. The one thing double-entry gets right that naive single-entry gets wrong, money moving between your own Accounts, is handled by an explicit Transfer that pairs two Transactions and excludes them from income and expense.

## Consequences

A balance-sheet / net-worth view, if wanted later, is built from Account balances rather than from ledger postings.
