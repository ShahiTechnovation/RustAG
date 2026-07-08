# RustAG — Full Tutorial

A hands-on, copy-pasteable walkthrough of **every** RustAG command and integration path:
the CLI, the JSON-RPC endpoint, the REST API, the TypeScript SDK, the dashboard, and the
examples. If you just want the 2-minute version, see [getting-started.md](getting-started.md).

---

## 0. The mental model (read this first)

RustAG runs a **stagenet** — a local Solana environment that *mirrors mainnet on demand*:

- When something reads an account RustAG doesn't have, it **fetches it from mainnet**,
  caches it, and marks it `Clean`.
- When a transaction **writes** an account, RustAG marks it `Dirty` and freezes it from
  mainnet sync forever.
- A background task refreshes `Clean` **oracle** accounts (Pyth/Switchboard) every 30s.
- You can **airdrop unlimited SOL**, **override** any account, and point any Solana client
  at it as if it were a real cluster.

There are two long-running processes you interact with:

| Process | Default port | What it is |
| ------- | ------------ | ---------- |
| **JSON-RPC** | `8899` | Solana-compatible RPC — point `@solana/web3.js`, `anchor`, `solana` here. |
| **WebSocket** | `8900` | `accountSubscribe` push (poll-based in Phase 1). |
| **REST API** | `9000` | Powers the dashboard + the `@rustag/sdk` + the CLI client commands. |

A stagenet's data lives in **`./.rustag/db.sqlite`** (project-local). All commands operate
on the stagenet registry in the current directory.

---

## 1. Prerequisites

