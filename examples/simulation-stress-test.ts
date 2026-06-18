/**
 * Simulation stress test (Phase 2).
 *
 * Funds a payer, builds N signed transfers, and replays them against an isolated
 * *fork* of the stagenet via `POST /api/simulate`. The base stagenet is never
 * mutated — this answers "what happens if N users all act at once?". Run a
 * stagenet first (see README).
 */
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";

const RPC_URL = process.env.RUSTAG_RPC_URL ?? "http://127.0.0.1:8899";
const API_URL = process.env.RUSTAG_API_URL ?? "http://127.0.0.1:9000";

const ACTORS = 25;
const TRANSFER_SOL = 0.5;
const FUNDING_SOL = 5; // only ~10 transfers can succeed → a real success/fail split

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");
  const payer = Keypair.generate();

  // Fund the payer through the unlimited, instant REST faucet.
  await fetch(`${API_URL}/api/airdrop`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ pubkey: payer.publicKey.toBase58(), sol: FUNDING_SOL }),
  });

  const { blockhash } = await connection.getLatestBlockhash();
  const transactions: string[] = [];
  for (let i = 0; i < ACTORS; i++) {
    const tx = new Transaction();
    tx.feePayer = payer.publicKey;
    tx.recentBlockhash = blockhash;
    tx.add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: Keypair.generate().publicKey,
        lamports: Math.floor(TRANSFER_SOL * LAMPORTS_PER_SOL),
      }),
    );
    tx.sign(payer);
    transactions.push(tx.serialize().toString("base64"));
  }

  console.log(`Replaying ${ACTORS} transfers against a fork of the stagenet...`);
  const res = await fetch(`${API_URL}/api/simulate`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ label: "transfer-herd", transactions, encoding: "base64" }),
  });
  if (!res.ok) {
    throw new Error(`simulate failed: ${res.status} ${await res.text()}`);
  }
  const report = await res.json();

  console.log(`\n  Scenario "${report.label}"`);
  console.log(`  ${report.succeeded}/${report.total} succeeded (${report.failed} failed)`);
  console.log(`  total compute units: ${report.totalComputeUnits}`);
  console.log(`  wall-clock:          ${report.durationMs}ms\n`);
  console.log("✓ Ran against an isolated fork — the base stagenet is untouched.");
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("✗", err.message);
    process.exit(1);
  });
