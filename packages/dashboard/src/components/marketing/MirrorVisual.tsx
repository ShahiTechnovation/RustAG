"use client";

import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { Globe } from "lucide-react";

import type { SyncState } from "@rustag/sdk";

import { AnimatedNumber, StatePill } from "@/components/ui";

const STATES: SyncState[] = ["Unknown", "Clean", "Dirty", "Pinned"];

const CARDS = [
  { label: "Pyth · SOL/USD", pubkey: "H6AR…jcW9", base: 147.82, step: 0.42 },
  { label: "Your wallet", pubkey: "9xQe…F4kP", base: 1000, step: 12 },
  { label: "Raydium pool", pubkey: "58oQ…b3Rt", base: 84120, step: 240 },
];

function AccountCard({ index, tick }: { index: number; tick: number }) {
  const card = CARDS[index];
  const state = STATES[(tick + index) % STATES.length];
  const balance = card.base + (tick % 4) * card.step;

  return (
    <motion.div
      className="flex items-center justify-between gap-4 rounded-[3px] border border-border bg-subtle px-4 py-3"
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: 0.2 + index * 0.12, duration: 0.6 }}
    >
      <div className="min-w-0">
        <div className="truncate text-xs font-medium text-fg">{card.label}</div>
        <div className="font-mono text-[11px] text-faint">{card.pubkey}</div>
      </div>
      <div className="flex items-center gap-3">
        <span className="font-mono text-sm text-fg tabular-nums">
          <AnimatedNumber value={balance} format={{ maximumFractionDigits: 2 }} />
        </span>
        <StatePill state={state} />
      </div>
    </motion.div>
  );
}

function Endpoint({ icon, label }: { icon: React.ReactNode; label: string }) {
  return (
    <div className="flex items-center gap-2">
      <span className="grid size-8 place-items-center rounded-[4px] border border-border bg-subtle text-brand">
        {icon}
      </span>
      <span className="label !text-muted">{label}</span>
    </div>
  );
}

export function MirrorVisual() {
  const [tick, setTick] = useState(0);

  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 1900);
    return () => clearInterval(id);
  }, []);

  return (
    <motion.div
      className="relative w-full max-w-md"
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.8, ease: [0.22, 1, 0.36, 1] }}
    >
      <div className="rounded-[4px] border border-border-strong bg-surface p-5">
        {/* endpoints + sync line */}
        <div className="mb-4 flex items-center justify-between">
          <Endpoint icon={<Globe size={15} />} label="mainnet" />
          <svg className="mx-2 h-4 flex-1" viewBox="0 0 100 8" preserveAspectRatio="none">
            <line
              x1="0"
              y1="4"
              x2="100"
              y2="4"
              stroke="var(--brand)"
              strokeOpacity="0.7"
              strokeWidth="1.5"
              strokeDasharray="4 4"
              className="animate-dash"
            />
          </svg>
          <Endpoint
            icon={<span className="font-mono text-xs font-bold text-brand">R</span>}
            label="stagenet"
          />
        </div>

        <div className="space-y-2.5">
          {CARDS.map((_, i) => (
            <AccountCard key={i} index={i} tick={tick} />
          ))}
        </div>

        <div className="mt-4 flex items-center justify-between border-t border-border pt-3 text-[11px] text-faint">
          <span className="label !text-faint">Lazy-mirrored from mainnet</span>
          <span className="inline-flex items-center gap-1.5 font-mono uppercase tracking-wider text-faint">
            <span className="size-1.5 animate-pulse rounded-[1px] bg-brand" />
            syncing
          </span>
        </div>
      </div>
    </motion.div>
  );
}
