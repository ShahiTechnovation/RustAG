# API Reference

A RustAG stagenet exposes three surfaces:

- **JSON-RPC** (default `:8899`) — Solana-compatible; point any client here.
- **WebSocket** (default `:8900`) — `accountSubscribe` (poll-based) + unary JSON-RPC.
- **REST** (default `:9000`) — for the dashboard and the `@rustag/sdk`.

## JSON-RPC (Solana-compatible)

`POST /` with a standard JSON-RPC 2.0 body. Single requests and batches are supported.

| Method | Notes |
| ------ | ----- |
| `getHealth` | Returns `"ok"`. |
| `getVersion` | `{ "solana-core": "2.1.0", "feature-set": 0 }`. |
| `getGenesisHash` / `getIdentity` | Fixed stagenet values. |
| `getSlot` / `getBlockHeight` | Monotonic slot (advances per transaction). |
| `getEpochInfo` | Slot-derived epoch info. |
| `getLatestBlockhash` | `{ blockhash, lastValidBlockHeight }`. |
| `isBlockhashValid` | Always `true` (the stagenet blockhash never expires). |
| `getMinimumBalanceForRentExemption` | `[dataLen]` → lamports. |
| `getBalance` | `[pubkey]` → `{ context, value }`. Lazily mirrors. |
| `getAccountInfo` | `[pubkey, {encoding}]` → base64 account or `null`. Lazily mirrors. |
| `getMultipleAccounts` | `[[pubkey,…], {encoding}]`. |
| `getProgramAccounts` | `[programId]` → accounts owned by the program. |
| `getTokenAccountBalance` | `[tokenAccount]` → SPL amount. |
| `requestAirdrop` | `[pubkey, lamports]` → signature. |
| `sendTransaction` | `[txBlob, {encoding}]` → signature. base64 or base58. |
| `simulateTransaction` | `[txBlob, {encoding}]` → `{ err, logs, unitsConsumed }`. |
| `getSignatureStatuses` | `[[sig,…]]` → statuses (used by `confirmTransaction`). |
| `getTransaction` | `[signature]` → indexed meta. |
| `getFeeForMessage` | Fixed 5000-lamport fee. |

Example:

```bash
curl -s http://127.0.0.1:8899 -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getBalance","params":["<PUBKEY>"]}'
```

## WebSocket

Connect to `ws://127.0.0.1:8900` and send:

```json
{ "jsonrpc": "2.0", "id": 1, "method": "accountSubscribe", "params": ["<PUBKEY>"] }
```

You receive a subscription id, then `accountNotification` messages whenever the account
changes (polled every ~1s). `accountUnsubscribe` cancels.

`signatureSubscribe` is also implemented — it fires one `signatureNotification` once the
transaction is found, then auto-cancels. This is what `@solana/web3.js` uses to confirm
transactions, so `sendAndConfirmTransaction` / `confirmTransaction` work out of the box.
`slotSubscribe` is accepted for compatibility (no slot stream is emitted). Any other
JSON-RPC method sent over the socket is dispatched like the HTTP endpoint.

## REST (dashboard / SDK)

Base: `http://127.0.0.1:9000`.

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

### `@rustag/sdk`

```ts
import { RustagClient } from "@rustag/sdk";

const client = new RustagClient({ baseUrl: "http://localhost:9000" });

await client.getStagenet();
await client.listAccounts({ limit: 100 });
await client.getAccount("<PUBKEY>");
await client.listTransactions({ limit: 50 });
await client.airdrop("<PUBKEY>", 1000);
await client.overrideAccount({ pubkey: "<PUBKEY>", lamports: 5_000_000_000 });
await client.preload(["jupiter", "pyth", "raydium"]);
```

## Preload targets

`jupiter`, `pyth`, `raydium`, `orca`, `marinade`, `spl-token`,
`token-2022`, `metaplex`. Run `rustag preload -s <name>` with no args to list them.
