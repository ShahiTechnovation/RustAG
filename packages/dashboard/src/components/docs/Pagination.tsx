"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ArrowLeft, ArrowRight } from "lucide-react";

import { DOC_ORDER } from "./nav";

/** Prev / next page footer, derived from DOC_ORDER and the current route. */
export function Pagination() {
  const pathname = usePathname();
  const idx = DOC_ORDER.findIndex((d) => d.href === pathname);
  if (idx === -1) return null;

  const prev = idx > 0 ? DOC_ORDER[idx - 1] : null;
  const next = idx < DOC_ORDER.length - 1 ? DOC_ORDER[idx + 1] : null;

  return (
    <div className="mt-16 grid gap-3 border-t border-border pt-8 sm:grid-cols-2">
      {prev ? (
        <Link
          href={prev.href}
          className="glow-card group flex flex-col gap-1 p-4 sm:items-start"
        >
          <span className="label flex items-center gap-1.5 text-faint">
            <ArrowLeft size={12} /> Previous
          </span>
          <span className="font-display font-semibold text-fg transition-colors group-hover:text-brand">
            {prev.title}
          </span>
        </Link>
      ) : (
        <span />
      )}
      {next ? (
        <Link
          href={next.href}
          className="glow-card group flex flex-col gap-1 p-4 text-right sm:items-end"
        >
          <span className="label flex items-center gap-1.5 text-faint">
            Next <ArrowRight size={12} />
          </span>
          <span className="font-display font-semibold text-fg transition-colors group-hover:text-brand">
            {next.title}
          </span>
        </Link>
      ) : (
        <span />
      )}
    </div>
  );
}
