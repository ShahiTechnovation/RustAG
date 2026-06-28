"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ArrowUpRight } from "lucide-react";

import { cn } from "@/lib/cn";
import { DOCS_NAV } from "./nav";

/** Grouped left-rail navigation with active-route highlighting. */
export function DocsSidebar({ onNavigate }: { onNavigate?: () => void }) {
  const pathname = usePathname();

  return (
    <nav className="px-4 py-6 text-[13.5px]">
      {DOCS_NAV.map((group) => (
        <div key={group.label} className="mb-6">
          <div className="label mb-2.5 px-2 text-faint">{group.label}</div>
          <ul className="space-y-0.5">
            {group.items.map((item) => {
              const base = item.href.split("#")[0];
              const isHash = item.href.includes("#");
              const active = pathname === base && !isHash;
              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    onClick={onNavigate}
                    className={cn(
                      "group flex items-center justify-between gap-2 rounded-[3px] py-1.5 pl-3 pr-2 transition-colors",
                      isHash && "text-[12.5px]",
                      active
                        ? "bg-brand/10 font-medium text-brand"
                        : "text-muted hover:bg-white/[0.03] hover:text-fg",
                    )}
                  >
                    <span className="flex min-w-0 items-center gap-2.5">
                      <span
                        className={cn(
                          "h-3.5 w-px shrink-0 rounded transition-colors",
                          active ? "bg-brand" : "bg-border group-hover:bg-border-strong",
                        )}
                      />
                      <span className="truncate">{item.title}</span>
                    </span>
                    {item.badge ? (
                      <span className="shrink-0 rounded-[3px] border border-state-pinned/30 bg-state-pinned/10 px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-[0.14em] text-state-pinned">
                        {item.badge}
                      </span>
                    ) : null}
                  </Link>
                </li>
              );
            })}
          </ul>
        </div>
      ))}

      <div className="mt-8 border-t border-border px-2 pt-5">
        <a
          href="https://github.com/ShahiTechnovation/RustAG"
          target="_blank"
          rel="noreferrer"
          className="inline-flex items-center gap-1.5 font-mono text-[11px] uppercase tracking-[0.14em] text-faint transition-colors hover:text-brand"
        >
          View source
          <ArrowUpRight size={12} />
        </a>
        <p className="mt-3 text-[11px] leading-relaxed text-faint">
          Open source · MIT OR Apache-2.0 · workspace v0.1.0
        </p>
      </div>
    </nav>
  );
}
