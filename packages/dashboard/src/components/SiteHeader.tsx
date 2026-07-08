"use client";

import { useEffect, useState } from "react";
import Link from "next/link";

import { GitHubIcon } from "./icons";
import { ButtonLink } from "./ui";
import { Logo } from "./Logo";
import { cn } from "@/lib/cn";

const LINKS = [
  { href: "#features", label: "Features" },
  { href: "#the-mirror", label: "The mirror" },
  { href: "#how-it-works", label: "How it works" },
  { href: "/docs", label: "Docs" },
];

export function SiteHeader() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 16);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header className="fixed inset-x-0 top-0 z-50 px-4 pt-3 sm:px-6">
      <div
        className={cn(
          "mx-auto flex max-w-6xl items-center justify-between gap-4 rounded-[3px] px-4 py-2.5 transition-all duration-300",
          scrolled ? "glass" : "border border-transparent",
        )}
      >
        <div className="flex items-center gap-2">
          <Logo showTag={false} />
          <span className="rounded-[3px] border border-brand/40 bg-brand/10 px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-[0.18em] text-brand">
            Beta
          </span>
        </div>
        <nav className="hidden items-center gap-1 md:flex">
          {LINKS.map((l) => (
            <Link
              key={l.href}
              href={l.href}
              className="rounded-[3px] px-3 py-1.5 font-mono text-[11px] uppercase tracking-[0.14em] text-muted transition-colors hover:text-fg"
            >
              {l.label}
            </Link>
          ))}
        </nav>
        <div className="flex items-center gap-2">
          <a
            href="https://github.com/ShahiTechnovation/RustAG"
            target="_blank"
            rel="noreferrer"
            className="hidden size-9 place-items-center rounded-[3px] text-muted transition-colors hover:text-fg sm:grid"
            aria-label="GitHub"
          >
            <GitHubIcon size={18} />
          </a>
          <ButtonLink href="/app" size="sm" className="hidden sm:inline-flex">
            Live demo
          </ButtonLink>
          <ButtonLink href="/early-access" size="sm" variant="secondary">
            Early access
          </ButtonLink>
        </div>
      </div>
    </header>
  );
}
