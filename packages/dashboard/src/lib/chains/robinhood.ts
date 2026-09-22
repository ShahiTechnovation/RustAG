import { defineChain } from 'viem';

// Verified against https://docs.robinhood.com/chain/connecting/ on 2026-09-22.
export const robinhoodMainnet = defineChain({ id: 4663, name: 'Robinhood Chain', nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 }, rpcUrls: { default: { http: ['https://rpc.mainnet.chain.robinhood.com'] } }, blockExplorers: { default: { name: 'Blockscout', url: 'https://robinhoodchain.blockscout.com' } } });
export const robinhoodTestnet = defineChain({ id: 46630, name: 'Robinhood Chain Testnet', nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 }, rpcUrls: { default: { http: ['https://rpc.testnet.chain.robinhood.com'] } }, blockExplorers: { default: { name: 'Robinhood Explorer', url: 'https://explorer.testnet.chain.robinhood.com' } }, testnet: true });
export type Network = 'mainnet' | 'testnet';
export const defaultNetwork: Network = process.env.NEXT_PUBLIC_ROBINHOOD_NETWORK === 'mainnet' ? 'mainnet' : 'testnet';
export function chainFor(network: Network) {
  const chain = network === 'mainnet' ? robinhoodMainnet : robinhoodTestnet;
  // Generic overrides apply ONLY to the configured default network.
  const rpc = network === 'mainnet' ? process.env.NEXT_PUBLIC_ROBINHOOD_MAINNET_RPC_URL : process.env.NEXT_PUBLIC_ROBINHOOD_TESTNET_RPC_URL;
  const explorer = network === 'mainnet' ? process.env.NEXT_PUBLIC_ROBINHOOD_MAINNET_EXPLORER_URL : process.env.NEXT_PUBLIC_ROBINHOOD_TESTNET_EXPLORER_URL;
  return { ...chain, rpcUrls: { default: { http: [rpc || (network === defaultNetwork && process.env.NEXT_PUBLIC_ROBINHOOD_RPC_URL) || chain.rpcUrls.default.http[0]] } }, blockExplorers: { default: { ...chain.blockExplorers.default, url: explorer || (network === defaultNetwork && process.env.NEXT_PUBLIC_ROBINHOOD_EXPLORER_URL) || chain.blockExplorers.default.url } } };
}
