// Robinhood Chain network configuration — single source of truth.
// Verified against official docs on 2025-09-22. No external dependencies.

export type Network = "mainnet" | "testnet";

export interface ChainConfig {
  id: number;
  hex: string;
  name: string;
  rpcUrl: string;
  explorerUrl: string;
  currency: string;
  testnet?: boolean;
}

export const ROBINHOOD_MAINNET: ChainConfig = {
  id: 4663,
  hex: "0x1237",
  name: "Robinhood Chain",
  rpcUrl: "https://rpc.mainnet.chain.robinhood.com",
  explorerUrl: "https://robinhoodchain.blockscout.com",
  currency: "ETH",
};

export const ROBINHOOD_TESTNET: ChainConfig = {
  id: 46630,
  hex: "0xb626",
  name: "Robinhood Chain Testnet",
  rpcUrl: "https://rpc.testnet.chain.robinhood.com",
  explorerUrl: "https://explorer.testnet.chain.robinhood.com",
  currency: "ETH",
  testnet: true,
};

export const defaultNetwork: Network =
  process.env.NEXT_PUBLIC_ROBINHOOD_NETWORK === "testnet" ? "testnet" : "mainnet";

export function chainFor(network: Network): ChainConfig {
  if (network === "mainnet") {
    return {
      ...ROBINHOOD_MAINNET,
      rpcUrl: process.env.NEXT_PUBLIC_ROBINHOOD_RPC_URL || ROBINHOOD_MAINNET.rpcUrl,
      explorerUrl: process.env.NEXT_PUBLIC_ROBINHOOD_EXPLORER_URL || ROBINHOOD_MAINNET.explorerUrl,
    };
  }
  return ROBINHOOD_TESTNET;
}
