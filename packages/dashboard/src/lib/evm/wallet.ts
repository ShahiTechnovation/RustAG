import type { Address } from 'viem';
import { chainFor, type Network } from '../chains/robinhood';
export interface BrowserWallet {
  request(args: { method: string; params?: unknown[] }): Promise<unknown>;
  on?(event: string, listener: (...args: unknown[]) => void): void;
  removeListener?(event: string, listener: (...args: unknown[]) => void): void;
}
declare global { interface Window { ethereum?: BrowserWallet } }
export function walletAddress(value: unknown): Address | undefined {
  const address = Array.isArray(value) ? value[0] : undefined;
  return typeof address === 'string' && /^0x[0-9a-fA-F]{40}$/.test(address) ? address as Address : undefined;
}
export async function switchNetwork(wallet: BrowserWallet, network: Network) {
  const chain = chainFor(network);
  const chainId = `0x${chain.id.toString(16)}`;
  try { await wallet.request({ method: 'wallet_switchEthereumChain', params: [{ chainId }] }); }
  catch (error) {
    if ((error as { code?: number }).code !== 4902) throw error;
    await wallet.request({ method: 'wallet_addEthereumChain', params: [{ chainId, chainName: chain.name, nativeCurrency: chain.nativeCurrency, rpcUrls: chain.rpcUrls.default.http, blockExplorerUrls: [chain.blockExplorers.default.url] }] });
    await wallet.request({ method: 'wallet_switchEthereumChain', params: [{ chainId }] });
  }
}
