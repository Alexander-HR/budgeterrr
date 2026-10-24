# Budgeterrr

A personal finance app for running a household's money like a company: bank statements are imported, transactions are categorized, and performance is reviewed each quarter.

## Money

**Account**:
A real-world place money is held, such as a bank account or credit card, whose statements are imported.
_Avoid_: Wallet, source

**Transaction**:
A single movement of money in or out of one Account, either imported from a bank statement or entered manually; the user may edit it, and an imported one keeps the bank's original values alongside.
_Avoid_: Entry, line item, posting

**Manual Transaction**:
A Transaction entered by the user rather than imported, such as a cash purchase.
_Avoid_: Cash entry, custom transaction

**Excluded Transaction**:
A Transaction the user has crossed out: still visible, but not counted in any total.
_Avoid_: Deleted, hidden, ignored

**Transfer**:
A pair of Transactions representing money moving between two of the user's own Accounts; it is neither income nor expense.
_Avoid_: Internal transaction, move

**Refund**:
A money-in Transaction linked to the earlier money-out Transaction it fully or partly reverses.
_Avoid_: Return, reversal, chargeback

**Refund Mode**:
The user's preference for how Refunds are counted: _leave out_ (subtracted from the purchase, in the purchase's Quarter) or _gross_ (counted as income in the Refund's own Quarter).
_Avoid_: Refund setting, netting

**Króna**:
The only currency; amounts are always whole krónur (ISK has no minor unit).
_Avoid_: Cents, aurar

## Categorization

**Category Group**:
A top-level grouping of related Categories, such as Housing; the review totals each group.
_Avoid_: Parent category, section

**Category**:
A user-defined label describing what a Transaction was for, such as Groceries or Salary.
_Avoid_: Tag, bucket, envelope

**Categorization Rule**:
A user-defined condition on a Transaction (such as payee text) that assigns a Category automatically.
_Avoid_: Filter, auto-tag

**Split**:
A portion of a Transaction's amount assigned to one Category; every Transaction has one or more Splits.
_Avoid_: Allocation, sub-transaction

**Suggestion**:
A Category the app proposes for a Split based on the user's past choices; it does not count until the user confirms it.
_Avoid_: Prediction, guess, auto-category

## Import

**Statement Import**:
One act of loading a bank's XLSX export for an Account; rows already imported before are skipped.
_Avoid_: Upload, sync

## Review

**Quarter**:
A calendar quarter: Q1 is January–March, through Q4 October–December.
_Avoid_: Period, term

**Quarterly Review**:
The user's periodic look at a Quarter's income and expenses by Category, and its Net Income.
_Avoid_: Report, summary

**Net Income**:
Total income minus total expenses for a Quarter, excluding Transfers and Excluded Transactions.
_Avoid_: Netto, net result, profit, balance

**Review Queue**:
The list of Transactions needing the user's attention: uncategorized, carrying an unconfirmed Suggestion, or part of a proposed Transfer or Refund that the user hasn't confirmed. Excluded Transactions never appear.
_Avoid_: Inbox, todo list
