# Getting Started

From zero to a running stagenet in under two minutes.

## Prerequisites

- **Rust 1.96+** (pinned in `rust-toolchain.toml`; `rustup` will pick it up).
- **Node 22+** and **pnpm 10+** (only needed for the SDK and dashboard).
- A **mainnet RPC endpoint**. The public one works but is heavily rate-limited; a free
  [Helius](https://helius.dev) / [Triton](https://triton.one) key is strongly recommended.

## 1. Build

```bash
git clone <this-repo> && cd rustag
cargo build --release           # target/release/rustag
```

Add `target/release` to your `PATH`, or call the binary directly.

## 2. Configure the mirror

```bash
cp .env.example .env.local
# edit .env.local and set RUSTAG_MAINNET_RPC
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

## 3. Create & start a stagenet

```bash
rustag create demo
rustag start demo --preload pyth raydium
```

`start` runs in the foreground and prints:

```
  ✓ Opened stagenet 'demo' (id: a1b2c3d4)
  ✓ Preloaded 4 accounts from mainnet
  ✓ RPC endpoint: http://127.0.0.1:8899
  ✓ WebSocket:    ws://127.0.0.1:8900
  ✓ REST API:     http://127.0.0.1:9000
```

## 4. Use it

In a second terminal:

```bash
# Unlimited airdrop
rustag airdrop -s demo <YOUR_WALLET> 1000

# Inspect
rustag status -s demo
rustag logs   -s demo --follow

# Read a real mainnet oracle through the stagenet
curl -s http://127.0.0.1:8899 -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG",{"encoding":"base64"}]}'
```

Point your existing tooling at the RPC endpoint:

```bash
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor test
solana config set --url http://127.0.0.1:8899
```

## 5. The dashboard (optional)

```bash
pnpm install
NEXT_PUBLIC_RUSTAG_API_URL=http://localhost:9000 pnpm --filter dashboard dev
# open http://localhost:3000
```

## 6. The SDK (optional)

```ts
import { RustagClient } from "@rustag/sdk";
import { Connection } from "@solana/web3.js";

const client = new RustagClient({ baseUrl: "http://localhost:9000" });
const stagenet = await client.getStagenet();

await client.airdrop(wallet.toBase58(), 1000);

const connection = new Connection(stagenet.rpcUrl); // drop-in mainnet replacement
```

## Troubleshooting

- **"could not reach the stagenet's REST API"** — the client commands (`airdrop`,
  `override`, `preload`, `logs`) talk to a *running* stagenet. Start one with
  `rustag start` first.
- **Mainnet 429s** — set `RUSTAG_MAINNET_RPC` to a keyed endpoint and/or lower
  `[mirror].max_rps` in `RustAG.toml`.
- **Port already in use** — pick different ports with
  `rustag create <name> --rpc-port 18899 --api-port 19000`.
