# RustAG — Demo Recording Script (for judges)

A tight **~3-minute** screen recording that proves the concept. Every command below is
verified to work. Be honest about the one Phase-2 boundary (noted at the end).

---

## What you're proving

1. RustAG gives you **real mainnet state locally** (a live Pyth price), with **$0 spent**.
2. **Unlimited airdrops** — the faucet problem is gone.
3. It's a **drop-in cluster**: a standard `@solana/web3.js` app sends and confirms a real
   transaction against it.
4. A **dashboard** shows accounts, sync state, and the live transaction feed.

---

## Pre-recording setup (do this BEFORE you hit record)

```bash
# 1. Build the binary
cargo build --release
export PATH="$PWD/target/release:$PATH"        # or use ./target/release/rustag

# 2. Point the mirror at a (fast, keyed) mainnet RPC
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"

# 3. Install JS deps once (examples + dashboard)
pnpm install
( cd examples && npm install )

# 4. Pre-warm a wallet keypair you'll airdrop to (or just use this pubkey)
#    GsbwXfJraMomNxBcjK8h8gPbBzqVuU7m5jbB1c9Wp4Lp
```

Arrange **three terminals** + a **browser**:
- **T1** — the stagenet server (`rustag start`)
- **T2** — commands you type live
- **T3** — `examples/` (the web3.js proof)
- **Browser** — `http://localhost:3000` (dashboard)

Start the dashboard in a spare terminal before recording:
```bash
NEXT_PUBLIC_RUSTAG_API_URL=http://localhost:9000 pnpm --filter dashboard dev
```

---

## The script

### [0:00–0:20] The problem
> *"Solana's testnet faucet caps you at ~5 SOL a day, and testnet has no real DeFi state —
> no real Pyth prices, no real pools. So integration tests are either starved for SOL or
> lying against mocks. RustAG fixes both."*

(Show the README's problem block, or just say it over a blank terminal.)

### [0:20–0:45] Spin up a stagenet — **T1**
```bash
rustag create demo
rustag start demo --preload pyth raydium
```
> *"One command creates a local stagenet; `--preload` pulls real Jupiter, Pyth and Raydium
> accounts straight from mainnet."*

Point to the printed endpoints (RPC `:8899`, REST `:9000`).

### [0:45–1:05] Real mainnet state, locally — **T2**
```bash
cd examples
npm run pyth
```
> *"This reads the SOL/USD **Pyth** price feed through the stagenet. On first access RustAG
> lazily fetches it from mainnet — so this is the real, current price, read locally, zero
> SOL spent."*

Show: `SOL/USD = $NNN.NN  (expo -8)`.

### [1:05–1:25] Unlimited airdrop — **T2**
```bash
rustag airdrop -s demo GsbwXfJraMomNxBcjK8h8gPbBzqVuU7m5jbB1c9Wp4Lp 1000
```
> *"Need 1000 SOL? Done. Instant, free, no faucet."*

### [1:25–2:05] It's a drop-in cluster — **T3**
```bash
npm run jupiter
```
> *"This is a plain `@solana/web3.js` script — same code you'd run against mainnet. It
> airdrops, then **sends and confirms a real transaction** against the stagenet."*

Show the output:
```
Airdropped 100 SOL → payer balance: 100 SOL
Transfer confirmed: <signature>
Receiver balance: 1 SOL
✓ Airdrop + preload + confirmed tx, zero mainnet SOL spent.
```
> *"`sendAndConfirmTransaction` resolves — the same way your existing tests would. The only
> thing that changed is the cluster URL."*

### [2:05–2:40] The dashboard — **Browser**
Switch to `http://localhost:3000`.
> *"And here's the visual: every account with its **sync state** — green `Clean` for mainnet
> mirrors, amber `Dirty` for anything a transaction touched — plus a live transaction feed
> with compute units."*

Click **Accounts** (show the Pyth oracle row, `Clean`), then **Transactions** (show the
confirmed transfer), then the **Overview** action panel — click an **airdrop** button and
watch the count tick up.

### [2:40–3:00] Close
> *"On devnet this would've needed dozens of SOL and a flaky faucet. With RustAG: real
> mainnet state, unlimited airdrops, a drop-in RPC, and a dashboard — running locally, free.
> It's open source, and it's the staging layer Solana developers don't have yet."*

---

## Honesty note (have an answer ready)

If a judge asks *"can it run a full Jupiter swap?"*: **your own program** reading real
mainnet state works today; executing a **foreign** on-chain program end-to-end (loading
Jupiter's BPF bytecode from its program-data account) is the headline **Phase 2** item. Say
that plainly — it shows you understand the architecture. (See
[architecture.md](architecture.md#known-limitations).)

---

## Quick reset between takes

```bash
rustag stop -s demo        # or Ctrl-C in T1
rm -rf .rustag             # wipe all stagenet state for a clean run
```
