"use client";

import { useId } from "react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import type { MetricPoint } from "@rustag/sdk";

function timeLabel(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

type TooltipProps = {
  active?: boolean;
  payload?: { value: number; payload: MetricPoint }[];
  format: (v: number) => string;
};

function ChartTooltip({ active, payload, format }: TooltipProps) {
  if (!active || !payload?.length) return null;
  const p = payload[0];
  return (
    <div className="rounded-[3px] border border-border bg-surface px-3 py-2 text-xs">
      <div className="label normal-case text-faint">{timeLabel(p.payload.t)}</div>
      <div className="mt-0.5 font-display font-semibold tabular-nums text-fg">{format(p.value)}</div>
    </div>
  );
}

export function MetricChart({
  points,
  color,
  format = (v) => v.toLocaleString(),
  height = 240,
  minimal = false,
}: {
  points: MetricPoint[];
  color: string;
  format?: (v: number) => string;
  height?: number;
  minimal?: boolean;
}) {
  const id = useId().replace(/:/g, "");

  if (points.length === 0) {
    return (
      <div className="flex items-center justify-center text-xs text-faint" style={{ height }}>
        no data yet
      </div>
    );
  }

  return (
    <div style={{ height, width: "100%" }}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={points} margin={{ top: 8, right: 8, left: minimal ? 0 : 8, bottom: 0 }}>
          <defs>
            <linearGradient id={`grad-${id}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={color} stopOpacity={0.35} />
              <stop offset="100%" stopColor={color} stopOpacity={0} />
            </linearGradient>
          </defs>
          {minimal ? null : (
            <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" vertical={false} />
          )}
          <XAxis
            dataKey="t"
            tickFormatter={timeLabel}
            hide={minimal}
            tick={{ fill: "var(--fg-subtle)", fontSize: 11 }}
            axisLine={false}
            tickLine={false}
            minTickGap={48}
          />
          <YAxis
            hide={minimal}
            tickFormatter={(v) => format(v as number)}
            tick={{ fill: "var(--fg-subtle)", fontSize: 11 }}
            axisLine={false}
            tickLine={false}
            width={56}
          />
          <Tooltip
            content={<ChartTooltip format={format} />}
            cursor={{ stroke: "rgba(255,255,255,0.12)" }}
          />
          <Area
            type="monotone"
            dataKey="v"
            stroke={color}
            strokeWidth={2}
            fill={`url(#grad-${id})`}
            isAnimationActive={false}
            dot={false}
            activeDot={{ r: 3, fill: color, stroke: "var(--bg)", strokeWidth: 2 }}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
