"use client";

import type { ReactNode } from "react";

import { cn } from "@/lib/cn";
import { GlowCard } from "./GlowCard";

/** A feature tile for the landing bento grid. */
export function BentoCard({
  icon,
  title,
  description,
  media,
  index,
  className,
  accent = "var(--brand)",
}: {
  icon?: ReactNode;
  title: string;
  description: string;
  media?: ReactNode;
  index?: string;
  className?: string;
  accent?: string;
}) {
  return (
    <GlowCard className={cn("group flex flex-col gap-4 p-6", className)}>
      <div className="flex items-center justify-between">
        {icon ? (
          <div
            className="inline-flex size-10 items-center justify-center rounded-[3px] border border-border bg-white/[0.02]"
            style={{ color: accent }}
          >
            {icon}
          </div>
        ) : null}
        {index ? <span className="label text-faint">{index}</span> : null}
      </div>
      <div>
        <h3 className="font-display text-base font-semibold tracking-tight text-fg">{title}</h3>
        <p className="mt-1.5 text-sm leading-relaxed text-muted">{description}</p>
      </div>
      {media ? <div className="mt-auto pt-2">{media}</div> : null}
    </GlowCard>
  );
}
