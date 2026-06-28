"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { CornerDownLeft, Search } from "lucide-react";

import { cn } from "@/lib/cn";
import { DOCS_SEARCH_INDEX } from "./nav";

/** ⌘K command palette: fuzzy-ish filter over the docs nav, keyboard-driven. */
export function CommandPalette({ open, onClose }: { open: boolean; onClose: () => void }) {
  const router = useRouter();
  const [q, setQ] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const results = useMemo(() => {
    const s = q.trim().toLowerCase();
    if (!s) return DOCS_SEARCH_INDEX;
    return DOCS_SEARCH_INDEX.filter((r) => `${r.title} ${r.group}`.toLowerCase().includes(s));
  }, [q]);

  useEffect(() => {
    if (open) {
      setQ("");
      setActive(0);
      const t = setTimeout(() => inputRef.current?.focus(), 10);
      return () => clearTimeout(t);
    }
  }, [open]);

  useEffect(() => setActive(0), [q]);

  if (!open) return null;

  const go = (i: number) => {
    const r = results[i];
    if (r) {
      router.push(r.href);
      onClose();
    }
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(a + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      go(active);
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 z-[100] flex items-start justify-center p-4 pt-[12vh]"
      role="dialog"
      aria-modal="true"
      onClick={onClose}
    >
      <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" />
      <div
        className="relative w-full max-w-xl overflow-hidden rounded-[6px] border border-border-strong bg-surface shadow-glow"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 border-b border-border px-4">
          <Search size={16} className="shrink-0 text-faint" />
          <input
            ref={inputRef}
            value={q}
            onChange={(e) => setQ(e.target.value)}
            onKeyDown={onKey}
            placeholder="Search the docs…"
            className="h-12 flex-1 bg-transparent text-sm text-fg outline-none placeholder:text-faint"
          />
          <kbd className="hidden rounded-[3px] border border-border bg-bg px-1.5 py-0.5 font-mono text-[10px] text-faint sm:block">
            ESC
          </kbd>
        </div>
        <ul className="max-h-80 overflow-y-auto p-2">
          {results.length === 0 ? (
            <li className="px-3 py-8 text-center text-sm text-faint">No matches for “{q}”.</li>
          ) : (
            results.map((r, i) => (
              <li key={r.href}>
                <button
                  onMouseEnter={() => setActive(i)}
                  onClick={() => go(i)}
                  className={cn(
                    "flex w-full items-center justify-between gap-3 rounded-[3px] px-3 py-2 text-left transition-colors",
                    i === active ? "bg-brand/10" : "hover:bg-white/[0.03]",
                  )}
                >
                  <span className="flex min-w-0 flex-col">
                    <span className={cn("truncate text-sm", i === active ? "text-fg" : "text-muted")}>
                      {r.title}
                    </span>
                    <span className="font-mono text-[10px] uppercase tracking-[0.14em] text-faint">
                      {r.group}
                    </span>
                  </span>
                  {i === active ? <CornerDownLeft size={13} className="shrink-0 text-brand" /> : null}
                </button>
              </li>
            ))
          )}
        </ul>
        <div className="flex items-center gap-4 border-t border-border px-4 py-2 font-mono text-[10px] uppercase tracking-[0.14em] text-faint">
          <span className="flex items-center gap-1.5">
            <kbd className="rounded-[3px] border border-border px-1">↑</kbd>
            <kbd className="rounded-[3px] border border-border px-1">↓</kbd>
            navigate
          </span>
          <span className="flex items-center gap-1.5">
            <kbd className="rounded-[3px] border border-border px-1">↵</kbd>
            open
          </span>
        </div>
      </div>
    </div>
  );
}
