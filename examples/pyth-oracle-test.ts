/**
 * Pyth oracle test.
 *
 * Reads the SOL/USD Pyth price feed *through the stagenet*. On first access the
 * account is lazily fetched from mainnet, so you get the real, current price —
 * with no mainnet SOL spent. Run a stagenet first (see README).
 */
import { Connection, PublicKey } from "@solana/web3.js";

const RPC_URL = process.env.RUSTAG_RPC_URL ?? "http://127.0.0.1:8899";

// Pyth SOL/USD price account on mainnet-beta.
const SOL_USD_PYTH = new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");

/** Decode the aggregate price from a Pyth v2 price account. */
function decodePythPrice(data: Buffer): { price: number; expo: number } {
  // Pyth v2 layout: expo (i32) @ 20, agg.price (i64) @ 208.
  const expo = data.readInt32LE(20);
  const rawPrice = data.readBigInt64LE(208);
  const price = Number(rawPrice) * 10 ** expo;
  return { price, expo };
}

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");
  console.log(`Connected to RustAG stagenet at ${RPC_URL}`);

  const info = await connection.getAccountInfo(SOL_USD_PYTH);
  if (!info) {
    throw new Error("Pyth SOL/USD account not found — is mirroring enabled?");
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
