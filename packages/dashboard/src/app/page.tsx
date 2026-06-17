"use client";

import { ActionsPanel } from "@/components/ActionsPanel";
import { TxFeed } from "@/components/TxFeed";
import { Card, StatCard } from "@/components/ui";
import { useStagenet } from "@/lib/hooks";

export default function OverviewPage() {
  const { data, isLoading, isError } = useStagenet();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-semibold text-zinc-50">Overview</h1>
        <p className="mt-1 text-sm text-zinc-500">
          A persistent, mainnet-mirroring staging environment for Solana programs.
        </p>
      </div>

      {isError ? (
        <Card>
          Could not reach the stagenet REST API. Start one with{" "}
          <code className="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200">rustag start</code>.
        </Card>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard label="Stagenet" value={isLoading ? "…" : (data?.name ?? "—")} hint={data?.network} />
          <StatCard label="Accounts" value={isLoading ? "…" : (data?.accounts ?? 0)} hint={`${data?.dirtyAccounts ?? 0} dirty`} />
          <StatCard label="Transactions" value={isLoading ? "…" : (data?.transactions ?? 0)} />
          <StatCard
            label="Mirror"
            value={data?.mirrorEnabled ? "On" : "Off"}
            hint={data?.mirrorEnabled ? "mainnet on demand" : "offline"}
          />
        </div>
      )}

      <section className="space-y-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500">Actions</h2>
        <ActionsPanel />
      </section>

      <section className="space-y-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500">Recent transactions</h2>
        <TxFeed limit={10} />
      </section>
    </div>
  );
}
