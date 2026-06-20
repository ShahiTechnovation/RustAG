"use client";

import type { ReactNode } from "react";
import { useRef } from "react";

import { cn } from "@/lib/cn";

/**
 * Elevated card whose radial highlight follows the cursor (the Linear/Resend
 * signature). Sets --mx/--my CSS vars consumed by the `.glow-card` ::before.
 */
export function GlowCard({
  children,
  className,
  as: Tag = "div",
}: {
  children: ReactNode;
  className?: string;
  as?: "div" | "article" | "li";
}) {
  const ref = useRef<HTMLDivElement>(null);

  const onMove = (e: React.MouseEvent) => {
    const el = ref.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    el.style.setProperty("--mx", `${((e.clientX - rect.left) / rect.width) * 100}%`);
    el.style.setProperty("--my", `${((e.clientY - rect.top) / rect.height) * 100}%`);
  };

  return (
    <Tag
      ref={ref as never}
      onMouseMove={onMove}
      className={cn("glow-card p-5", className)}
    >
      {children}
    </Tag>
  );
}
