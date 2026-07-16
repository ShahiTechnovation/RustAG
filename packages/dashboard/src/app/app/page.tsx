"use client";

import { Activity, BadgeCheck, Database, ShieldCheck } from "lucide-react";
import Link from "next/link";
import { ArrowUpRight } from "lucide-react";

import { ActionsPanel } from "@/components/ActionsPanel";
import { OraclePrices } from "@/components/OraclePrices";
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
          RustAG GroundTruth — attested pre-execution assurance for Solana privileged operations.
        </p>
      </div>

      {/* GroundTruth quick actions */}
      <div className="grid gap-3 sm:grid-cols-2">
        <Link
          href="/app/rehearse"
          className="group flex items-start gap-3 rounded-[3px] border border-brand/30 bg-brand/5 p-4 transition-colors hover:border-brand/60 hover:bg-brand/10"
        >
          <ShieldCheck size={20} className="mt-0.5 shrink-0 text-brand" />
          <div>
            <p className="font-display text-sm font-semibold text-fg">Rehearse a proposal</p>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              Paste a Squads v4 proposal pubkey or a base64 transaction — get a signed
              EvidenceBundle back.
            </p>
          </div>
          <ArrowUpRight size={15} className="ml-auto shrink-0 text-faint transition-colors group-hover:text-brand" />
        </Link>
        <Link
          href="/app/forensics"
          className="group flex items-start gap-3 rounded-[3px] border border-border bg-surface/40 p-4 transition-colors hover:border-brand/40 hover:bg-brand/5"
        >
          <BadgeCheck size={20} className="mt-0.5 shrink-0 text-brand" />
          <div>
            <p className="font-display text-sm font-semibold text-fg">Forensics</p>
            <p className="mt-1 text-xs text-muted leading-relaxed">
              Re-execute a historical transaction. Use --patch to run counterfactual analysis.
            </p>
          </div>
          <ArrowUpRight size={15} className="ml-auto shrink-0 text-faint transition-colors group-hover:text-brand" />
        </Link>
      </div>

      {isError ? (
        <Card className="text-sm text-muted">
          Could not reach the stagenet REST API. Run the demo with{" "}
          <code className="rounded-[3px] bg-white/5 px-1.5 py-0.5 font-mono text-fg">rustag rehearse --demo</code>
          {" "}or{" "}
          <code className="rounded-[3px] bg-white/5 px-1.5 py-0.5 font-mono text-fg">rustag serve</code>.
        </Card>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard
            label="Stagenet"
            value={isLoading ? "…" : (data?.name ?? "-")}
            hint={data?.network}
            icon={<ShieldCheck size={16} />}
          />
          <StatCard
            label="Accounts"
            value={isLoading ? "…" : (data?.accounts ?? 0)}
            hint={`${data?.dirtyAccounts ?? 0} modified`}
            icon={<Database size={16} />}
            accent="var(--accent)"
          />
          <StatCard
            label="Rehearsals"
            value={isLoading ? "…" : (data?.transactions ?? 0)}
            icon={<Activity size={16} />}
            accent="var(--accent-2)"
          />
          <StatCard
            label="Assurance"
            value={data?.mirrorEnabled ? "Live" : "Off"}
            hint={data?.mirrorEnabled ? "mainnet on demand" : "offline"}
            icon={<BadgeCheck size={16} />}
            accent={data?.mirrorEnabled ? "var(--accent)" : "var(--fg-subtle)"}
          />
        </div>
      )}

      <OraclePrices />

      <section id="actions" className="space-y-3">
        <h2 className="label">Stagenet actions</h2>
        <ActionsPanel />
      </section>

      <section className="space-y-3">
        <h2 className="label">Recent rehearsals</h2>
        <TxFeed limit={10} />
      </section>
    </div>
  );
}
