"use client";

import { motion } from "motion/react";
import { Globe } from "lucide-react";

import type { AccountInfo, SyncState } from "@rustag/sdk";

import { AnimatedNumber, StatePill } from "@/components/ui";
import { useAccounts, useStagenet } from "@/lib/hooks";
import { cn } from "@/lib/cn";

type Row = { label: string; pubkey: string; sol: number; state: SyncState };

// Shown only when the demo backend is unreachable/asleep — clearly an
// illustration, replaced by real mirrored accounts the moment data arrives.
const FALLBACK: Row[] = [
  { label: "Oracle · Pyth", pubkey: "H6AR…jcW9", sol: 0.023, state: "Clean" },
  { label: "Token program", pubkey: "Toke…5DA", sol: 0, state: "Clean" },
  { label: "Demo wallet", pubkey: "US51…LFx", sol: 5.01, state: "Dirty" },
];

function short(pk: string) {
  return pk.length > 10 ? `${pk.slice(0, 4)}…${pk.slice(-4)}` : pk;
}

function labelFor(a: AccountInfo): string {
  switch (a.category) {
    case "Oracle":
      return "Oracle · Pyth";
    case "TokenMint":
      return "Token mint";
    case "Program":
      return "Program";
    default:
      return a.executable ? "Program" : "Account";
  }
}

function AccountRow({ row, index }: { row: Row; index: number }) {
  return (
    <motion.div
      className="flex items-center justify-between gap-4 rounded-[3px] border border-border bg-subtle px-4 py-3"
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: 0.2 + index * 0.12, duration: 0.6 }}
    >
      <div className="min-w-0">
        <div className="truncate text-xs font-medium text-fg">{row.label}</div>
        <div className="font-mono text-[11px] text-faint">{row.pubkey}</div>
      </div>
      <div className="flex items-center gap-3">
        <span className="font-mono text-sm text-fg tabular-nums">
          <AnimatedNumber value={row.sol} format={{ maximumFractionDigits: 3 }} />
          <span className="text-faint"> ◎</span>
        </span>
        <StatePill state={row.state} />
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
  const { data: accounts } = useAccounts(12);
  const { data: stagenet } = useStagenet();

  const live = !!accounts && accounts.length > 0;
  const rows: Row[] = live
    ? accounts.slice(0, 3).map((a) => ({
        label: labelFor(a),
        pubkey: short(a.pubkey),
        sol: a.sol,
        state: a.syncState,
      }))
    : FALLBACK;

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
          {rows.map((row, i) => (
            <AccountRow key={`${row.pubkey}-${i}`} row={row} index={i} />
          ))}
        </div>

        <div className="mt-4 flex items-center justify-between border-t border-border pt-3 text-[11px] text-faint">
          <span className="label !text-faint">Lazy-mirrored from mainnet</span>
          <span className="inline-flex items-center gap-1.5 font-mono uppercase tracking-wider text-faint">
            <span
              className={cn(
                "size-1.5 rounded-[1px]",
                live ? "animate-pulse bg-brand" : "bg-faint",
              )}
            />
            {live ? `${(stagenet?.accounts ?? rows.length).toLocaleString()} mirrored` : "syncing"}
          </span>
        </div>
      </div>
    </motion.div>
  );
}
