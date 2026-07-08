/**
 * Pyth oracle test.
 *
 * Reads the SOL/USD Pyth price feed *through the stagenet*. On first access the
 * account is lazily fetched from mainnet, so you get the real, current price -
 * with no mainnet SOL spent. Run a stagenet first (see README).
 */
import { Connection, PublicKey } from "@solana/web3.js";

const RPC_URL = process.env.RUSTAG_RPC_URL ?? "http://127.0.0.1:8899";

// Pyth SOL/USD pull-oracle price-feed account on mainnet-beta.
const SOL_USD_PYTH = new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

/** Decode the price from a Pyth `PriceUpdateV2` account (pull oracle). */
function decodePythPrice(data: Buffer): { price: number; expo: number } {
  // 8-byte discriminator, write_authority (32) @ 8, verification_level @ 40
  // (Full = 1 byte; Partial = tag + u8), then PriceFeedMessage: feed_id (32),
  // price (i64), conf (u64), exponent (i32). The level's width varies per update.
  const msg = 41 + (data.readUInt8(40) === 0 ? 1 : 0);
  const expo = data.readInt32LE(msg + 48);
  const rawPrice = data.readBigInt64LE(msg + 32);
  const price = Number(rawPrice) * 10 ** expo;
  return { price, expo };
}

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");
  console.log(`Connected to RustAG stagenet at ${RPC_URL}`);

  const info = await connection.getAccountInfo(SOL_USD_PYTH);
  if (!info) {
    throw new Error("Pyth SOL/USD account not found - is mirroring enabled?");
  }

  console.log(`Owner:     ${info.owner.toBase58()}`);
  console.log(`Data size: ${info.data.length} bytes (mirrored from mainnet)`);

  const { price, expo } = decodePythPrice(Buffer.from(info.data));
  console.log(`\n  SOL/USD = $${price.toFixed(2)}  (expo ${expo})\n`);
  console.log("✓ Real mainnet oracle state, read locally, zero SOL spent.");
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("✗", err.message);
    process.exit(1);
  });
