"use client";

import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Check, FlaskConical, Play, X } from "lucide-react";

import type { ScenarioReport } from "@rustag/sdk";

import { Button, Card, Field, GlowCard, Input, StatCard, shortKey } from "@/components/ui";
import { client } from "@/lib/client";
import { cn } from "@/lib/cn";

export default function SimulationsPage() {
  const [label, setLabel] = useState("stress-scenario");
  const [raw, setRaw] = useState("");

  const sim = useMutation<ScenarioReport, Error, { transactions: string[]; label: string }>({
    mutationFn: (vars) => client.simulate(vars.transactions, { label: vars.label }),
  });

  const transactions = raw
    .split(/\s+/)
    .map((t) => t.trim())
    .filter(Boolean);

  const report = sim.data;

  return (
    <div className="space-y-8">
      <div>
        <h1 className="font-display text-3xl font-bold tracking-tight text-fg">Simulations</h1>
        <p className="mt-1 text-sm text-muted">
          Replay signed transactions against an isolated fork of the stagenet. The base stagenet is
          never mutated - fork, stress, and compare freely.
        </p>
      </div>

      <GlowCard className="space-y-4 p-6">
        <Field label="Scenario label">
          <Input value={label} onChange={(e) => setLabel(e.target.value)} />
        </Field>
        <Field label="Signed transactions (base64, one per line)" hint={`${transactions.length} parsed`}>
          <textarea
            value={raw}
            onChange={(e) => setRaw(e.target.value)}
            rows={6}
            spellCheck={false}
            placeholder="AQAB... &#10;AgAC..."
            className="w-full rounded-[3px] border border-border-strong bg-subtle px-3 py-2 font-mono text-xs text-fg placeholder:text-faint transition-colors focus:border-brand focus:outline-none focus:ring-2 focus:ring-[var(--ring)]/30"
          />
        </Field>
        <Button
          onClick={() => sim.mutate({ transactions, label })}
          disabled={transactions.length === 0 || sim.isPending}
        >
          <Play size={16} />
          {sim.isPending ? "Running scenario…" : "Run scenario"}
        </Button>
        {sim.isError ? <p className="text-xs text-red-400">{sim.error.message}</p> : null}
      </GlowCard>

      {report ? (
        <ReportView report={report} />
      ) : (
        <Card className="flex items-center gap-3 text-sm text-muted">
          <FlaskConical size={18} className="text-brand" />
          Paste signed transactions above and run a scenario to see per-transaction outcomes and
          aggregate compute, fees, and timing.
        </Card>
      )}
    </div>
  );
}

function ReportView({ report }: { report: ScenarioReport }) {
  const passRate = report.total > 0 ? Math.round((report.succeeded / report.total) * 100) : 0;
  return (
    <div className="space-y-4">
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard label="Succeeded" value={report.succeeded} hint={`${passRate}% pass rate`} accent="var(--accent)" />
        <StatCard label="Failed" value={report.failed} accent="#f87171" />
        <StatCard label="Total CU" value={report.totalComputeUnits} hint={`${report.maxComputeUnits.toLocaleString()} max`} accent="var(--brand)" />
        <StatCard label="Duration" value={report.durationMs} hint="ms" accent="var(--accent-2)" />
      </div>

      <div className="overflow-hidden rounded-card border border-border bg-surface">
        <div className="border-b border-border px-5 py-3 text-sm">
          <span className="font-medium text-fg">{report.label}</span>
          <span className="ml-2 text-faint">· {report.total} transactions replayed</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-border">
              <tr>
                <th className="label px-5 py-2.5">#</th>
                <th className="label px-5 py-2.5">Signature</th>
                <th className="label px-5 py-2.5">Status</th>
                <th className="label px-5 py-2.5 text-right">CU</th>
                <th className="label px-5 py-2.5 text-right">Fee</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {report.outcomes.map((o) => (
                <tr key={o.index} className="hover:bg-white/[0.02]">
                  <td className="px-5 py-2.5 font-mono text-faint">{o.index}</td>
                  <td className="px-5 py-2.5 font-mono text-muted" title={o.signature}>
                    {shortKey(o.signature, 6, 6)}
                  </td>
                  <td className="px-5 py-2.5">
                    <span
                      className={cn(
                        "inline-flex items-center gap-1.5 rounded-[3px] border px-2 py-0.5 font-mono text-[11px] uppercase tracking-wider",
                        o.success
                          ? "border-brand/30 bg-brand/10 text-brand"
                          : "border-red-500/30 bg-red-500/10 text-red-400",
                      )}
                    >
                      {o.success ? <Check size={12} /> : <X size={12} />}
                      {o.success ? "ok" : "failed"}
                    </span>
                  </td>
                  <td className="px-5 py-2.5 text-right tabular-nums text-muted">
                    {o.computeUnits.toLocaleString()}
                  </td>
                  <td className="px-5 py-2.5 text-right tabular-nums text-muted">{o.fee.toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
