"use client";

import { useTransactions } from "@/lib/hooks";
import { Badge, Card, shortKey } from "./ui";

export function TxFeed({ limit }: { limit?: number }) {
  const { data, isLoading, isError } = useTransactions(limit ?? 50);

  if (isLoading) return <Card>Loading transactions…</Card>;
  if (isError) return <Card>Could not reach the stagenet API.</Card>;

  const txs = data ?? [];
  if (txs.length === 0) return <Card>No transactions yet.</Card>;

  return (
    <Card className="p-0">
      <ul className="divide-y divide-zinc-800/70">
        {txs.map((tx) => (
          <li key={tx.signature} className="flex items-center gap-4 px-5 py-3 text-sm">
            <Badge tone={tx.success ? "emerald" : "red"}>{tx.success ? "✓" : "✗"}</Badge>
            <span className="font-mono text-zinc-300" title={tx.signature}>
              {shortKey(tx.signature, 6, 6)}
            </span>
            <span className="flex-1 truncate text-zinc-500">
              {tx.programs.length > 0 ? tx.programs.map((p) => shortKey(p)).join(", ") : "—"}
            </span>
            <span className="tabular-nums text-zinc-400">{(tx.computeUnits ?? 0).toLocaleString()} CU</span>
            <span className="text-xs text-zinc-600">{new Date(tx.createdAt).toLocaleTimeString()}</span>
          </li>
        ))}
      </ul>
    </Card>
  );
}
