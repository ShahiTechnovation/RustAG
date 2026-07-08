"use client";

import { useOraclePrices } from "@/lib/hooks";
import { PYTH_FEEDS } from "@/lib/pyth";
import { cn } from "@/lib/cn";

function fmtPrice(p: number | null): string {
  if (p == null) return "—";
  const digits = p < 10 ? 4 : 2;
  return `$${p.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: digits })}`;
}

function LiveDot({ live }: { live: boolean }) {
  return (
    <span className="relative flex size-1.5">
      {live ? (
        <span className="absolute inline-flex size-full animate-ping rounded-full bg-brand opacity-60" />
      ) : null}
      <span className={cn("relative inline-flex size-1.5 rounded-full", live ? "bg-brand" : "bg-faint")} />
    </span>
  );
}

/**
 * Real mainnet Pyth prices, decoded in the browser from the mirrored account
 * bytes. `compact` renders a one-line strip (for the landing); the default is a
 * titled panel (for the app).
 */
export function OraclePrices({ compact = false }: { compact?: boolean }) {
  const { data, isError } = useOraclePrices();
  const rows = data ?? PYTH_FEEDS.map((f) => ({ symbol: f.symbol, price: null, syncState: null }));
  const live = !!data && !isError && data.some((d) => d.price != null);

  if (compact) {
    return (
      <div className="flex flex-wrap items-center justify-center gap-x-6 gap-y-2 text-sm">
        <span className="label inline-flex items-center gap-1.5 text-brand">
          <LiveDot live={live} />
          Live Pyth
        </span>
        {rows.map((d) => (
          <span key={d.symbol} className="font-mono">
            <span className="text-faint">{d.symbol}</span>{" "}
            <span className="tabular-nums text-fg">{fmtPrice(d.price)}</span>
          </span>
        ))}
      </div>
    );
  }

  return (
    <div className="rounded-card border border-border bg-surface p-5">
      <div className="mb-4 flex items-center justify-between">
        <h3 className="font-display text-sm font-semibold uppercase tracking-wide text-fg">
          Live oracle prices
        </h3>
        <span
          className={cn(
            "inline-flex items-center gap-1.5 rounded-[3px] border px-2 py-0.5 font-mono text-[10px] uppercase tracking-wider",
            live ? "border-brand/40 bg-brand/10 text-brand" : "border-border-strong text-faint",
          )}
        >
          <LiveDot live={live} />
          Pyth
        </span>
      </div>
      <div className="grid grid-cols-3 gap-3">
        {rows.map((d) => (
          <div key={d.symbol} className="rounded-[3px] border border-border bg-subtle px-3 py-3 text-center">
            <div className="label justify-center text-faint">{d.symbol}</div>
            <div className="mt-1 font-mono text-lg font-semibold tabular-nums text-fg">
              {fmtPrice(d.price)}
            </div>
          </div>
        ))}
      </div>
      <p className="mt-3 text-xs text-faint">
        Decoded in your browser from real mainnet Pyth accounts — cross-check against any price
        site. Auto-refreshed ~30s.
      </p>
    </div>
  );
}
