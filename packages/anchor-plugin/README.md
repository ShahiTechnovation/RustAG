# @rustag/anchor-plugin

Native [Anchor](https://www.anchor-lang.com/) integration for **RustAG** - spin up an
ephemeral, mainnet-mirroring stagenet for your Anchor tests, get a funded
`AnchorProvider`, and tear it down automatically.

Your tests run against **real** Pyth prices, **real** Raydium pools, and **real**
token mints - with unlimited airdrops and zero mainnet SOL.

## Install

```bash
pnpm add -D @rustag/anchor-plugin @coral-xyz/anchor @solana/web3.js
```

Requires the `rustag` CLI on your `PATH` (or set `RUSTAG_BIN`). Set
`RUSTAG_MAINNET_RPC` to a mainnet endpoint to enable mirroring.

## Usage

```ts
import { rustagAnchorProvider } from "@rustag/anchor-plugin";
import { Program, setProvider } from "@coral-xyz/anchor";

describe("my program against mainnet state", () => {
  let ctx: Awaited<ReturnType<typeof rustagAnchorProvider>>;

  before(async () => {
    ctx = await rustagAnchorProvider({ preload: ["pyth", "raydium"] });
    setProvider(ctx.provider);
  });

  after(() => ctx.stagenet.stop());

  it("reads a real Pyth price", async () => {
    // ctx.provider is a normal AnchorProvider - use Program, etc.
    const balance = await ctx.provider.connection.getBalance(ctx.wallet.publicKey);
    // ... your assertions ...
  });
});
```

### Lower-level: just the stagenet

```ts
import { withEphemeralStagenet } from "@rustag/anchor-plugin";

await withEphemeralStagenet({ noMirror: true }, async (sn) => {
  await sn.client.airdrop("<WALLET>", 1000);
  // sn.rpcUrl / sn.wsUrl / sn.apiUrl point at the running stagenet
});
```

## How it works

`EphemeralStagenet.start()` runs `rustag create` + `rustag start` in an isolated
temp directory on randomized ports, polls the REST health endpoint until ready,
and exposes a `RustagClient`. `rustagAnchorProvider()` layers an Anchor provider
with a freshly airdropped wallet on top.
