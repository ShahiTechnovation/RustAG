import type { ReactNode } from "react";
import { Info, Lightbulb, Rocket, TriangleAlert } from "lucide-react";

import { cn } from "@/lib/cn";

type Variant = "info" | "warning" | "tip" | "early";

const STYLES: Record<Variant, { wrap: string; icon: ReactNode; label: string; labelCls: string }> = {
  info: {
    wrap: "border-accent-2/30 bg-accent-2/[0.05]",
    icon: <Info size={15} />,
    label: "Note",
    labelCls: "text-accent-2",
  },
  warning: {
    wrap: "border-state-dirty/30 bg-state-dirty/[0.05]",
    icon: <TriangleAlert size={15} />,
    label: "Heads up",
    labelCls: "text-state-dirty",
  },
  tip: {
    wrap: "border-state-clean/30 bg-state-clean/[0.05]",
    icon: <Lightbulb size={15} />,
    label: "Tip",
    labelCls: "text-state-clean",
  },
  early: {
    wrap: "border-brand/35 bg-brand/[0.06]",
    icon: <Rocket size={15} />,
    label: "Early access",
    labelCls: "text-brand",
  },
};

/** Bordered admonition block. `early` carries the early-access narrative. */
export function Callout({
  variant = "info",
  title,
  children,
  className,
}: {
  variant?: Variant;
  title?: string;
  children: ReactNode;
  className?: string;
}) {
  const s = STYLES[variant];
  return (
    <div className={cn("my-6 rounded-[4px] border px-4 py-3.5", s.wrap, className)}>
      <div className={cn("mb-1.5 flex items-center gap-2 font-mono text-[11px] uppercase tracking-[0.16em]", s.labelCls)}>
        {s.icon}
        {title ?? s.label}
      </div>
      <div className="text-sm leading-relaxed text-muted [&_a]:text-fg [&_a]:underline [&_a]:decoration-border-strong [&_a:hover]:decoration-brand [&_code]:rounded-[3px] [&_code]:border [&_code]:border-border [&_code]:bg-bg [&_code]:px-1 [&_code]:py-0.5 [&_code]:font-mono [&_code]:text-[0.85em] [&_code]:text-fg">
        {children}
      </div>
    </div>
  );
}
