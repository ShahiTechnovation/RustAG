/**
 * Activity Scheduler demo (Phase 2).
 *
 * Registers a recurring airdrop that tops up a wallet every 2 seconds, waits for
 * a few firings, then inspects the run count and cleans up. Run a stagenet first
 * (see README).
 */
import { Keypair } from "@solana/web3.js";

const API_URL = process.env.RUSTAG_API_URL ?? "http://127.0.0.1:9000";

async function api(path: string, init?: RequestInit): Promise<any> {
  const res = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
  });
  if (!res.ok) throw new Error(`${path} → ${res.status} ${await res.text()}`);
  return res.json();
}

async function main() {
  const wallet = Keypair.generate().publicKey.toBase58();

  console.log("Creating a recurring airdrop activity (every 2s)...");
  const created = await api("/api/schedules", {
    method: "POST",
    body: JSON.stringify({
      name: "faucet-topup",
      schedule: "@every 2s",
      action: { type: "airdrop", pubkey: wallet, sol: 1 },
    }),
  });
  console.log(`  created activity ${created.id}`);

  // Let it fire a few times.
  await new Promise((r) => setTimeout(r, 7000));

  const list = await api("/api/schedules");
  const mine = list.schedules.find((s: any) => s.id === created.id);
  console.log(`\n  runs so far: ${mine?.runCount ?? 0}`);
  console.log(`  last status: ${mine?.lastStatus ?? "—"}`);

  await api(`/api/schedules/${created.id}`, { method: "DELETE" });
  console.log("\n✓ Activity fired on schedule, then was removed.");
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("✗", err.message);
    process.exit(1);
  });
