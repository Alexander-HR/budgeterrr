![budgeterrr banner logo](logos/budgeterrr-banner-parchment.png)

Budgeterrr is a yourrr program for knowing how your budget is going at a glance.

# Installing
For now installing is done by building the source with future plans to accommodate the package managers winget, homebrew and apt (see issue #36).
Install rust-lang, download all the files and run `cargo build --release` to build the .exe file.
Then you can run budgeterrr.exe

# Transactions

Create an Account to add Manual Transactions. Each Transaction has a calendar
date, a signed amount in whole krónur, and text; Transactions can be edited or
deleted and are listed newest first, one page at a time. Amounts are displayed
with periods as thousands separators, for example `1.234.567 kr.`.

# For developers
To open the docs, run `cargo doc --open`