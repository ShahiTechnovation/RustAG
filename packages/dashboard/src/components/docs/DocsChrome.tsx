"use client";

import { useEffect, useState } from "react";
import { Menu, Search, X } from "lucide-react";

import { GitHubIcon } from "@/components/icons";
import { Logo } from "@/components/Logo";
import { ButtonLink } from "@/components/ui";
import { cn } from "@/lib/cn";
import { CommandPalette } from "./CommandPalette";
import { DocsSidebar } from "./DocsSidebar";

/** Fixed docs header + fixed desktop sidebar + mobile drawer + ⌘K palette. */
export function DocsChrome() {
  const [drawer, setDrawer] = useState(false);
  const [palette, setPalette] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPalette((p) => !p);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    document.body.style.overflow = drawer ? "hidden" : "";
    return () => {
      document.body.style.overflow = "";
    };
  }, [drawer]);

  return (
    <>
      <header className="glass fixed inset-x-0 top-0 z-50 h-14 border-x-0 border-t-0">
        <div className="flex h-full items-center gap-3 px-4 sm:px-6">
          <button
            onClick={() => setDrawer(true)}
            aria-label="Open navigation"
            className="-ml-1 grid size-9 place-items-center rounded-[3px] text-muted transition-colors hover:bg-white/5 hover:text-fg lg:hidden"
          >
            <Menu size={18} />
          </button>

          <div className="flex items-center gap-2">
            <Logo showTag={false} />
            <span className="hidden rounded-[3px] border border-brand/40 bg-brand/10 px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-[0.16em] text-brand sm:inline-block">
              v0.1 · Early Access
            </span>
          </div>

          <div className="ml-auto flex items-center gap-2">
            <button
              onClick={() => setPalette(true)}
              className="hidden h-9 items-center gap-2 rounded-[3px] border border-border bg-bg/60 pl-3 pr-2 text-xs text-faint transition-colors hover:border-border-strong hover:text-muted sm:flex"
            >
              <Search size={14} />
              <span>Search docs</span>
              <kbd className="ml-2 rounded-[3px] border border-border px-1.5 py-0.5 font-mono text-[10px] text-faint">
                ⌘K
              </kbd>
            </button>
            <button
              onClick={() => setPalette(true)}
              aria-label="Search"
              className="grid size-9 place-items-center rounded-[3px] text-muted transition-colors hover:bg-white/5 hover:text-fg sm:hidden"
            >
              <Search size={18} />
            </button>
            <a
              href="https://github.com/ShahiTechnovation/RustAG"
              target="_blank"
              rel="noreferrer"
              aria-label="GitHub"
              className="hidden size-9 place-items-center rounded-[3px] text-muted transition-colors hover:text-fg sm:grid"
            >
              <GitHubIcon size={18} />
            </a>
            <ButtonLink href="/app" size="sm" variant="secondary" className="hidden sm:inline-flex">
              Open app
            </ButtonLink>
            <ButtonLink href="/early-access" size="sm" className="hidden md:inline-flex">
              Early access
            </ButtonLink>
          </div>
        </div>
      </header>

      {/* Desktop sidebar */}
      <aside className="fixed bottom-0 left-0 top-14 hidden w-64 overflow-y-auto border-r border-border bg-bg/40 lg:block">
        <DocsSidebar />
      </aside>

      {/* Mobile drawer */}
      {drawer ? (
        <div className="fixed inset-0 z-[70] lg:hidden">
          <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" onClick={() => setDrawer(false)} />
          <div className="absolute left-0 top-0 h-full w-72 max-w-[85vw] overflow-y-auto border-r border-border-strong bg-surface">
            <div className="flex h-14 items-center justify-between border-b border-border px-4">
              <Logo showTag={false} />
              <button
                onClick={() => setDrawer(false)}
                aria-label="Close navigation"
                className="grid size-9 place-items-center rounded-[3px] text-muted transition-colors hover:bg-white/5 hover:text-fg"
              >
                <X size={18} />
              </button>
            </div>
            <DocsSidebar onNavigate={() => setDrawer(false)} />
          </div>
        </div>
      ) : null}

      <CommandPalette open={palette} onClose={() => setPalette(false)} />
    </>
  );
}
