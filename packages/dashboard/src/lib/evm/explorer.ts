import { chainFor, type Network } from '../chains/robinhood';
export function explorerUrl(network: Network, type: 'address' | 'tx' | 'block', value: string | number | bigint) {
  return `${chainFor(network).blockExplorers.default.url.replace(/\/$/, '')}/${type}/${encodeURIComponent(String(value))}`;
}
export const shorten = (value: string) => `${value.slice(0, 6)}…${value.slice(-4)}`;
