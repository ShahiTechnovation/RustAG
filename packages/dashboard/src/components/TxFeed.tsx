"use client";

import { Check, X } from "lucide-react";

import { useTransactions } from "@/lib/hooks";
import { cn } from "@/lib/cn";
import { Card, CopyText, shortKey } from "./ui";

function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const s = Math.round(diff / 1000);
  if (s < 60) return `${s}s ago`;
  const m = Math.round(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.round(m / 60);
  if (h < 24) return `${h}h ago`;
  return new Date(iso).toLocaleDateString();
}

export function TxFeed({ limit }: { limit?: number }) {
  const { data, isLoading, isError } = useTransactions(limit ?? 50);

  if (isLoading) return <Card className="text-sm text-muted">Loading transactions…</Card>;
  if (isError) return <Card className="text-sm text-muted">Could not reach the stagenet API.</Card>;

  const txs = data ?? [];
  if (txs.length === 0) return <Card className="text-sm text-muted">No transactions yet.</Card>;

  const maxCu = Math.max(1, ...txs.map((t) => t.computeUnits ?? 0));

  return (
    <div className="overflow-hidden rounded-card border border-border bg-surface">
      <ul className="divide-y divide-border">
        {txs.map((tx) => (
          <li
            key={tx.signature}
            className="relative flex items-center gap-4 px-5 py-3 text-sm transition-colors hover:bg-white/[0.02]"
          >
            <span
              className={cn(
                "absolute inset-y-0 left-0 w-0.5",
                tx.success ? "bg-accent/70" : "bg-red-500/70",
              )}
            />
            <span
              className={cn(
                "grid size-6 shrink-0 place-items-center rounded-[3px] border",
                tx.success
                  ? "border-accent/40 bg-accent/10 text-accent"
                  : "border-red-500/30 bg-red-500/10 text-red-400",
              )}
            >
              {tx.success ? <Check size={13} /> : <X size={13} />}
            </span>
            <CopyText
              value={tx.signature}
              display={shortKey(tx.signature, 6, 6)}
              className="text-fg"
            />
            <span className="hidden flex-1 truncate text-faint sm:block">
              {tx.programs.length > 0 ? tx.programs.map((p) => shortKey(p)).join(", ") : "-"}
            </span>
            <span className="ml-auto flex items-center gap-2">
              <span className="hidden h-1 w-14 overflow-hidden rounded-[1px] bg-white/5 sm:block">
                <span
                  className="block h-full rounded-[1px] bg-brand/60"
                  style={{ width: `${Math.min(100, ((tx.computeUnits ?? 0) / maxCu) * 100)}%` }}
                />
              </span>
              <span className="tabular-nums text-muted">{(tx.computeUnits ?? 0).toLocaleString()} CU</span>
            </span>
            <span className="hidden w-16 text-right text-xs text-faint md:block">
              {relativeTime(tx.createdAt)}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
