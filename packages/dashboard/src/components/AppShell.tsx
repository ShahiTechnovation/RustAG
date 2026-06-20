"use client";

import type { ReactNode } from "react";
import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { AnimatePresence, motion } from "motion/react";
import {
  Activity,
  CalendarClock,
  Database,
  FlaskConical,
  LayoutDashboard,
  LineChart,
  Menu,
  Radio,
  X,
} from "lucide-react";

import { useStagenet } from "@/lib/hooks";
import { cn } from "@/lib/cn";
import { AnimatedNumber } from "./ui";
import { Logo } from "./Logo";

const NAV = [
  { href: "/app", label: "Overview", icon: LayoutDashboard },
  { href: "/app/accounts", label: "Accounts", icon: Database },
  { href: "/app/transactions", label: "Transactions", icon: Activity },
  { href: "/app/analytics", label: "Analytics", icon: LineChart },
  { href: "/app/schedules", label: "Scheduler", icon: CalendarClock },
  { href: "/app/simulations", label: "Simulations", icon: FlaskConical },
];

function isActive(pathname: string, href: string) {
  return href === "/app" ? pathname === "/app" : pathname.startsWith(href);
}

function NavList({ onNavigate }: { onNavigate?: () => void }) {
  const pathname = usePathname();
  return (
    <nav className="flex flex-col gap-1">
      {NAV.map(({ href, label, icon: Icon }) => {
        const active = isActive(pathname, href);
        return (
          <Link
            key={href}
            href={href}
            onClick={onNavigate}
            className={cn(
              "relative flex items-center gap-3 rounded-[3px] px-3 py-2 text-xs uppercase tracking-wide transition-colors",
              active ? "text-fg" : "text-muted hover:text-fg hover:bg-white/[0.03]",
            )}
          >
            {active ? (
              <motion.span
                layoutId="nav-active"
                className="absolute inset-0 -z-10 rounded-[3px] border border-brand bg-brand/10"
                transition={{ type: "spring", stiffness: 400, damping: 32 }}
              />
            ) : null}
            <Icon size={17} className={active ? "text-brand" : ""} />
            {label}
          </Link>
        );
      })}
    </nav>
  );
}

function PageTitle() {
  const pathname = usePathname();
  const match = [...NAV].reverse().find((n) => isActive(pathname, n.href));
  return (
    <span className="font-display text-sm font-semibold uppercase tracking-wide text-fg">
      {match?.label ?? "Dashboard"}
    </span>
  );
}

function LiveStatus() {
  const { data, isError, isLoading } = useStagenet();
  const live = !!data?.mirrorEnabled && !isError;

  return (
    <div className="flex items-center gap-3">
      {data ? (
        <span className="label hidden items-center gap-1.5 normal-case sm:inline-flex">
          <span>slot</span> <AnimatedNumber value={data.slot} className="font-mono text-muted" />
        </span>
      ) : null}
      <span
        className={cn(
          "inline-flex items-center gap-1.5 rounded-[3px] border px-2.5 py-1 font-mono text-[11px] uppercase tracking-wider",
          isError
            ? "border-red-500/30 bg-red-500/10 text-red-400"
            : live
              ? "border-brand/40 bg-brand/10 text-brand"
              : "border-border-strong bg-white/[0.03] text-faint",
        )}
      >
        <span className="relative flex size-1.5">
          {live ? (
            <span className="absolute inline-flex size-full animate-ping rounded-full bg-brand opacity-60" />
          ) : null}
          <span
            className={cn(
              "relative inline-flex size-1.5 rounded-full",
              isError ? "bg-red-400" : live ? "bg-brand" : "bg-faint",
            )}
          />
        </span>
        {isError ? "Offline" : isLoading ? "Connecting" : live ? "Mirror live" : "Mirror off"}
      </span>
    </div>
  );
}

export function AppShell({ children }: { children: ReactNode }) {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <div className="relative min-h-screen lg:grid lg:grid-cols-[256px_1fr]">
      {/* Desktop sidebar */}
      <aside className="sticky top-0 hidden h-screen flex-col gap-6 border-r border-border bg-surface/40 px-4 py-5 backdrop-blur-xl lg:flex">
        <Logo href="/app" />
        <div className="mt-2">
          <NavList />
        </div>
        <div className="mt-auto rounded-[3px] border border-border bg-white/[0.02] p-3">
          <div className="flex items-center gap-2 text-xs text-faint">
            <Radio size={13} className="text-brand" />
            Real mainnet state, on demand.
          </div>
          <Link
            href="/"
            className="label mt-2 inline-block text-brand transition-colors hover:text-brand-strong"
          >
            ← Back to site
          </Link>
        </div>
      </aside>

      {/* Main column */}
      <div className="flex min-h-screen flex-col">
        <header className="sticky top-0 z-40 flex items-center justify-between gap-4 border-b border-border bg-bg/70 px-4 py-3 backdrop-blur-xl sm:px-6">
          <div className="flex items-center gap-3">
            <button
              onClick={() => setMobileOpen(true)}
              className="grid size-9 place-items-center rounded-[3px] border border-border text-muted hover:border-brand hover:text-fg lg:hidden"
              aria-label="Open menu"
            >
              <Menu size={18} />
            </button>
            <PageTitle />
          </div>
          <LiveStatus />
        </header>

        <main className="flex-1 px-4 py-6 sm:px-6 sm:py-8">{children}</main>
      </div>

      {/* Mobile sheet */}
      <AnimatePresence>
        {mobileOpen ? (
          <motion.div
            className="fixed inset-0 z-50 lg:hidden"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={() => setMobileOpen(false)} />
            <motion.aside
              className="absolute left-0 top-0 flex h-full w-72 flex-col gap-6 border-r border-border bg-surface px-4 py-5"
              initial={{ x: "-100%" }}
              animate={{ x: 0 }}
              exit={{ x: "-100%" }}
              transition={{ type: "spring", stiffness: 380, damping: 36 }}
            >
              <div className="flex items-center justify-between">
                <Logo href="/app" />
                <button
                  onClick={() => setMobileOpen(false)}
                  className="grid size-8 place-items-center rounded-[3px] border border-border text-muted hover:border-brand hover:text-fg"
                  aria-label="Close menu"
                >
                  <X size={16} />
                </button>
              </div>
              <NavList onNavigate={() => setMobileOpen(false)} />
            </motion.aside>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
