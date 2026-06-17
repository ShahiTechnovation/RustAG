"use client";

import { useAccounts } from "@/lib/hooks";
import { Card, SyncBadge, shortKey } from "./ui";

export function AccountsTable() {
  const { data, isLoading, isError } = useAccounts();

  if (isLoading) return <Card>Loading accounts…</Card>;
  if (isError) return <Card>Could not reach the stagenet API.</Card>;

  const accounts = data ?? [];
  if (accounts.length === 0) {
    return <Card>No accounts yet. Preload mainnet programs or airdrop to a wallet.</Card>;
  }

  return (
    <Card className="overflow-x-auto p-0">
      <table className="w-full text-left text-sm">
        <thead className="border-b border-zinc-800 text-xs uppercase tracking-wider text-zinc-500">
          <tr>
            <th className="px-5 py-3 font-medium">Pubkey</th>
            <th className="px-5 py-3 font-medium">Owner</th>
            <th className="px-5 py-3 text-right font-medium">SOL</th>
            <th className="px-5 py-3 text-right font-medium">Data</th>
            <th className="px-5 py-3 font-medium">Category</th>
            <th className="px-5 py-3 font-medium">Sync</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-zinc-800/70">
          {accounts.map((a) => (
            <tr key={a.pubkey} className="hover:bg-zinc-800/30">
              <td className="px-5 py-3 font-mono text-zinc-200" title={a.pubkey}>
                {shortKey(a.pubkey, 6, 6)}
              </td>
              <td className="px-5 py-3 font-mono text-zinc-400" title={a.owner}>
                {shortKey(a.owner, 4, 4)}
              </td>
              <td className="px-5 py-3 text-right tabular-nums text-zinc-200">
                {a.sol.toLocaleString(undefined, { maximumFractionDigits: 4 })}
              </td>
              <td className="px-5 py-3 text-right tabular-nums text-zinc-400">{a.dataLen} B</td>
              <td className="px-5 py-3 text-zinc-400">{a.category ?? "—"}</td>
              <td className="px-5 py-3">
                <SyncBadge state={a.syncState} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </Card>
  );
}
