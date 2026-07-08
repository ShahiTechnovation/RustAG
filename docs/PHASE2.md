# RustAG Phase 2 — Feature Guide

Phase 2 turns RustAG from a local devtool into a real-time, schedulable,
simulatable, multi-tenant platform. Everything below is built on the Phase 1
invariant: **a `Dirty` or `Pinned` account is never overwritten by any sync.**

- Real-time mirror (push) · Activity Scheduler · Simulation framework ·
  Analytics · Cloud control plane · GitHub Action · Anchor plugin

---

## 1. Real-time mirror (push updates)

Phase 1 refreshes CLEAN oracle accounts by polling every 30s. Phase 2 adds a
**push** path over the standard `accountSubscribe` WebSocket (the protocol
Yellowstone/Geyser-backed RPCs serve), so oracle prices update sub-second.

It is behind a cargo feature so the default build stays dependency-light:

```bash
cargo build -p rustag-cli --features realtime
```

Enable it on a stagenet (config or `RustAG.toml`):

```toml
realtime_enabled = true
realtime_ws      = "wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

The consumer applies pushed updates only to non-dirty accounts — the same
invariant, enforced on the push path. A native **Yellowstone gRPC** source is a
drop-in alternative: implement a producer that sends `RemoteAccount`s into the
same `mpsc::Sender` that `rustag_core::spawn_realtime_apply` consumes.

---

## 2. Activity Scheduler

Schedule recurring on-chain actions to simulate realistic, ongoing usage.

```bash
# Top up a faucet wallet every 30 seconds
rustag schedule add faucet "@every 30s" --airdrop <WALLET> --sol 1

# Replay a captured swap every 5 minutes (blockhash never expires on a stagenet)
rustag schedule add swap "*/5 * * * *" --raw-tx <BASE64_SIGNED_TX>

# Periodic transfer from a funded staging wallet
rustag schedule add pay "@hourly" --transfer-from <SECRET_BASE58> --to <WALLET> --sol 0.5

rustag schedule list
rustag schedule toggle <id> --off
rustag schedule rm <id>
```

Schedule expressions: `@every 30s` / `1h30m` / `2d`, the aliases
`@minutely`/`@hourly`/`@daily`/`@weekly`/`@monthly`, or a 5-field cron
(`*/5 * * * *`, `0 9 * * 1-5`). Via the SDK:

```ts
await client.createSchedule({
  name: "faucet",
  schedule: "@every 30s",
  action: { type: "airdrop", pubkey: WALLET, sol: 1 },
});
```

---

## 3. Simulation framework

Fork a stagenet into an isolated in-memory copy and replay/stress/compare
transactions — the base is never mutated.

```ts
// Replay signed transactions against a fork
const report = await client.simulate([txBase64a, txBase64b], { label: "swaps" });
console.log(report.succeeded, "/", report.total, "succeeded");
```

In Rust (e.g. "what if 1,000 users liquidate at once?"):

```rust
let mut fork = base.fork("herd").await?;
let report = rustag_sim::stress(&mut fork, "liquidations", 1_000, |i| build_tx(i)).await?;
println!("success rate: {:.1}%", report.success_rate() * 100.0);
```

`compare(base, variants)` forks once per variant and reports them side by side.

---

## 4. Analytics

A background sampler captures TVL (Σ lamports), transaction volume, accounts
mirrored, dirty count, and slot on an interval (default 60s, bounded retention).

```bash
rustag metrics --series tvl_lamports --limit 50
```

```ts
const metrics = await client.getMetrics({ limit: 120 }); // { series: [{ t, v }] }
```

The dashboard renders these as sparklines under **Analytics**. The `metrics`
table is a Postgres/TimescaleDB hypertable on `recorded_at` in production.

---

## 5. Cloud control plane (`rustag-cloud`)

Multi-tenant orchestrator: each hosted stagenet runs as an isolated child
`rustag` process with its own ports and data dir, reachable through a reverse
proxy. API-key auth; tenant-scoped.

```bash
RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=KEY" \
RUSTAG_BIN="$PWD/target/release/rustag" \
  cargo run -p rustag-cloud -- serve            # listens on 127.0.0.1:8080
```

```bash
# 1. Sign up — returns your API key ONCE
curl -s localhost:8080/v1/signup -d '{"name":"Acme","email":"a@acme.dev"}'

# 2. Create + start a stagenet
curl -s localhost:8080/v1/stagenets -H "authorization: Bearer rk_..." \
  -d '{"name":"my-project","mainnetRpc":"https://..."}'

# 3. Talk to it through the proxy (point any Solana client here)
curl -s localhost:8080/my-project/rpc -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'

# 4. Tear it down
curl -s -X DELETE localhost:8080/my-project -H "authorization: Bearer rk_..."
```

In production: front the proxy with TLS + a per-subdomain router so each project
gets `my-project.stagesvm.dev`; swap the control-plane SQLite for Postgres (the
schema is portable) and put the slug→port routing table behind Redis. Web auth
(Clerk/NextAuth) sits in front and mints API keys via `/v1/api-keys`.

---

## 6. GitHub Action — ephemeral stagenet per PR

```yaml
- uses: ./.github/actions/rustag      # or rustag/github-action@v1
  with:
    mainnet-rpc: ${{ secrets.RUSTAG_MAINNET_RPC }}
    preload: "pyth raydium"
    run: |
      cd examples && npm install && npx tsx pyth-oracle-test.ts
```

The action builds the CLI, boots an ephemeral stagenet, runs your command with
`RUSTAG_RPC_URL` / `ANCHOR_PROVIDER_URL` set, posts a summary to the PR, and
tears the stagenet down. See [`.github/workflows/rustag-pr.yml`](../.github/workflows/rustag-pr.yml).

---

## 7. Anchor plugin (`@rustag/anchor-plugin`)

```ts
import { rustagAnchorProvider } from "@rustag/anchor-plugin";
import { setProvider } from "@coral-xyz/anchor";

const { provider, stagenet } = await rustagAnchorProvider({ preload: ["pyth"] });
setProvider(provider);                 // funded wallet, real mainnet state
// ... run your Anchor program ...
await stagenet.stop();
```

`EphemeralStagenet.start()` / `withEphemeralStagenet()` give you the same
lifecycle without Anchor. See [`packages/anchor-plugin`](../packages/anchor-plugin).
