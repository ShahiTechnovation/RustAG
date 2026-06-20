"use client";

import { Activity, Boxes, Database, Radio } from "lucide-react";

import { ActionsPanel } from "@/components/ActionsPanel";
import { TxFeed } from "@/components/TxFeed";
import { Card, StatCard } from "@/components/ui";
import { useStagenet } from "@/lib/hooks";

export default function OverviewPage() {
  const { data, isLoading, isError } = useStagenet();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="font-display text-3xl font-bold tracking-tight text-fg">Overview</h1>
        <p className="mt-1 text-sm text-muted">
          A persistent, mainnet-mirroring staging environment for Solana programs.
        </p>
      </div>

      {isError ? (
        <Card className="text-sm text-muted">
          Could not reach the stagenet REST API. Start one with{" "}
          <code className="rounded-[3px] bg-white/5 px-1.5 py-0.5 font-mono text-fg">rustag start</code>.
        </Card>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard
            label="Stagenet"
            value={isLoading ? "…" : (data?.name ?? "-")}
            hint={data?.network}
            icon={<Boxes size={16} />}
          />
          <StatCard
            label="Accounts"
            value={isLoading ? "…" : (data?.accounts ?? 0)}
            hint={`${data?.dirtyAccounts ?? 0} dirty`}
            icon={<Database size={16} />}
            accent="var(--accent)"
          />
          <StatCard
            label="Transactions"
            value={isLoading ? "…" : (data?.transactions ?? 0)}
            icon={<Activity size={16} />}
            accent="var(--accent-2)"
          />
          <StatCard
            label="Mirror"
            value={data?.mirrorEnabled ? "Live" : "Off"}
            hint={data?.mirrorEnabled ? "mainnet on demand" : "offline"}
            icon={<Radio size={16} />}
            accent={data?.mirrorEnabled ? "var(--accent)" : "var(--fg-subtle)"}
          />
        </div>
      )}

      <section id="actions" className="space-y-3">
        <h2 className="label">Actions</h2>
        <ActionsPanel />
      </section>

      <section className="space-y-3">
        <h2 className="label">Recent transactions</h2>
        <TxFeed limit={10} />
      </section>
    </div>
  );
}
