import { createPublicClient, http } from 'viem';
import { chainFor, type Network } from '../chains/robinhood';
export async function readNetwork(network: Network) {
  const chain = chainFor(network);
  const client = createPublicClient({ chain, transport: http(chain.rpcUrls.default.http[0], { timeout: 8000, retryCount: 0 }) });
  const [chainId, block] = await Promise.all([client.getChainId(), client.getBlockNumber()]);
  if (chainId !== chain.id) throw new Error('RPC returned an unexpected chain ID.');
  return { chainId, block: block.toString() };
}
