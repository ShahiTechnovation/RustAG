import type { EvmAddress, EvmHex, RehearsalRequest } from "./types";

function isEvmAddress(value: string): boolean {
  return /^0x[0-9a-fA-F]{40}$/.test(value);
}

function toEvmAddress(value: string): EvmAddress {
  if (!isEvmAddress(value)) throw new Error("Enter a valid 0x-prefixed EVM address (40 hex characters).");
  return value as EvmAddress;
}

function parseEthToWei(eth: string): string {
  if (!/^(0|[1-9]\d*)(\.\d{1,18})?$/.test(eth)) {
    throw new Error("ETH value must be a non-negative decimal with at most 18 decimal places.");
  }
  const [whole, frac = ""] = eth.split(".");
  const fracPadded = frac.padEnd(18, "0");
  const wei = BigInt(whole) * BigInt("1000000000000000000") + BigInt(fracPadded);
  if (wei >= 2n ** 256n) throw new Error("ETH value exceeds the EVM uint256 limit.");
  return wei.toString();
}

export interface RawFormFields { from: string; to: string; value: string; data: string; block: string; }

export function parseTransaction(input: RawFormFields, chainId: number): RehearsalRequest {
  if (!isEvmAddress(input.to)) throw new Error("Enter a valid target contract address (0x-prefixed, 40 hex chars).");
  if (input.from && !isEvmAddress(input.from)) throw new Error("Enter a valid executor address.");
  if (!/^0x([0-9a-fA-F]{2})*$/.test(input.data)) throw new Error("Calldata must be 0x-prefixed, whole hexadecimal bytes.");
  if (input.block && !/^(0|[1-9]\d*)$/.test(input.block)) throw new Error("Block number must be a non-negative integer.");
  return {
    chainId,
    from: input.from ? toEvmAddress(input.from) : undefined,
    to: toEvmAddress(input.to),
    value: parseEthToWei(input.value),
    data: input.data as EvmHex,
    blockNumber: input.block || undefined,
  };
}

export function parseTransactionJson(text: string, chainId: number): RawFormFields {
  let tx: Record<string, unknown>;
  try { tx = JSON.parse(text) as Record<string, unknown>; }
  catch { throw new Error("Invalid JSON. Paste a valid transaction JSON object."); }
  if (!tx || typeof tx !== "object" || Array.isArray(tx)) throw new Error("Expected a transaction JSON object.");
  if (tx.chainId != null && Number(tx.chainId) !== chainId) throw new Error("JSON chain ID does not match the selected network.");
  if (tx.transactions != null && (!Array.isArray(tx.transactions) || tx.transactions.length !== 1)) throw new Error("Import one transaction at a time; batches are not supported yet.");
  const item = (tx.transactions as Record<string, unknown>[] | undefined)?.[0] ?? tx;
  const wei = String(item.value ?? "0");
  if (!/^\d+$/.test(wei)) throw new Error("JSON value must be an integer string in wei.");
  const weiBig = BigInt(wei);
  const whole = weiBig / 10n ** 18n;
  const fracStr = (weiBig % 10n ** 18n).toString().padStart(18, "0").replace(/0+$/, "") || "0";
  return {
    from: String(item.from ?? (tx as Record<string, unknown>).safe ?? (tx as Record<string, Record<string, unknown>>).meta?.createdFromSafeAddress ?? ""),
    to: String(item.to ?? ""),
    value: `${whole}.${fracStr}`,
    data: String(item.data ?? "0x"),
    block: String((tx as Record<string, unknown>).blockNumber ?? ""),
  };
}
