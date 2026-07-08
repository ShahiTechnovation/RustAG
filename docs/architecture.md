# RustAG Architecture

RustAG turns [LiteSVM](https://github.com/LiteSVM/litesvm) — an in-process Solana VM —
into a *persistent, mainnet-mirroring* staging environment. This document explains how
the pieces fit together and why.

## Crate graph

```
rustag-cli ─┬─► rustag-rpc ──► rustag-core ──► rustag-mirror
            ├──────────────────► rustag-core
            └──────────────────────────────────► (registry)
```

- **`rustag-mirror`** — a dependency-light read-side. Given pubkeys, it returns their
  current mainnet state via a raw `getMultipleAccounts` JSON-RPC call over `reqwest`.
  It deliberately avoids `solana-rpc-client`, which would fork the Solana crate versions
  that LiteSVM 0.12 unifies on (the 3.x line). It owns the **known-program registry**.
- **`rustag-core`** — the engine. Owns the `LiteSVM` instance, the SQLite store, the
  account-state machine, and the lazy-mirror logic.
- **`rustag-rpc`** — a Solana-compatible JSON-RPC + WebSocket server and a REST API for
  the dashboard, all built on `axum`.
- **`rustag-cli`** — the `rustag` binary.

## The account-state machine

Every account carries one of four sync states (`crates/rustag-core/src/account_state.rs`):

| State     | Meaning                                            | Background sync? |
| --------- | -------------------------------------------------- | ---------------- |
| `Unknown` | Never fetched; fetched lazily on first access.     | n/a              |
| `Clean`   | A faithful mainnet copy.                           | **Yes**          |
| `Dirty`   | Modified by a local transaction.                   | Never            |
| `Pinned`  | Set via the override API.                          | Never            |

This is the crux of "mainnet replay" on the SVM. EVM tools fork at a block hash; Solana
has no equivalent, so RustAG fetches accounts on demand and tracks writes so it knows
what it may and may not refresh.

## Transaction lifecycle

`Stagenet::send_transaction` (`crates/rustag-core/src/stagenet.rs`):

1. **Pre-load** — extract the transaction's static account keys; for each one that is not
   already loaded and not `Dirty`, batch-fetch it from mainnet and load it into LiteSVM
   as `Clean`. Fetch failures are logged and tolerated.
2. **Execute** — run the transaction through LiteSVM (signature + blockhash checks on).
3. **Track writes** — derive the *writable* accounts from the message header and mark
   them `Dirty`, persisting their post-state from the SVM. Read-only accounts (programs,
   oracles, sysvars) stay `Clean` and keep syncing.
4. **Index** — record the transaction (signature, success, fee, compute units, programs,
   logs) for the dashboard and `rustag logs`.

Writable accounts are computed from the message header's
`(num_required_signatures, num_readonly_signed, num_readonly_unsigned)` layout.

## Background oracle sync

`spawn_oracle_sync` (`crates/rustag-core/src/sync.rs`) wakes on an interval (30s by
default), takes a write lock on the stagenet, and calls `refresh_clean_oracles`, which
re-fetches every `Clean` account tagged `Oracle` from mainnet. `Dirty`/`Pinned` accounts
are skipped, so user-modified state is never clobbered.

## Persistence

SQLite via `sqlx` (`crates/rustag-core/src/account_store.rs`), using the runtime query
API (no compile-time `DATABASE_URL` needed). Three tables: `stagenets`, `accounts`,
`transactions` (`migrations/001_initial.sql`). On `rustag start`, the stored accounts are
rehydrated into a fresh LiteSVM instance, so a stagenet survives restarts. The schema is
Postgres-portable for Phase 2.

## RPC compatibility

`rustag-rpc` implements the subset of the Solana JSON-RPC API that a wallet or
`@solana/web3.js` `Connection` needs: `getAccountInfo`, `getBalance`,
`getMultipleAccounts`, `getProgramAccounts`, `getLatestBlockhash`, `sendTransaction`,
`simulateTransaction`, `requestAirdrop`, `getSignatureStatuses`, `getTransaction`, and
friends. A minimal poll-based `accountSubscribe` is served over WebSocket. See
[`api-reference.md`](api-reference.md).

## Address lookup tables (v0 transactions)

`send_transaction`/`simulate_transaction` call `prepare_accounts`, which — for a
`VersionedMessage::V0` — fetches the lookup-table accounts named in
`address_table_lookups`, deserializes them, and pre-loads every resolved address before
execution (and includes the writable resolved addresses in dirty tracking). This lets v0
DeFi transactions read real mainnet state through the mirror instead of failing with
`LookupTableAccountNotFound`.

## Known limitations (Phase 1)

- **Executing arbitrary mainnet programs.** The mirror loads program accounts *verbatim*
  (so they are readable and present), but it does not yet extract and JIT-load their BPF
  bytecode from the separate program-data account. That means *your own* deployed program
  can read real mainnet state, but invoking a foreign program like Jupiter end-to-end
  needs the fuller program-loading planned for Phase 2.
- **`getProgramAccounts`** supports `dataSize` and `memcmp` filters but ignores
  `dataSlice`, and reads only the stagenet's local account set (capped at 10k rows).
- **WebSocket pub/sub** is poll-based; Yellowstone gRPC push is a Phase 2 upgrade.
