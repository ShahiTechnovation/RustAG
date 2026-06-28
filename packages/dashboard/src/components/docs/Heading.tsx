import type { ReactNode } from "react";
import { Hash } from "lucide-react";

import { cn } from "@/lib/cn";

/** Anchored section heading (h2) with a hover # link — the docs signature. */
export function H2({ id, children, className }: { id: string; children: ReactNode; className?: string }) {
  return (
    <h2
      id={id}
      className={cn(
        "group/h scroll-mt-24 pt-4 font-display text-2xl font-semibold tracking-tight text-fg sm:text-[1.7rem]",
        className,
      )}
    >
      <a href={`#${id}`} className="inline-flex items-center gap-2">
        <span>{children}</span>
        <Hash
          size={16}
          className="text-faint opacity-0 transition-opacity group-hover/h:opacity-100"
        />
      </a>
    </h2>
  );
}

/** Anchored subsection heading (h3). */
export function H3({ id, children, className }: { id: string; children: ReactNode; className?: string }) {
  return (
    <h3
      id={id}
      className={cn(
        "group/h scroll-mt-24 pt-2 font-display text-lg font-semibold tracking-tight text-fg",
        className,
      )}
    >
      <a href={`#${id}`} className="inline-flex items-center gap-2">
        <span>{children}</span>
        <Hash
          size={14}
          className="text-faint opacity-0 transition-opacity group-hover/h:opacity-100"
        />
      </a>
    </h3>
  );
}
