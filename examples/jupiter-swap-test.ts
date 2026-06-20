/**
 * Jupiter swap test.
 *
 * Demonstrates the unlimited-airdrop + preload workflow and a confirmed
 * transaction against the stagenet using a standard web3.js Connection.
 *
 * NOTE: executing a *full* Jupiter swap (invoking Jupiter's on-chain program)
 * requires loading the program's BPF bytecode, which is Phase 2 (see
 * docs/architecture.md#known-limitations). Here we preload Jupiter's program
 * account, airdrop test SOL, and run a confirmed SystemProgram transfer to show
 * the end-to-end RPC path works exactly like a real cluster.
 */
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

const RPC_URL = process.env.RUSTAG_RPC_URL ?? "http://127.0.0.1:8899";
const API_URL = process.env.RUSTAG_API_URL ?? "http://127.0.0.1:9000";

async function rest(path: string, body: unknown) {
  const res = await fetch(`${API_URL}${path}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`${path} -> ${res.status}: ${await res.text()}`);
  return res.json();
}

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");
  console.log(`Connected to RustAG stagenet at ${RPC_URL}`);

  // 1. Preload Jupiter's program account from mainnet.
  const preload = (await rest("/api/preload", { programs: ["jupiter"] })) as { loaded: number };
  console.log(`Preloaded Jupiter: ${preload.loaded} account(s) from mainnet`);

  // 2. Unlimited airdrop - no faucet, no mainnet SOL.
  const payer = Keypair.generate();
  const receiver = Keypair.generate();
  await rest("/api/airdrop", { pubkey: payer.publicKey.toBase58(), sol: 100 });
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`Airdropped 100 SOL → payer balance: ${balance / LAMPORTS_PER_SOL} SOL`);

  // 3. A confirmed transaction over the standard RPC path.
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: receiver.publicKey,
      lamports: 1 * LAMPORTS_PER_SOL,
    }),
  );
  const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
  console.log(`Transfer confirmed: ${sig}`);

  const recvBalance = await connection.getBalance(receiver.publicKey);
  console.log(`Receiver balance: ${recvBalance / LAMPORTS_PER_SOL} SOL`);
  console.log("\n✓ Airdrop + preload + confirmed tx, zero mainnet SOL spent.");
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("✗", err.message);
    process.exit(1);
  });
