import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair } from "@solana/web3.js";

import { EphemeralStagenet, type EphemeralStagenetOptions } from "./stagenet";

export interface RustagAnchorOptions extends EphemeralStagenetOptions {
  /** SOL to airdrop to the generated wallet (default 100). */
  airdropSol?: number;
  /** Commitment level for the connection (default `confirmed`). */
  commitment?: "processed" | "confirmed" | "finalized";
}

export interface RustagAnchorContext {
  /** Anchor provider wired to the stagenet with a funded wallet. */
  provider: AnchorProvider;
  /** The underlying ephemeral stagenet (call `.stop()` when done). */
  stagenet: EphemeralStagenet;
  /** The generated, funded wallet keypair. */
  wallet: Keypair;
}

/**
 * Boot an ephemeral, mainnet-mirroring stagenet and return an Anchor
 * `AnchorProvider` wired to it with a freshly funded wallet.
 *
 * ```ts
 * import { rustagAnchorProvider } from "@rustag/anchor-plugin";
 * import { Program, setProvider } from "@coral-xyz/anchor";
 *
 * const { provider, stagenet } = await rustagAnchorProvider({ preload: ["pyth"] });
 * setProvider(provider);
 * // ... run your Anchor program against real mainnet state ...
 * await stagenet.stop();
 * ```
 */
export async function rustagAnchorProvider(
  opts: RustagAnchorOptions = {},
): Promise<RustagAnchorContext> {
  const stagenet = await EphemeralStagenet.start(opts);
  const wallet = Keypair.generate();
  await stagenet.client.airdrop(wallet.publicKey.toBase58(), opts.airdropSol ?? 100);

  const commitment = opts.commitment ?? "confirmed";
  const connection = new Connection(stagenet.rpcUrl, commitment);
  const provider = new AnchorProvider(connection, new Wallet(wallet), { commitment });

  return { provider, stagenet, wallet };
}
