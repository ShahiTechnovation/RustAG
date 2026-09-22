import { getAddress, isAddress, parseEther, type Hex } from 'viem';
import type { RehearsalRequest } from './types';
export function parseTransaction(input: { from: string; to: string; value: string; data: string; block: string }, chainId: number): RehearsalRequest {
  if (!isAddress(input.to)) throw new Error('Enter a valid target contract address (including a valid checksum for mixed-case addresses).');
  if (input.from && !isAddress(input.from)) throw new Error('Enter a valid executor address.');
  if (!/^(0|[1-9]\d*)(\.\d{1,18})?$/.test(input.value)) throw new Error('ETH value must be non-negative with at most 18 decimal places.');
  if (!/^0x([0-9a-fA-F]{2})*$/.test(input.data)) throw new Error('Calldata must be 0x-prefixed, whole hexadecimal bytes.');
  if (input.block && !/^(0|[1-9]\d*)$/.test(input.block)) throw new Error('Block number must be a non-negative integer.');
  const value = parseEther(input.value);
  if (value >= 2n ** 256n) throw new Error('ETH value exceeds the EVM uint256 limit.');
  return { chainId, from: input.from ? getAddress(input.from) : undefined, to: getAddress(input.to), value: value.toString(), data: input.data as Hex, blockNumber: input.block ? BigInt(input.block) : undefined };
}
export function parseTransactionJson(text: string, chainId: number) {
  const tx = JSON.parse(text);
  if (!tx || typeof tx !== 'object' || Array.isArray(tx)) throw new Error('Expected a transaction JSON object.');
  if (tx.chainId != null && Number(tx.chainId) !== chainId) throw new Error('JSON chain ID does not match the selected network.');
  // Safe exports may wrap the single transaction in transactions[]. Reject batches.
  if (tx.transactions && (!Array.isArray(tx.transactions) || tx.transactions.length !== 1)) throw new Error('Import one transaction at a time; batches are not supported yet.');
  const item = tx.transactions?.[0] ?? tx;
  const wei = String(item.value ?? '0');
  if (!/^\d+$/.test(wei)) throw new Error('JSON value must be an integer string in wei.');
  const value = BigInt(wei);
  const eth = `${value / 10n ** 18n}.${(value % 10n ** 18n).toString().padStart(18, '0')}`;
  return { from: String(item.from ?? tx.safe ?? tx.meta?.createdFromSafeAddress ?? ''), to: String(item.to ?? ''), value: eth, data: String(item.data ?? '0x'), block: String(tx.blockNumber ?? '') };
}
