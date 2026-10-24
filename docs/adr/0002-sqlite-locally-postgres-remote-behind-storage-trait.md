# SQLite locally, Postgres on the homelab later, behind one storage trait

Each install keeps its own SQLite database by default. To use the app from several devices, we will later let the app connect directly to a Postgres database on a self-hosted homelab server over WireGuard instead of SQLite ("remote mode"), rather than building local-first sync. Sync means conflict resolution and replication machinery that a single user doesn't need; the price is that remote mode doesn't work when the homelab is unreachable. Domain code talks only to a storage trait, with one SQLite and one Postgres implementation, and SQL stays portable between the two so switching backends is configuration, not a rewrite.

Network use in remote mode must stay minimal. The storage trait is therefore shaped around what a screen needs ("Quarter totals by Category", "a page of Transactions"), never "load all Transactions": the database does aggregation and filtering, and only results cross the wire. The app has no built-in "copy local database to remote" feature; moving existing data to the homelab is a one-off manual migration.

## Considered Options

- **Local-first sync** (SQLite on every device, replicated via the homelab): works offline, but needs conflict resolution. Rejected for now.
- **Hosted only**: breaks the v1 constraint that everything runs locally.
- **Fat-client queries** (load rows, aggregate in Rust): simpler trait, but pulls whole tables over the network in remote mode. Rejected.

## Consequences

Review maths that lives in `core` must also be expressible as SQL aggregation, or the trait must expose it as a query the backend answers. Remote-mode credentials live in the OS keychain and connections use TLS.