- **Rust 1.96+** — `rust-toolchain.toml` pins it; `rustup` installs it automatically.
- **Node 22+** and **pnpm 10+** — only for the SDK and dashboard.
- A **mainnet RPC endpoint**. The public endpoint works but is rate-limited; a free
  [Helius](https://helius.dev)/[Triton](https://triton.one) key is strongly recommended.

Verify:

```bash
rustc --version    # 1.96.0+
node --version     # v22+
pnpm --version     # 10+
```

---

## 2. Build & install the CLI

```bash
# From the repo root:
cargo build --release          # produces target/release/rustag(.exe)
```

Either call the binary by path (`./target/release/rustag …`) or put it on your `PATH`:

```bash
# macOS/Linux
export PATH="$PWD/target/release:$PATH"
# Windows PowerShell
$env:PATH = "$PWD\target\release;$env:PATH"
```

From here on, the tutorial writes `rustag` — substitute the full path if you skipped this.

Check it works:

```bash
rustag --version        # rustag 0.1.0
rustag --help           # lists all commands
```

---

## 3. Configure the mainnet mirror

```bash
cp .env.example .env.local
```

Set your endpoint (used by every stagenet you create afterwards):

```bash
# macOS/Linux
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
# Windows PowerShell
$env:RUSTAG_MAINNET_RPC = "https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

---

## 4. The complete CLI reference

> Convention: `-s, --stagenet <NAME>` selects which stagenet a command targets. If you
> only have one stagenet, you can omit it.

### `rustag create <NAME>` — register a stagenet

```bash
rustag create my-defi-project
```

```
  ✓ Created stagenet 'my-defi-project' (id: a1b2c3d4)
  ✓ RPC endpoint: http://127.0.0.1:8899
  ✓ WebSocket:    ws://127.0.0.1:8900
  ✓ REST API:     http://127.0.0.1:9000
  • Start it with: rustag start my-defi-project
```

Flags:

| Flag | Default | Meaning |
| ---- | ------- | ------- |
| `--rpc-port <PORT>` | `8899` | JSON-RPC port. |
| `--ws-port <PORT>` | `rpc_port + 1` | WebSocket port. |
| `--api-port <PORT>` | `9000` | REST API port. |
| `--mainnet-rpc <URL>` | `$RUSTAG_MAINNET_RPC` or public | Endpoint the mirror fetches from. |
| `--no-mirror` | off | Fully offline stagenet (no mainnet fetches). |

Run multiple stagenets side-by-side by giving each distinct ports:

```bash
rustag create alt --rpc-port 18899 --api-port 19000
```

### `rustag start [NAME]` — run the servers

This is the long-running process. Run it in its own terminal.

```bash
rustag start my-defi-project --preload pyth raydium
```

```
  ✓ Opened stagenet 'my-defi-project' (id: a1b2c3d4)
  ✓ Preloaded 4 accounts from mainnet
  ✓ RPC endpoint: http://127.0.0.1:8899
  ✓ WebSocket:    ws://127.0.0.1:8900
  ✓ REST API:     http://127.0.0.1:9000
  • Point your cluster URL at the RPC endpoint. Press Ctrl-C to stop.
```

- `NAME` is optional if only one stagenet exists.
- `--preload <names…>` loads programs/oracles on startup (same names as `preload` below).
- Press **Ctrl-C** to stop, or use `rustag stop` from another terminal.

### `rustag stop [-s NAME]` — stop a running stagenet

```bash
rustag stop -s my-defi-project
```

Stops the process recorded in `./.rustag/<name>.pid` (validated, and on Windows filtered to
the `rustag` image so a recycled PID is never force-killed).

### `rustag status [-s NAME]` — inspect a stagenet

```bash
rustag status -s my-defi-project
```

```
  my-defi-project
  • id:           a1b2c3d4
  • status:       running
  • rpc:          http://127.0.0.1:8899
  • rest api:     http://127.0.0.1:9000
  • mainnet rpc:  https://mainnet.helius-rpc.com/?api-key=…
  • mirror:       enabled
  • accounts:     5
  • transactions: 2
```

### `rustag list` — list all stagenets

```bash
rustag list
```

```
  NAME                     ID         RPC                        STATUS
  my-defi-project          a1b2c3d4   http://127.0.0.1:8899      running
  alt                      e5f6a7b8   http://127.0.0.1:18899     stopped
```

### `rustag airdrop -s NAME <PUBKEY> <SOL>` — unlimited faucet

> Requires the stagenet to be **running** (talks to its REST API).

```bash
rustag airdrop -s my-defi-project GsbwXfJraMomNxBcjK8h8gPbBzqVuU7m5jbB1c9Wp4Lp 1000
```

```
  ✓ Airdropped 1000 SOL to GsbwXfJraMomNxBcjK8h8gPbBzqVuU7m5jbB1c9Wp4Lp
```

### `rustag override -s NAME …` — set account state

> Requires the stagenet to be **running**. Pins the account (immune to mainnet sync).

```bash
# Set a wallet's lamport balance directly
rustag override -s my-defi-project \
  --pubkey GsbwXfJraMomNxBcjK8h8gPbBzqVuU7m5jbB1c9Wp4Lp \
  --lamports 5000000000

# Set an SPL token account's amount (raw units)
rustag override -s my-defi-project \
  --pubkey <TOKEN_ACCOUNT> \
  --token-balance 1000000
```

Pass `--lamports` **or** `--token-balance` (at least one).

### `rustag preload -s NAME <PROGRAMS…>` — load real mainnet state

> Requires the stagenet to be **running**.

```bash
rustag preload -s my-defi-project jupiter pyth raydium
```

```
  ✓ Preloaded 5 accounts from mainnet
```

Run with no arguments to list the available targets:

```bash
rustag preload -s my-defi-project
#   • available targets: jupiter, pyth, switchboard, raydium, orca, marinade, spl-token, token-2022, metaplex
```

| Target | Loads |
| ------ | ----- |
| `jupiter` | Jupiter V6 aggregator program |
| `pyth` | Pyth SOL/USD, ETH/USD, USDC/USD price feeds |
| `switchboard` | Switchboard USDT/USD feed |
| `raydium` | Raydium AMM v4 program |
| `orca` | Orca Whirlpools program |
| `marinade` | Marinade staking program |
| `spl-token` | SPL Token + Associated Token programs |
| `token-2022` | Token-2022 program |
| `metaplex` | Token Metadata + Core programs |

### `rustag logs -s NAME [-f]` — tail the transaction feed

```bash
rustag logs -s my-defi-project --follow
```

```
  [12:34:56] ✓ JUP6…TaV4 (CUs: 89,234)
  [12:34:57] ✓ — (CUs: 450)
  [12:34:58] ✗ 675k…1Mp8 (CUs: 23,401)
```

`-f, --follow` streams new transactions until you Ctrl-C; without it, prints the recent
history and exits.

---

## 5. The full walkthrough (the "wow" flow)

Two terminals.

**Terminal A — run the stagenet:**

```bash
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
rustag create demo
rustag start demo --preload pyth raydium jupiter
```

**Terminal B — drive it:**

```bash
# 1. Unlimited airdrop
rustag airdrop -s demo GsbwXfJraMomNxBcjK8h8gPbBzqVuU7m5jbB1c9Wp4Lp 1000

# 2. Read a REAL mainnet oracle through the stagenet
curl -s http://127.0.0.1:8899 -H 'content-type: application/json' -d '{
  "jsonrpc":"2.0","id":1,"method":"getAccountInfo",
  "params":["H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG",{"encoding":"base64"}]
}'

