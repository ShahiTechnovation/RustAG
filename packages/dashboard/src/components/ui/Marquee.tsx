import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

/**
 * Infinite horizontal ticker. Children are duplicated so the CSS translateX(-50%)
 * loop is seamless. Pauses on hover.
 */
export function Marquee({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div className={cn("marquee-paused group relative overflow-hidden", className)}>
      <div className="marquee gap-12 pr-12">
        <div className="flex shrink-0 items-center gap-12">{children}</div>
        <div className="flex shrink-0 items-center gap-12" aria-hidden>
          {children}
        </div>
      </div>
      {/* edge fades */}
      <div className="pointer-events-none absolute inset-y-0 left-0 w-24 bg-gradient-to-r from-bg to-transparent" />
      <div className="pointer-events-none absolute inset-y-0 right-0 w-24 bg-gradient-to-l from-bg to-transparent" />
    </div>
  );
}
