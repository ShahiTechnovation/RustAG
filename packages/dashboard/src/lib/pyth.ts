/**
 * Client-side Pyth price decoding.
 *
 * The demo mirrors real mainnet Pyth price-feed accounts (`preload pyth`), and
 * the REST API already serves their raw bytes as `dataBase64`, so we can decode
 * the live price in the browser — no special endpoint needed. A reviewer can
 * cross-check the number against any price site: it's real mainnet state.
 *
 * These are Pyth's pull-oracle "price feed accounts" — an Anchor
 * `PriceUpdateV2` account owned by the receiver program. The legacy v2 "price
 * accounts" (magic 0xa1b2c3d4, `agg.price` @ 208) were deprecated by Pyth and
 * stopped updating in Nov 2024, which would freeze the displayed price.
 */

/** Pyth pull-oracle USD price-feed accounts on mainnet-beta, mirrored by the demo. */
export const PYTH_FEEDS = [
  { symbol: "SOL/USD", pubkey: "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE" },
  { symbol: "ETH/USD", pubkey: "42amVS4KgzR9rA28tkVYqVXjq9Qa8dcZQMbH5EYFX6XC" },
  { symbol: "USDC/USD", pubkey: "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX" },
  { symbol: "USDT/USD", pubkey: "HT2PLQBcG5EiCcNSaMHAjSgd9F98ecpATbk4Sk5oYuM" },
] as const;

/** Anchor account discriminator = sha256("account:PriceUpdateV2")[0..8]. */
const PRICE_UPDATE_V2_DISCRIMINATOR = [0x22, 0xf1, 0x23, 0x63, 0x9d, 0x7e, 0xf4, 0xcd];

/**
 * Decode the price from a base64-encoded Pyth `PriceUpdateV2` account.
 *
 * Layout: 8-byte Anchor discriminator, `write_authority` (32) @ 8,
 * `verification_level` @ 40, then the `PriceFeedMessage` — `feed_id` (32),
 * `price` (i64), `conf` (u64), `exponent` (i32). `verification_level` is a Borsh
 * enum whose width varies between updates (`Full` = 1 byte; `Partial` = 1 tag +
 * 1 byte `num_signatures`), so we read its tag byte to place the message.
 * Returns `null` if the bytes are not a recognizable `PriceUpdateV2` account.
 */
export function decodePythPrice(base64: string): number | null {
  try {
    const bin = atob(base64);
    if (bin.length < 132) return null;
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    for (let i = 0; i < 8; i++) {
      if (bytes[i] !== PRICE_UPDATE_V2_DISCRIMINATOR[i]) return null;
    }
    const dv = new DataView(bytes.buffer);
    // verification_level @ 40: 0 = Partial (tag + u8 num_signatures), else Full.
    const msg = 41 + (bytes[40] === 0 ? 1 : 0); // start of PriceFeedMessage
    const price = dv.getBigInt64(msg + 32, true); // after feed_id[32]
    const expo = dv.getInt32(msg + 48, true); // after price (i64) + conf (u64)
    return Number(price) * 10 ** expo;
  } catch {
    return null;
  }
}
