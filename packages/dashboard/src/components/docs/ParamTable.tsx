import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

export type Row = {
  name: string;
  type?: string;
  required?: boolean;
  desc: ReactNode;
};

/** Reference table for CLI flags, params, REST fields, etc. */
export function ParamTable({
  rows,
  cols = ["Name", "Type", "Description"],
  className,
}: {
  rows: Row[];
  cols?: [string, string, string];
  className?: string;
}) {
  return (
    <div className={cn("my-6 overflow-x-auto rounded-[4px] border border-border", className)}>
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-border bg-white/[0.015]">
            <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.14em] text-faint">
              {cols[0]}
            </th>
            <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.14em] text-faint">
              {cols[1]}
            </th>
            <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.14em] text-faint">
              {cols[2]}
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.name} className="border-b border-border/60 last:border-0 align-top">
              <td className="whitespace-nowrap px-4 py-3 font-mono text-[12.5px] text-brand">
                {r.name}
                {r.required ? <span className="ml-1 text-state-dirty">*</span> : null}
              </td>
              <td className="whitespace-nowrap px-4 py-3 font-mono text-[12px] text-accent-2">
                {r.type ?? "—"}
              </td>
              <td className="px-4 py-3 leading-relaxed text-muted">{r.desc}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
