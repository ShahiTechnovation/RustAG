import Link from "next/link";

import { GitHubIcon } from "./icons";
import { Logo } from "./Logo";

const COLUMNS = [
  {
    title: "Product",
    links: [
      { label: "Overview", href: "/app" },
      { label: "Rehearse", href: "/app/rehearse" },
      { label: "Forensics", href: "/app/forensics" },
      { label: "Analytics", href: "/app/analytics" },
      { label: "Simulations", href: "/app/simulations" },
    ],
  },
  {
    title: "Features",
    links: [
      { label: "Squads v4 decoder", href: "#features" },
      { label: "Invariant alarms", href: "#features" },
      { label: "EvidenceBundle", href: "#features" },
      { label: "Upgrade CI gate", href: "#features" },
      { label: "Time-travel replay", href: "#features" },
    ],
  },
  {
    title: "Resources",
    links: [
      { label: "Docs", href: "/docs" },
      { label: "How it works", href: "#how-it-works" },
      { label: "GitHub", href: "https://github.com/ShahiTechnovation/RustAG" },
      { label: "Early access", href: "/early-access" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-border px-6 py-14">
      <div className="mx-auto grid max-w-6xl gap-10 sm:grid-cols-2 lg:grid-cols-[1.5fr_1fr_1fr_1fr]">
        <div>
          <Logo />
          <p className="mt-4 max-w-xs text-sm leading-relaxed text-muted">
            The GroundTruth pre-execution assurance layer for Solana — know exactly what a
            transaction does before any multisig signer approves it.
          </p>
          <a
            href="https://github.com/ShahiTechnovation/RustAG"
            target="_blank"
            rel="noreferrer"
            className="mt-4 inline-grid size-9 place-items-center rounded-[3px] border border-border text-muted transition-colors hover:border-brand hover:text-brand"
            aria-label="GitHub"
          >
            <GitHubIcon size={18} />
          </a>
        </div>
        {COLUMNS.map((col) => (
          <div key={col.title}>
            <div className="label text-faint">{col.title}</div>
            <ul className="mt-3 space-y-2">
              {col.links.map((l) => (
                <li key={l.label}>
                  <Link href={l.href} className="text-sm text-muted transition-colors hover:text-fg">
                    {l.label}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <div className="mx-auto mt-12 max-w-6xl border-t border-border pt-6 text-xs text-faint">
        © {new Date().getFullYear()} RustAG · GroundTruth for Solana. Built for the ecosystem.
      </div>
    </footer>
  );
}
