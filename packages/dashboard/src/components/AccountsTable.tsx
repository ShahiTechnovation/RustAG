"use client";

import { useMemo, useState } from "react";
import { Search } from "lucide-react";

import type { SyncState } from "@rustag/sdk";

import { useAccounts } from "@/lib/hooks";
import { cn } from "@/lib/cn";
import { Card, CategoryBadge, CopyText, StatePill, shortKey } from "./ui";

const FILTERS: ("All" | SyncState)[] = ["All", "Clean", "Dirty", "Pinned", "Unknown"];

export function AccountsTable() {
  const { data, isLoading, isError } = useAccounts();
  const [filter, setFilter] = useState<"All" | SyncState>("All");
  const [query, setQuery] = useState("");

  const accounts = useMemo(() => data ?? [], [data]);
  const maxSol = useMemo(() => Math.max(1, ...accounts.map((a) => a.sol)), [accounts]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return accounts.filter((a) => {
      if (filter !== "All" && a.syncState !== filter) return false;
      if (q && !a.pubkey.toLowerCase().includes(q) && !a.owner.toLowerCase().includes(q)) return false;
      return true;
    });
  }, [accounts, filter, query]);

  if (isLoading) return <Card className="text-sm text-muted">Loading accounts…</Card>;
  if (isError) return <Card className="text-sm text-muted">Could not reach the stagenet API.</Card>;
  if (accounts.length === 0) {
    return (
      <Card className="text-sm text-muted">
        No accounts yet. Preload mainnet programs or airdrop to a wallet.
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {/* Toolbar */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex flex-wrap gap-1.5">
          {FILTERS.map((f) => {
            const count = f === "All" ? accounts.length : accounts.filter((a) => a.syncState === f).length;
            return (
              <button
                key={f}
                onClick={() => setFilter(f)}
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-[3px] border px-2.5 py-1 font-mono text-[11px] uppercase tracking-wider transition-colors cursor-pointer",
                  filter === f
                    ? "border-brand bg-brand/10 text-brand"
                    : "border-border text-muted hover:border-border-strong hover:text-fg",
                )}
              >
                {f}
                <span className="text-faint">{count}</span>
              </button>
            );
          })}
        </div>
        <div className="relative sm:w-64">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-faint" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search pubkey or owner…"
            className="h-9 w-full rounded-[3px] border border-border bg-subtle pl-9 pr-3 text-sm text-fg placeholder:text-faint focus:border-brand focus:outline-none"
          />
        </div>
      </div>

      <div className="overflow-hidden rounded-card border border-border bg-surface">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="sticky top-0 z-10 border-b border-border bg-surface/90 backdrop-blur">
              <tr>
                <th className="label px-5 py-3 text-left font-normal">Pubkey</th>
                <th className="label px-5 py-3 text-left font-normal">Owner</th>
                <th className="label px-5 py-3 text-right font-normal">SOL</th>
                <th className="label px-5 py-3 text-right font-normal">Data</th>
                <th className="label px-5 py-3 text-left font-normal">Category</th>
                <th className="label px-5 py-3 text-left font-normal">Sync</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {filtered.map((a) => (
                <tr key={a.pubkey} className="group transition-colors hover:bg-white/[0.02]">
                  <td className="px-5 py-3">
                    <CopyText value={a.pubkey} display={shortKey(a.pubkey, 6, 6)} className="text-fg" />
                  </td>
                  <td className="px-5 py-3">
                    <CopyText value={a.owner} display={shortKey(a.owner, 4, 4)} className="text-muted" />
                  </td>
                  <td className="px-5 py-3 text-right">
                    <div className="flex flex-col items-end gap-1">
                      <span className="tabular-nums text-fg">
                        {a.sol.toLocaleString(undefined, { maximumFractionDigits: 4 })}
                      </span>
                      <span className="h-1 w-20 overflow-hidden rounded-[1px] bg-white/5">
                        <span
                          className="block h-full rounded-[1px] bg-[image:var(--gradient-brand)]"
                          style={{ width: `${Math.min(100, (a.sol / maxSol) * 100)}%` }}
                        />
                      </span>
                    </div>
                  </td>
                  <td className="px-5 py-3 text-right tabular-nums text-muted">{a.dataLen} B</td>
                  <td className="px-5 py-3">
                    <CategoryBadge category={a.category} />
                  </td>
                  <td className="px-5 py-3">
                    <StatePill state={a.syncState} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {filtered.length === 0 ? (
          <div className="px-5 py-8 text-center text-sm text-faint">No accounts match this filter.</div>
        ) : null}
      </div>
    </div>
  );
}