# 3. Watch transactions live
rustag logs -s demo --follow
```

Point your existing tooling at it — change one line:

```bash
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor test
solana config set --url http://127.0.0.1:8899
solana balance GsbwXfJraMomNxBcjK8h8gPbBzqVuU7m5jbB1c9Wp4Lp
```

---

## 6. Using it from `@solana/web3.js`

The JSON-RPC endpoint is a drop-in cluster URL:

```ts
import { Connection, Keypair, LAMPORTS_PER_SOL, SystemProgram, Transaction,
         sendAndConfirmTransaction } from "@solana/web3.js";

const connection = new Connection("http://127.0.0.1:8899", "confirmed");

// Airdrop works just like a validator faucet — but unlimited
const wallet = Keypair.generate();
await connection.requestAirdrop(wallet.publicKey, 100 * LAMPORTS_PER_SOL);

// Send a normal transaction
const to = Keypair.generate();
const tx = new Transaction().add(SystemProgram.transfer({
  fromPubkey: wallet.publicKey, toPubkey: to.publicKey, lamports: LAMPORTS_PER_SOL,
}));
const sig = await sendAndConfirmTransaction(connection, tx, [wallet]);
```

---

## 7. JSON-RPC methods (port 8899)

`POST /` with a JSON-RPC 2.0 body (single or batch). Implemented:

`getHealth`, `getVersion`, `getGenesisHash`, `getIdentity`, `getSlot`, `getBlockHeight`,
`getEpochInfo`, `getLatestBlockhash`, `isBlockhashValid`, `getMinimumBalanceForRentExemption`,
`getBalance`, `getAccountInfo`, `getMultipleAccounts`, `getProgramAccounts` (supports
`dataSize`/`memcmp` filters), `getTokenAccountBalance`, `requestAirdrop`, `sendTransaction`,
`simulateTransaction`, `getSignatureStatuses`, `getTransaction`, `getFeeForMessage`.

```bash
# getBalance
curl -s http://127.0.0.1:8899 -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getBalance","params":["<PUBKEY>"]}'

# requestAirdrop (lamports)
curl -s http://127.0.0.1:8899 -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"requestAirdrop","params":["<PUBKEY>", 1000000000]}'
```

---

## 8. REST API (port 9000)

Used by the dashboard, the SDK, and the CLI client commands.

| Method & path | Body / query | Returns |
| ------------- | ------------ | ------- |
| `GET /api/health` | — | `{ status }` |
| `GET /api/stagenet` | — | id, name, ports, counts, mirror state |
| `GET /api/accounts` | `?limit&offset` | `{ accounts: [...] }` |
| `GET /api/accounts/:pubkey` | — | one account (lazily mirrored) |
| `GET /api/transactions` | `?limit` | `{ transactions: [...] }` |
| `POST /api/airdrop` | `{ pubkey, sol }` | `{ signature, lamports }` |
| `POST /api/override` | `{ pubkey, lamports?, tokenBalance? }` | `{ ok }` |
| `POST /api/preload` | `{ programs: [...] }` | `{ loaded, unknown }` |

```bash
curl -s http://127.0.0.1:9000/api/stagenet
curl -s -X POST http://127.0.0.1:9000/api/airdrop \
  -H 'content-type: application/json' -d '{"pubkey":"<PUBKEY>","sol":500}'
