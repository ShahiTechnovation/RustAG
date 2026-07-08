/**
 * Client-side Pyth price decoding.
 *
 * The demo mirrors real mainnet Pyth price accounts (`preload pyth`), and the
 * REST API already serves their raw bytes as `dataBase64`, so we can decode the
 * live aggregate price in the browser — no special endpoint needed. A reviewer
 * can cross-check the number against any price site: it's real mainnet state.
 */

/** Pyth USD price feeds on mainnet-beta, mirrored by the demo. */
export const PYTH_FEEDS = [
  { symbol: "SOL/USD", pubkey: "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG" },
  { symbol: "ETH/USD", pubkey: "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB" },
  { symbol: "USDC/USD", pubkey: "Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD" },
] as const;

const PYTH_MAGIC = 0xa1b2c3d4;

/**
 * Decode the aggregate price from a base64-encoded Pyth v2 price account.
 * Layout: magic (u32 LE) @ 0, expo (i32 LE) @ 20, agg.price (i64 LE) @ 208.
 * Returns `null` if the bytes are not a recognizable Pyth v2 account.
 */
export function decodePythPrice(base64: string): number | null {
  try {
    const bin = atob(base64);
    if (bin.length < 216) return null;
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    const dv = new DataView(bytes.buffer);
    if (dv.getUint32(0, true) !== PYTH_MAGIC) return null;
    const expo = dv.getInt32(20, true);
    const rawPrice = dv.getBigInt64(208, true);
    return Number(rawPrice) * 10 ** expo;
  } catch {
    return null;
  }
}
