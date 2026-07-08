# RustAG — Phase 1 Completion Checklist

> Verifies the Phase 1 MVP against the master development prompt
> ([`STAGESVM_PHASE1_PROMPT.md`](../STAGESVM_PHASE1_PROMPT.md)), file by file.
> Status legend: ✅ done · ⚠️ done with a noted caveat.

## How to verify locally

```bash
cargo build --workspace                 # all crates compile
cargo test  --workspace                 # all unit + integration tests pass
pnpm install && pnpm -r typecheck       # SDK + dashboard typecheck
cargo run -p rustag-cli -- --help       # CLI surface
```

---

## 1. Deliverables (priority order from the prompt)

| # | Deliverable | Status | Where |
|---|-------------|--------|-------|
| 1 | `stagesvm-core` — LiteSVM wrapper + account state machine | ✅ | [`crates/rustag-core`](../crates/rustag-core) |
| 2 | `stagesvm-rpc` — Solana-compatible JSON-RPC server | ✅ | [`crates/rustag-rpc`](../crates/rustag-rpc) |
| 3 | `stagesvm-mirror` — mainnet account fetcher (HTTP polling) | ✅ | [`crates/rustag-mirror`](../crates/rustag-mirror) |
| 4 | `stagesvm-cli` — `create/start/stop/airdrop/override/preload/logs` | ✅ | [`crates/rustag-cli`](../crates/rustag-cli) |
| 5 | `@stagesvm/sdk` — TypeScript SDK | ✅ | [`packages/sdk`](../packages/sdk) (`@rustag/sdk`) |
| 6 | Next.js dashboard (account explorer, tx feed, airdrop UI) | ✅ | [`packages/dashboard`](../packages/dashboard) |
| 7 | SQLite persistence | ✅ | [`crates/rustag-core/src/account_store.rs`](../crates/rustag-core/src/account_store.rs) + [`migrations/001_initial.sql`](../migrations/001_initial.sql) |
| 8 | `moka` in-memory cache | ✅ | [`stagenet.rs`](../crates/rustag-core/src/stagenet.rs) (`Cache<Pubkey, AccountEntry>`) |
| 9 | Program registry (Jupiter, Pyth, Raydium, …) | ✅ | [`registry.rs`](../crates/rustag-mirror/src/registry.rs) |
| 10 | Example tests (Jupiter swap, Pyth oracle, Raydium pool) | ✅ | [`examples/`](../examples) |

> Naming note: the project shipped as **RustAG** (`rustag-*` crates, `@rustag/sdk`),
> not the working title `stagesvm` from the prompt. Functionality is unchanged.

## 2. Core algorithms (prompt §5)

- **Account state machine** (`Unknown → Clean → Dirty → Pinned`) — ✅
  [`account_state.rs`](../crates/rustag-core/src/account_state.rs). `is_syncable()`
  gates background overwrite; `Dirty`/`Pinned` are frozen. Unit-tested
  (`dirty_clean_transitions`, `sync_label_roundtrip`).
- **The core struct** (`Stagenet`) — ✅ [`stagenet.rs`](../crates/rustag-core/src/stagenet.rs):
  lazy `pre_load_accounts_for_tx`, `send_transaction`/`simulate_transaction`,
  unlimited `airdrop`, `override_account`/`override_token_balance`, impersonation,
  **v0 address-lookup-table resolution** for real DeFi transactions.
- **The mainnet mirror** — ✅ [`fetcher.rs`](../crates/rustag-mirror/src/fetcher.rs):
  raw `getMultipleAccounts` over `reqwest` (deliberately avoids `solana-rpc-client`
  to keep the Agave crate versions litesvm unifies on), batched to 100,
  `governor` rate limiting.
- **JSON-RPC server** — ✅ [`jsonrpc.rs`](../crates/rustag-rpc/src/jsonrpc.rs): 20+
  methods incl. `getAccountInfo`, `getMultipleAccounts`, `getProgramAccounts`
  (with `dataSize`/`memcmp` filters), `sendTransaction`, `simulateTransaction`,
  `getSignatureStatuses`. `accountSubscribe`/`signatureSubscribe` over WS in
  [`ws.rs`](../crates/rustag-rpc/src/ws.rs).
- **Database schema** — ✅ [`001_initial.sql`](../migrations/001_initial.sql):
  `stagenets`/`accounts`/`transactions`, ISO-8601 timestamps and 0/1 booleans for
  Postgres portability.

## 3. CLI (prompt §6)

✅ `create`, `start`, `stop`, `status`, `list`, `airdrop`, `override`, `preload`,
`logs` — [`crates/rustag-cli/src/commands`](../crates/rustag-cli/src/commands).
Client commands talk to the running stagenet's REST API; lifecycle commands use
the SQLite registry + a PID file.

## 4. SDK + dashboard (prompt §7, §3.2)

- ✅ `RustagClient` wraps the REST API: `getStagenet`, `listAccounts`,
  `getAccount`, `listTransactions`, `airdrop`, `overrideAccount`, `preload`.
- ✅ Dashboard: Overview, Accounts explorer, Transactions feed, Actions panel,
  React Query polling.

## 5. Configuration + registry (prompt §8, §9)

- ✅ [`RustAG.toml`](../RustAG.toml) per-project config; `StagenetConfig`
  serialized into the `stagenets.config_json` column so a stagenet fully
  reconstructs after restart.
- ✅ Registry: Jupiter v6, Raydium AMM v4, Orca Whirlpools, Marinade, SPL Token,
  ATA, Token-2022, Metaplex, Pyth (SOL/ETH/USDC/USDT) — all verified to
  parse (`registry_pubkeys_parse` test).

## 6. Quality standards (prompt §11)

- ✅ Typed errors (`thiserror`) per crate: `RustagError`, `MirrorError`,
  `RpcServerError`. No `unwrap()` without a safety comment (see `registry::pk`).
- ✅ `tracing` instrumentation from day one (`#[tracing::instrument]` on the hot
  paths).
- ✅ Tests: core state machine, store round-trip (incl. `u64::MAX` rent epoch),
  registry parsing, RPC airdrop/transfer/failed-tx round-trips.
- ✅ Runtime `sqlx::query` (not the compile-time macro) so a fresh checkout builds
  with no live database.

## 7. Known caveats carried into Phase 2

- ⚠️ WS pub/sub is **poll-based** (1s) — Phase 2 adds a real-time push path
  (see [`PHASE2.md`](./PHASE2.md)).
- ⚠️ `getTokenAccountBalance` assumes 0 decimals (raw amount) — fine for the demo,
  refine when reading mint decimals.
- ⚠️ One integration test is `#[ignore]` (needs a live mainnet RPC).

**Phase 1 status: complete.** Proceed to
[`STAGESVM_PHASE2_PROMPT.md`](./STAGESVM_PHASE2_PROMPT.md).
