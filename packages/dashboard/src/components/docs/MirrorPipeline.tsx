"use client";

import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { Globe } from "lucide-react";

import type { SyncState } from "@rustag/sdk";

import { AnimatedNumber, StatePill } from "@/components/ui";
import { LogoMark } from "@/components/LogoMark";
import { cn } from "@/lib/cn";

const ROWS = [
  { label: "Pyth · SOL/USD", pubkey: "H6AR…jcW9", base: 147.82, step: 0.41 },
  { label: "Raydium pool", pubkey: "58oQ…b3Rt", base: 84120, step: 260 },
  { label: "Your wallet", pubkey: "9xQe…F4kP", base: 1000, step: 18 },
];

const STATES: SyncState[] = ["Unknown", "Clean", "Dirty", "Pinned"];

function Node({
  icon,
  title,
  sub,
  highlight,
}: {
  icon: React.ReactNode;
  title: string;
  sub: string;
  highlight?: boolean;
}) {
  return (
    <div className="flex items-center gap-3 md:flex-col md:gap-2 md:text-center">
      <span
        className={cn(
          "relative grid size-11 shrink-0 place-items-center rounded-[4px] border bg-subtle text-brand",
          highlight ? "border-brand/50" : "border-border",
        )}
      >
        {highlight ? <span className="absolute inset-0 animate-pulse-ring rounded-[4px]" /> : null}
        {icon}
      </span>
      <span className="md:mt-1">
        <span className="block font-display text-sm font-semibold text-fg">{title}</span>
        <span className="block font-mono text-[10px] uppercase tracking-[0.14em] text-faint">{sub}</span>
      </span>
    </div>
  );
}

function Flow() {
  return (
    <div className="relative my-1 ml-5 h-7 w-px self-stretch md:mx-1 md:my-0 md:h-px md:flex-1 md:self-center">
      {/* base dashed line (vertical on mobile, horizontal on desktop) */}
      <svg className="absolute inset-0 hidden h-full w-full md:block" preserveAspectRatio="none" viewBox="0 0 100 2">
        <line
          x1="0"
          y1="1"
          x2="100"
          y2="1"
          stroke="var(--brand)"
          strokeOpacity="0.5"
          strokeWidth="1.5"
          strokeDasharray="4 4"
          className="animate-dash"
        />
      </svg>
      <svg className="absolute inset-0 h-full w-full md:hidden" preserveAspectRatio="none" viewBox="0 0 2 100">
        <line
          x1="1"
          y1="0"
          x2="1"
          y2="100"
          stroke="var(--brand)"
          strokeOpacity="0.5"
          strokeWidth="1.5"
          strokeDasharray="4 4"
          className="animate-dash"
        />
      </svg>
      {/* traveling packet (desktop only) */}
      <motion.span
        className="absolute top-1/2 hidden size-1.5 -translate-y-1/2 rounded-[1px] bg-brand md:block"
        style={{ boxShadow: "0 0 8px var(--brand)" }}
        animate={{ left: ["0%", "100%"], opacity: [0, 1, 1, 0] }}
        transition={{ duration: 1.8, repeat: Infinity, ease: "linear" }}
      />
    </div>
  );
}

/** The animated mainnet → mirror → stagenet pipeline. The docs hero centerpiece. */
export function MirrorPipeline() {
  const [tick, setTick] = useState(0);

  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 1800);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="panel relative overflow-hidden p-5 sm:p-7">
      <div className="bg-grid pointer-events-none absolute inset-0 opacity-40" aria-hidden />

      <div className="relative flex flex-col items-stretch md:flex-row md:items-center md:justify-between">
        <Node icon={<Globe size={18} />} title="Mainnet" sub="Helius / Triton" />
        <Flow />
        <Node icon={<LogoMark size={26} />} title="RustAG mirror" sub="fetch · cache · track" highlight />
        <Flow />
        <Node
          icon={<span className="font-mono text-sm font-bold">R</span>}
          title="Stagenet"
          sub="127.0.0.1:8899"
        />
      </div>

      <div className="relative mt-6 space-y-2">
        {ROWS.map((row, i) => {
          const state = STATES[(tick + i) % STATES.length];
          const value = row.base + (tick % 4) * row.step;
          return (
            <motion.div
              key={row.pubkey}
              className="flex items-center justify-between gap-4 rounded-[3px] border border-border bg-subtle px-3.5 py-2.5"
              initial={{ opacity: 0, x: -8 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.15 + i * 0.1, duration: 0.5 }}
            >
              <div className="min-w-0">
                <div className="truncate text-xs font-medium text-fg">{row.label}</div>
                <div className="font-mono text-[11px] text-faint">{row.pubkey}</div>
              </div>
              <div className="flex items-center gap-3">
                <span className="font-mono text-sm tabular-nums text-fg">
                  <AnimatedNumber value={value} format={{ maximumFractionDigits: 2 }} />
                </span>
                <StatePill state={state} />
              </div>
            </motion.div>
          );
        })}
      </div>

      <div className="relative mt-4 flex items-center justify-between border-t border-border pt-3">
        <span className="label !text-faint">Lazy-mirrored on first access</span>
        <span className="inline-flex items-center gap-1.5 font-mono text-[10px] uppercase tracking-[0.16em] text-faint">
          <span className="size-1.5 animate-pulse rounded-[1px] bg-brand" />
          live
        </span>
      </div>
    </div>
  );
}
