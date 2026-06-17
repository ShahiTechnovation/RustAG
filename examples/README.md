# RustAG Examples

Runnable scripts that talk to a **live** stagenet via a standard
`@solana/web3.js` `Connection` — proving RustAG is a drop-in cluster.

## Setup

```bash
# 1. In one terminal, start a stagenet with real mainnet state:
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
rustag create demo
rustag start demo --preload pyth raydium jupiter

# 2. In this folder, install deps:
cd examples
npm install
```

The scripts default to `http://127.0.0.1:8899` (RPC) and `http://127.0.0.1:9000` (REST).
Override with `RUSTAG_RPC_URL` / `RUSTAG_API_URL`.

## Run

```bash
npm run pyth       # read the real SOL/USD Pyth price through the stagenet
npm run raydium    # read a real Raydium AMM pool account (mirrored from mainnet)
npm run jupiter    # airdrop + preload Jupiter + a confirmed transfer
```

Each script demonstrates that the lazy mirror surfaces **real mainnet state** locally,
with unlimited airdrops and zero SOL spent. See each file's header comment for details.
