import type { ReactNode } from "react";

import type { SyncState } from "@rustag/sdk";

export function Card({ children, className = "" }: { children: ReactNode; className?: string }) {
  return (
    <div className={`rounded-xl border border-zinc-800 bg-zinc-900/50 p-5 ${className}`}>
      {children}
    </div>
  );
}

export function StatCard({ label, value, hint }: { label: string; value: ReactNode; hint?: string }) {
  return (
    <Card>
      <div className="text-xs font-medium uppercase tracking-wider text-zinc-500">{label}</div>
      <div className="mt-2 text-2xl font-semibold text-zinc-50">{value}</div>
      {hint ? <div className="mt-1 text-xs text-zinc-500">{hint}</div> : null}
    </Card>
  );
}

const SYNC_STYLES: Record<SyncState, string> = {
  Clean: "bg-emerald-500/15 text-emerald-400 border-emerald-500/30",
  Dirty: "bg-amber-500/15 text-amber-400 border-amber-500/30",
  Pinned: "bg-indigo-500/15 text-indigo-400 border-indigo-500/30",
  Unknown: "bg-zinc-500/15 text-zinc-400 border-zinc-500/30",
};

export function SyncBadge({ state }: { state: SyncState }) {
  return (
    <span className={`inline-flex rounded-full border px-2 py-0.5 text-xs font-medium ${SYNC_STYLES[state]}`}>
      {state}
    </span>
  );
}

export function Badge({ children, tone = "zinc" }: { children: ReactNode; tone?: "zinc" | "emerald" | "red" }) {
  const tones = {
    zinc: "bg-zinc-500/15 text-zinc-300 border-zinc-600/40",
    emerald: "bg-emerald-500/15 text-emerald-400 border-emerald-500/30",
    red: "bg-red-500/15 text-red-400 border-red-500/30",
  } as const;
  return (
    <span className={`inline-flex rounded-full border px-2 py-0.5 text-xs font-medium ${tones[tone]}`}>
      {children}
    </span>
  );
}

export function shortKey(key: string, head = 4, tail = 4): string {
  return key.length > head + tail + 1 ? `${key.slice(0, head)}…${key.slice(-tail)}` : key;
}
