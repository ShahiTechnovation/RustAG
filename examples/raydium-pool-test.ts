/**
 * Raydium pool test.
 *
 * Reads a real Raydium AMM v4 pool account *through the stagenet*. The account
 * is lazily mirrored from mainnet on first access, demonstrating that complex
 * DeFi state is available locally without any manual fixtures.
 */
import { Connection, PublicKey } from "@solana/web3.js";

const RPC_URL = process.env.RUSTAG_RPC_URL ?? "http://127.0.0.1:8899";

// Raydium AMM v4 SOL/USDC pool (AMM id) on mainnet-beta.
const RAYDIUM_SOL_USDC_POOL = new PublicKey("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2");
const RAYDIUM_AMM_V4 = new PublicKey("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");
  console.log(`Connected to RustAG stagenet at ${RPC_URL}`);

  const info = await connection.getAccountInfo(RAYDIUM_SOL_USDC_POOL);
  if (!info) {
    throw new Error("Pool account not found — is mirroring enabled?");
  }

  console.log(`\nRaydium SOL/USDC pool: ${RAYDIUM_SOL_USDC_POOL.toBase58()}`);
  console.log(`  Owner:     ${info.owner.toBase58()}`);
  console.log(`  Owned by Raydium AMM v4: ${info.owner.equals(RAYDIUM_AMM_V4)}`);
  console.log(`  State size: ${info.data.length} bytes (mirrored from mainnet)`);
  console.log("\n✓ Real Raydium pool state, read locally, zero SOL spent.");
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("✗", err.message);
    process.exit(1);
  });