```

---

## 9. The TypeScript SDK (`@rustag/sdk`)

```bash
pnpm install        # from repo root, installs the workspace
```

```ts
import { RustagClient } from "@rustag/sdk";
import { Connection } from "@solana/web3.js";

const client = new RustagClient({ baseUrl: "http://localhost:9000" });

const stagenet = await client.getStagenet();
await client.airdrop("<PUBKEY>", 1000);
await client.overrideAccount({ pubkey: "<PUBKEY>", lamports: 5_000_000_000 });
await client.preload(["jupiter", "pyth"]);
const accounts = await client.listAccounts({ limit: 100 });
const txs = await client.listTransactions({ limit: 50 });

// Drop-in Solana connection against the stagenet:
const connection = new Connection(stagenet.rpcUrl);
```

---

## 10. The dashboard (Next.js)

```bash
pnpm install
NEXT_PUBLIC_RUSTAG_API_URL=http://localhost:9000 pnpm --filter dashboard dev
# open http://localhost:3000
```

Pages: **Overview** (stats + airdrop/override/preload actions + recent txs), **Accounts**
(every account with its sync state), **Transactions** (live feed). It polls the REST API,
so keep a stagenet running.

Build for production:

```bash
pnpm --filter dashboard build
```

---

## 11. Runnable examples

```bash
cd examples
npm install
# (with a stagenet running and preloaded with pyth/raydium/jupiter)
npm run pyth        # read the real SOL/USD price through the stagenet
npm run raydium     # read a real Raydium pool account
npm run jupiter     # airdrop + preload + a confirmed transfer
```

---

## 12. Project config — `RustAG.toml`

Commit a `RustAG.toml` so your team shares one setup (see the sample at the repo root). It
mirrors the `StagenetConfig`: ports, `mirror.mainnet_rpc`, sync intervals, preload list,
limits, and storage path. `${VAR}` reads from the environment.

---

## 13. Task runner (`just`)

```bash
just build      # cargo build --workspace
just test       # cargo test --workspace
just test-all   # include network/mainnet tests
just lint       # clippy -D warnings + fmt --check
just ci         # lint + test
just dev -- status        # run the CLI (args after --)
just js-install / js-build / dashboard
```

---

## 14. Troubleshooting

| Symptom | Fix |
| ------- | --- |
| `could not reach the stagenet's REST API` | The client commands need a **running** stagenet — `rustag start <name>` first. |
| Mainnet `429` / slow fetches | Use a keyed `RUSTAG_MAINNET_RPC`; lower `[mirror].max_rps` in `RustAG.toml`. |
| `Port already in use` | `rustag create <name> --rpc-port 18899 --api-port 19000`. |
| `stagenet '<name>' is already running` | One instance per stagenet — `rustag stop` it first, or use a different name/ports. |
| Want a clean slate | Stop stagenets, then delete the `./.rustag` directory. |
| Offline / no mainnet | `rustag create <name> --no-mirror` (airdrops + your own programs still work). |

---

## 15. How it works & limits (the short version)

See [architecture.md](architecture.md) for the full design. In brief: a `Stagenet` wraps
LiteSVM; reads lazily mirror mainnet accounts; writes mark accounts `Dirty`; v0 transactions
get their address-lookup-table accounts resolved and pre-loaded; everything persists to
SQLite and rehydrates on restart.

**Phase 1 limitations:** executing *foreign* mainnet programs end-to-end (e.g. a full
Jupiter swap CPI) needs BPF bytecode loading — that's Phase 2. Your *own* deployed program
reading real mainnet state works today. WebSocket pub/sub is poll-based (Yellowstone gRPC
is Phase 2).

---

*Questions or contributions welcome. RustAG is MIT OR Apache-2.0.*
