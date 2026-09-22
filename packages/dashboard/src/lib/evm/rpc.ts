import { chainFor, type Network } from "../chains/robinhood";

async function jsonRpc(
  url: string,
  method: string,
  params: unknown[] = [],
): Promise<unknown> {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
    signal: AbortSignal.timeout(8000),
  });
  if (!response.ok) throw new Error("RPC HTTP " + response.status);
  const data = (await response.json()) as {
    result?: unknown;
    error?: { message?: string };
  };
  if (data.error) throw new Error(data.error.message ?? "RPC error");
  return data.result;
}

export interface NetworkInfo {
  chainId: number;
  block: string;
}

export async function readNetwork(network: Network): Promise<NetworkInfo> {
  const chain = chainFor(network);
  const [chainIdHex, blockHex] = await Promise.all([
    jsonRpc(chain.rpcUrl, "eth_chainId"),
    jsonRpc(chain.rpcUrl, "eth_blockNumber"),
  ]);
  const chainId = parseInt(String(chainIdHex), 16);
  if (chainId !== chain.id) {
    throw new Error(
      "RPC chain ID " + chainId + " does not match expected " + chain.id + " (" + chain.name + ").",
    );
  }
  return { chainId, block: String(parseInt(String(blockHex), 16)) };
}
