import { cn } from "@/lib/cn";

type Phase = 1 | 2 | 3;

const PHASE: Record<Phase, { label: string; cls: string; dot: string }> = {
  1: {
    label: "Phase 1 · Stable",
    cls: "border-state-clean/35 bg-state-clean/12 text-state-clean",
    dot: "bg-state-clean",
  },
  2: {
    label: "Phase 2 · Preview",
    cls: "border-state-pinned/35 bg-state-pinned/12 text-state-pinned",
    dot: "bg-state-pinned",
  },
  3: {
    label: "Phase 3 · Experimental",
    cls: "border-state-dirty/35 bg-state-dirty/12 text-state-dirty",
    dot: "bg-state-dirty",
  },
};

/** Maturity chip used inline next to headings and feature rows. */
export function PhaseBadge({
  phase,
  label,
  className,
}: {
  phase: Phase;
  label?: string;
  className?: string;
}) {
  const p = PHASE[phase];
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-[3px] border px-2 py-0.5 align-middle font-mono text-[10px] uppercase tracking-[0.16em]",
        p.cls,
        className,
      )}
    >
      <span className={cn("size-1.5 rounded-[1px]", p.dot)} />
      {label ?? p.label}
    </span>
  );
}
