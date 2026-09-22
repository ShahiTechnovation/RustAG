import { chainFor, type Network } from "../chains/robinhood";

export interface BrowserWallet {
  request(args: { method: string; params?: unknown[] }): Promise<unknown>;
  on?(event: string, listener: (...args: unknown[]) => void): void;
  removeListener?(event: string, listener: (...args: unknown[]) => void): void;
}

declare global { interface Window { ethereum?: BrowserWallet } }

export function walletAddress(value: unknown): string | undefined {
  const address = Array.isArray(value) ? value[0] : undefined;
  return typeof address === "string" && /^0x[0-9a-fA-F]{40}$/.test(address) ? address : undefined;
}

export async function switchNetwork(wallet: BrowserWallet, network: Network): Promise<void> {
  const chain = chainFor(network);
  const chainId = chain.hex;
  try {
    await wallet.request({ method: "wallet_switchEthereumChain", params: [{ chainId }] });
  } catch (err) {
    if ((err as { code?: number }).code !== 4902) throw err;
    await wallet.request({
      method: "wallet_addEthereumChain",
      params: [{ chainId, chainName: chain.name, nativeCurrency: { name: chain.currency, symbol: chain.currency, decimals: 18 }, rpcUrls: [chain.rpcUrl], blockExplorerUrls: [chain.explorerUrl] }],
    });
    await wallet.request({ method: "wallet_switchEthereumChain", params: [{ chainId }] });
  }
}
