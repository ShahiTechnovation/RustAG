import { chainFor, type Network } from "../chains/robinhood";

/** Build a Robinhood Chain explorer URL (respects mainnet/testnet). */
export function explorerUrl(
  network: Network,
  type: "address" | "tx" | "block",
  value: string | number,
): string {
  const base = chainFor(network).explorerUrl.replace(/\/$/, "");
  return base + "/" + type + "/" + encodeURIComponent(String(value));
}

/** Truncate an EVM address or hash for display. */
export function shorten(value: string): string {
  return value.slice(0, 6) + "\u2026" + value.slice(-4);
}
