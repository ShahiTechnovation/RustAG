"use client";

import { AnimatedNumber, Reveal, RingField } from "@/components/ui";
import { useStagenet } from "@/lib/hooks";
import { cn } from "@/lib/cn";

export function StatsBand() {
  const { data, isError } = useStagenet();
  const live = !!data && !isError;

  return (
    <section className="px-6 py-12">
      <div className="mx-auto max-w-6xl">
        <Reveal>
          <div className="relative overflow-hidden rounded-card border border-border bg-surface px-2 py-12 sm:px-6">
            <RingField className="opacity-40" />

            {/* Live/offline badge tied to the real demo backend. */}
            <div className="mb-8 flex justify-center">
              <span
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-[3px] border px-2.5 py-1 font-mono text-[11px] uppercase tracking-wider",
                  live
                    ? "border-brand/40 bg-brand/10 text-brand"
                    : "border-border-strong bg-white/[0.03] text-faint",
                )}
              >
                <span className="relative flex size-1.5">
                  {live ? (
                    <span className="absolute inline-flex size-full animate-ping rounded-full bg-brand opacity-60" />
                  ) : null}
                  <span
                    className={cn(
                      "relative inline-flex size-1.5 rounded-full",
                      live ? "bg-brand" : "bg-faint",
                    )}
                  />
                </span>
                {live ? `Assurance live · slot ${data.slot.toLocaleString()}` : "Live demo"}
              </span>
            </div>

            <div className="grid grid-cols-2 divide-x divide-y divide-border lg:grid-cols-4 lg:divide-y-0">
              <Stat
                value={live ? <AnimatedNumber value={data.accounts} /> : "—"}
                label="Accounts in pre-state closure"
              />
              <Stat
                value={live ? <AnimatedNumber value={data.transactions} /> : "—"}
                label="Rehearsals executed"
              />
              <Stat value="$0" label="Mainnet SOL spent" />
              <Stat value="Grade A" label="Fidelity standard" />
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  );
}

function Stat({ value, label }: { value: React.ReactNode; label: string }) {
  return (
    <div className="px-4 py-4 text-center">
      <div className="font-display text-4xl font-bold tracking-tight text-fg tabular-nums sm:text-5xl">
        {value}
      </div>
      <div className="label mt-3 justify-center text-muted">{label}</div>
    </div>
  );
}
