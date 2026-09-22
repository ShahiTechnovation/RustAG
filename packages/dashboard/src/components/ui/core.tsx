import type { ReactNode } from "react";

import type { AccountCategory, SyncState } from "@rustag/sdk";

import { cn } from "@/lib/cn";

/** Truncate a base58 key for display: `9xQe…F4kP`. */
export function shortKey(key: string, head = 4, tail = 4): string {
  return key.length > head + tail + 1 ? `${key.slice(0, head)}…${key.slice(-tail)}` : key;
}

/** Flat elevated panel. */
export function Card({ children, className = "" }: { children: ReactNode; className?: string }) {
  return (
    <div className={cn("rounded-card border border-border bg-surface p-5", className)}>{children}</div>
  );
}

/** Mono uppercase tracked label — the signature eyebrow/caption. */
export function Eyebrow({
  children,
  index,
  className,
}: {
  children: ReactNode;
  index?: string;
  className?: string;
}) {
  return (
    <div className={cn("label flex items-center gap-2", className)}>
      {index ? <span className="text-brand">{index}</span> : null}
      <span>{children}</span>
    </div>
  );
}

/* --- Account state machine: single source of truth for colors --------------- */

const SYNC_STYLES: Record<SyncState, string> = {
  Clean: "bg-state-clean/12 text-state-clean border-state-clean/35",
  Dirty: "bg-state-dirty/12 text-state-dirty border-state-dirty/35",
  Pinned: "bg-state-pinned/12 text-state-pinned border-state-pinned/35",
  Unknown: "bg-state-unknown/12 text-faint border-state-unknown/35",
};

const SYNC_DOT: Record<SyncState, string> = {
  Clean: "bg-state-clean",
  Dirty: "bg-state-dirty",
  Pinned: "bg-state-pinned",
  Unknown: "bg-state-unknown",
};

/** The Dirty/Clean/Pinned/Unknown pill — used in hero, accounts table, explainer. */
export function StatePill({ state, className = "" }: { state: SyncState; className?: string }) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-[3px] border px-2 py-0.5 font-mono text-[11px] uppercase tracking-wider",
        SYNC_STYLES[state],
        className,
      )}
    >
      <span className={cn("size-1.5 rounded-[1px]", SYNC_DOT[state])} />
      {state}
    </span>
  );
}

/** Backwards-compatible alias for the previous component name. */
export const SyncBadge = StatePill;

const CATEGORY_STYLES: Record<AccountCategory, string> = {
  Oracle: "bg-accent-2/10 text-accent-2 border-accent-2/30",
  Program: "bg-brand/10 text-brand border-brand/30",
  TokenMint: "bg-accent/10 text-accent border-accent/30",
  Data: "bg-state-unknown/12 text-muted border-border-strong",
};

export function CategoryBadge({ category }: { category: AccountCategory | null }) {
  if (!category) return <span className="text-faint">—</span>;
  return (
    <span
      className={cn(
        "inline-flex rounded-[3px] border px-2 py-0.5 font-mono text-[11px] uppercase tracking-wider",
        CATEGORY_STYLES[category],
      )}
    >
      {category}
    </span>
  );
}

const TONE_STYLES = {
  zinc: "bg-state-unknown/12 text-muted border-border-strong",
  brand: "bg-brand/12 text-brand border-brand/35",
  emerald: "bg-brand/12 text-brand border-brand/35",
  red: "bg-red-500/12 text-red-400 border-red-500/35",
  amber: "bg-state-dirty/12 text-state-dirty border-state-dirty/35",
} as const;

export function Badge({
  children,
  tone = "zinc",
  className = "",
}: {
  children: ReactNode;
  tone?: keyof typeof TONE_STYLES;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-[3px] border px-2 py-0.5 font-mono text-[11px] uppercase tracking-wider",
        TONE_STYLES[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}
