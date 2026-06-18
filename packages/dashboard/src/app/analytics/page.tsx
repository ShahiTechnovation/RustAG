"use client";

import { Sparkline } from "@/components/Sparkline";
import { Card } from "@/components/ui";
import { useMetrics } from "@/lib/hooks";

const SERIES: { key: string; label: string; format: (v: number) => string; stroke: string }[] = [
  { key: "tvl_lamports", label: "TVL (SOL)", format: (v) => (v / 1e9).toFixed(3), stroke: "#34d399" },
  { key: "transactions", label: "Transactions", format: (v) => v.toFixed(0), stroke: "#818cf8" },
  { key: "accounts", label: "Accounts mirrored", format: (v) => v.toFixed(0), stroke: "#60a5fa" },
  { key: "dirty_accounts", label: "Dirty accounts", format: (v) => v.toFixed(0), stroke: "#fbbf24" },
];

export default function AnalyticsPage() {
  const { data, isLoading, isError } = useMetrics(120);

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-semibold text-zinc-50">Analytics</h1>
        <p className="mt-1 text-sm text-zinc-500">
          Time-series captured by the analytics sampler — TVL, transaction volume, and mirror growth.
        </p>
      </div>

      {isError ? (
        <Card>Could not reach the stagenet REST API.</Card>
      ) : (
        <div className="grid gap-4 lg:grid-cols-2">
          {SERIES.map((s) => {
            const points = data?.[s.key] ?? [];
            const latest = points.length ? points[points.length - 1].v : 0;
            return (
              <Card key={s.key}>
                <div className="mb-3 flex items-baseline justify-between">
                  <div className="text-xs font-medium uppercase tracking-wider text-zinc-500">
                    {s.label}
                  </div>
                  <div className="text-lg font-semibold text-zinc-100">
                    {isLoading ? "…" : s.format(latest)}
                  </div>
                </div>
                <Sparkline points={points} stroke={s.stroke} />
                <div className="mt-2 text-xs text-zinc-600">{points.length} samples</div>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}
