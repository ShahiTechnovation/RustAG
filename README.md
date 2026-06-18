# RustAG

**A persistent, mainnet-mirroring staging environment for Solana programs.**

RustAG is the Solana equivalent of what Tenderly Virtual TestNets are for EVM — but
built natively for the SVM account model. It wraps [LiteSVM](https://github.com/LiteSVM/litesvm)
with a **lazy mainnet account mirror**, so your tests run against *real* Pyth prices,
*real* Raydium pools, and *real* token mints — without spending a single lamport of
mainnet SOL.

```
Solana testnet faucet:        ~5 SOL/day max
DeFi integration test suite:  20–50 SOL/day needed
With RustAG:                  unlimited airdrops, real mainnet state, $0
```

---

## The core idea: lazy account mirror

When a transaction reads account `X`:

1. **Local hit?** Return the stagenet's copy.
2. **Miss?** Fetch it from mainnet → cache it → mark it `Clean` → return it.
3. **A transaction writes `X`?** Mark it `Dirty` — it is now frozen from mainnet sync forever.

A background task re-fetches `Clean` **oracle** accounts every 30s, so Pyth/Switchboard
prices stay fresh. `Dirty` and `Pinned` accounts are never overwritten. This is how
"mainnet replay" works on the SVM, where (unlike EVM) there is no block to fork from.

---

## Quick start

```bash
# 1. Build the CLI
cargo build --release            # produces target/release/rustag

# 2. Point the mirror at a mainnet RPC (a free Helius key is recommended)
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"

# 3. Create and start a stagenet
rustag create demo
rustag start demo --preload pyth raydium       # run this in its own terminal

# 4. In another terminal: airdrop, inspect, tail logs
rustag airdrop -s demo <YOUR_WALLET> 1000
rustag status  -s demo
rustag logs    -s demo --follow
```

Now point any Solana client at `http://127.0.0.1:8899`:

```bash
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor test
solana balance <YOUR_WALLET> --url http://127.0.0.1:8899
```

The web dashboard lives in [`packages/dashboard`](packages/dashboard):

```bash
pnpm install
NEXT_PUBLIC_RUSTAG_API_URL=http://localhost:9000 pnpm --filter dashboard dev
# open http://localhost:3000
```

---

## Workspace layout

| Crate / package           | What it is                                                            |
| ------------------------- | -------------------------------------------------------------------- |
| `crates/rustag-core`      | The runtime: LiteSVM + account state machine + persistence + engine. |
| `crates/rustag-mirror`    | The mainnet fetcher: JSON-RPC over `reqwest`, registry, rate limiter, real-time push (feature `realtime`). |
| `crates/rustag-rpc`       | Solana-compatible JSON-RPC + WebSocket + REST (axum).                 |
| `crates/rustag-cli`       | The `rustag` binary.                                                  |
| `crates/rustag-scheduler` | **Phase 2** — Activity Scheduler (cron / interval on-chain actions). |
| `crates/rustag-sim`       | **Phase 2** — simulation framework (fork, replay, stress, compare).  |
| `crates/rustag-cloud`     | **Phase 2** — multi-tenant cloud control plane (`rustag-cloud`).     |
| `packages/sdk`            | `@rustag/sdk` — TypeScript client for the REST API.                  |
| `packages/dashboard`      | Next.js 15 dashboard: accounts, transactions, analytics, scheduler.  |
| `packages/anchor-plugin`  | **Phase 2** — `@rustag/anchor-plugin` ephemeral stagenet for Anchor. |
| `examples/`               | Runnable examples against a live stagenet.                           |

---

## CLI reference

| Command                                            | Description                              |
| -------------------------------------------------- | ---------------------------------------- |
| `rustag create <name>`                             | Register a new stagenet.                 |
| `rustag start [name] [--preload jupiter pyth ...]` | Run the JSON-RPC, WebSocket, REST servers. |
| `rustag stop [-s name]`                            | Stop a running stagenet.                 |
| `rustag status [-s name]`                          | Show counts, ports, running state.       |
| `rustag list`                                      | List all stagenets.                      |
| `rustag airdrop -s name <pubkey> <sol>`            | Credit SOL to a wallet.                  |
| `rustag override -s name --pubkey <pk> --lamports <n>` | Pin account state.                   |
| `rustag preload -s name jupiter pyth raydium`      | Load real mainnet programs/oracles.      |
| `rustag logs -s name --follow`                     | Tail the transaction feed.               |
| `rustag schedule add <name> "<expr>" --airdrop <pk> --sol <n>` | **Phase 2** — recurring on-chain activity. |
| `rustag schedule list / rm <id> / toggle <id>`     | **Phase 2** — manage activities.         |
| `rustag metrics [--series <s>] [--limit <n>]`      | **Phase 2** — analytics time-series.     |

---

## Development

```bash
just build      # cargo build --workspace
just test       # cargo test --workspace
just lint       # clippy -D warnings + fmt --check
just ci         # lint + test
just test-all   # include the network/mainnet tests
```

Requires Rust 1.96+ (pinned in `rust-toolchain.toml`), Node 22+, and pnpm 10+.

---

## Status & roadmap

Phase 1 (this repo) is a working local MVP: lazy mirror, dirty/clean tracking,
unlimited airdrops, overrides, Solana-compatible RPC, persistence, CLI, SDK, and
dashboard. Known limitation: executing *arbitrary mainnet programs* end-to-end (such as a
full Jupiter swap) needs the more complete program-loading planned for Phase 2 — your own
deployed program reading real mainnet state works today.

**Phase 2 (shipped in this repo):**

- **Real-time mirror** — push updates over the `accountSubscribe` WebSocket
  (the protocol Yellowstone/Geyser RPCs serve), sub-second oracle refresh.
  Build with `--features realtime`.
- **Activity Scheduler** — recurring on-chain actions (`@every`/cron).
- **Simulation framework** — fork a stagenet, replay/stress/compare scenarios
  ("what if 1,000 users liquidate at once?").
- **Analytics** — TVL / tx-volume / mirror-growth time-series + dashboard charts.
- **Cloud control plane** (`rustag-cloud`) — multi-tenant orchestration of hosted
  stagenets behind a reverse proxy with API-key auth and process isolation.
- **GitHub Action** — ephemeral stagenet per PR.
- **Anchor plugin** — `@rustag/anchor-plugin` for tests against real mainnet state.

See [`docs/PHASE2.md`](docs/PHASE2.md) for usage, the
[Phase 1 completion checklist](docs/phase1-completion-checklist.md), and the
[Phase 2 master prompt](docs/STAGESVM_PHASE2_PROMPT.md).

---

*RustAG — because the best DeFi bugs are the ones you find in staging, not on mainnet.*
*Open source. MIT OR Apache-2.0.*
