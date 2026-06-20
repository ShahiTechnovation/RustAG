"use client";

import type { ReactNode } from "react";
import type { Format } from "@number-flow/react";

import type { MetricPoint } from "@rustag/sdk";

import { cn } from "@/lib/cn";
import { Sparkline } from "../Sparkline";
import { AnimatedNumber } from "./AnimatedNumber";
import { GlowCard } from "./GlowCard";

export function StatCard({
  label,
  value,
  hint,
  icon,
  format,
  points,
  accent = "var(--brand)",
  className,
}: {
  label: string;
  value: ReactNode;
  hint?: ReactNode;
  icon?: ReactNode;
  format?: Format;
  points?: MetricPoint[];
  accent?: string;
  className?: string;
}) {
  return (
    <GlowCard className={cn("flex flex-col gap-2 overflow-hidden", className)}>
      <div className="flex items-center justify-between">
        <span className="label">{label}</span>
        {icon ? <span style={{ color: accent }}>{icon}</span> : null}
      </div>
      <div className="font-display text-3xl font-semibold tracking-tight text-fg tabular-nums">
        {typeof value === "number" ? <AnimatedNumber value={value} format={format} /> : value}
      </div>
      {hint ? <div className="text-xs text-faint">{hint}</div> : null}
      {points && points.length > 0 ? (
        <div className="mt-1 -mb-1 h-10 opacity-80">
          <Sparkline points={points} stroke={accent} height={40} />
        </div>
      ) : null}
    </GlowCard>
  );
}
