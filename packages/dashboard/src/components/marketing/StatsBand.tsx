"use client";

import { AnimatedNumber, Reveal, RingField } from "@/components/ui";

export function StatsBand() {
  return (
    <section className="px-6 py-12">
      <div className="mx-auto max-w-6xl">
        <Reveal>
          <div className="relative overflow-hidden rounded-card border border-border bg-surface px-2 py-12 sm:px-6">
            <RingField className="opacity-40" />
            <div className="grid grid-cols-2 divide-x divide-y divide-border lg:grid-cols-4 lg:divide-y-0">
              <Stat value={<AnimatedNumber value={0} />} label="Mainnet SOL spent" />
              <Stat value="∞" label="Free airdrops" />
              <Stat value="<1s" label="Oracle refresh" />
              <Stat value={<AnimatedNumber value={100} suffix="%" />} label="Mainnet-accurate state" />
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
