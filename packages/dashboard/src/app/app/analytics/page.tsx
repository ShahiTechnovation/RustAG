"use client";

import type { Format } from "@number-flow/react";

import { MetricChart } from "@/components/MetricChart";
import { AnimatedNumber, Card, GlowCard } from "@/components/ui";
import { useMetrics } from "@/lib/hooks";

type Series = {
  key: string;
  label: string;
  color: string;
  scale: (v: number) => number;
  format: (v: number) => string;
  numberFormat: Format;
};

const SERIES: Series[] = [
  {
    key: "transactions",
    label: "Transactions",
    color: "#5bc8ff",
    scale: (v) => v,
    format: (v) => v.toFixed(0),
    numberFormat: { maximumFractionDigits: 0 },
  },
  {
    key: "accounts",
    label: "Accounts mirrored",
    color: "#dcdcd2",
    scale: (v) => v,
    format: (v) => v.toFixed(0),
    numberFormat: { maximumFractionDigits: 0 },
  },
  {
    key: "dirty_accounts",
    label: "Dirty accounts",
    color: "#ffb020",
    scale: (v) => v,
    format: (v) => v.toFixed(0),
    numberFormat: { maximumFractionDigits: 0 },
  },
];

const TVL = {
  key: "tvl_lamports",
  label: "Total value locked",
  color: "#c5f54b",
  format: (v: number) => `${(v / 1e9).toFixed(3)} SOL`,
};

export default function AnalyticsPage() {
  const { data, isLoading, isError } = useMetrics(120);

  const tvlPoints = (data?.[TVL.key] ?? []).map((p) => ({ t: p.t, v: p.v / 1e9 }));
  const tvlLatest = tvlPoints.length ? tvlPoints[tvlPoints.length - 1].v : 0;

  return (
    <div className="space-y-8">
      <div>
        <h1 className="font-display text-3xl font-bold tracking-tight text-fg">Analytics</h1>
        <p className="mt-1 text-sm text-muted">
          Time-series captured by the analytics sampler - TVL, transaction volume, and mirror growth.
        </p>
      </div>

      {isError ? (
        <Card className="text-sm text-muted">Could not reach the stagenet REST API.</Card>
      ) : (
        <>
          {/* TVL hero chart */}
          <GlowCard className="p-6">
            <div className="mb-4 flex items-baseline justify-between">
              <div>
                <div className="label">{TVL.label}</div>
                <div className="mt-1 font-display text-3xl font-bold tracking-tight text-fg tabular-nums">
                  {isLoading ? (
                    "…"
                  ) : (
                    <AnimatedNumber value={tvlLatest} format={{ maximumFractionDigits: 3 }} suffix=" SOL" />
                  )}
                </div>
              </div>
              <span className="label">{tvlPoints.length} samples</span>
            </div>
            <MetricChart points={tvlPoints} color={TVL.color} format={(v) => `${v.toFixed(2)}`} height={260} />
          </GlowCard>

          {/* Series grid */}
          <div className="grid gap-4 lg:grid-cols-3">
            {SERIES.map((s) => {
              const points = data?.[s.key] ?? [];
              const latest = points.length ? points[points.length - 1].v : 0;
              return (
                <GlowCard key={s.key} className="p-5">
                  <div className="mb-3 flex items-baseline justify-between">
                    <div className="label">{s.label}</div>
                    <div className="font-display text-lg font-bold text-fg tabular-nums">
                      {isLoading ? "…" : <AnimatedNumber value={s.scale(latest)} format={s.numberFormat} />}
                    </div>
                  </div>
                  <MetricChart points={points} color={s.color} format={s.format} height={120} minimal />
                  <div className="mt-2 label">{points.length} samples</div>
                </GlowCard>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
