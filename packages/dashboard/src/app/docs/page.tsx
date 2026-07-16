import type { Metadata } from "next";
import Link from "next/link";
import {
  ArrowRight,
  ArrowUpRight,
  Boxes,
  Code2,
  Plug,
  Rocket,
  ShieldCheck,
  SquareTerminal,
} from "lucide-react";

import { CodeBlock } from "@/components/docs/CodeBlock";
import { MirrorPipeline } from "@/components/docs/MirrorPipeline";
import { ParticleField } from "@/components/marketing/ParticleField";
import {
  ButtonLink,
  Eyebrow,
  GlowCard,
  GridBackground,
  Reveal,
  RingField,
  StatePill,
} from "@/components/ui";

export const metadata: Metadata = {
  title: "Introduction",
  description:
    "RustAG is the GroundTruth layer for Solana — attested pre-execution assurance for privileged operations. Rehearse Squads proposals, get signed EvidenceBundles, run counterfactual forensics.",
};

const HERO_STATS = [
  { value: "$0", label: "Cost to rehearse" },
  { value: "Grade A", label: "Fidelity" },
  { value: "11", label: "Semantic change types" },
  { value: "6", label: "Invariant rules" },
];

const CATEGORIES = [
  {
    href: "/docs/quickstart",
    icon: <Rocket size={18} />,
    title: "Quickstart",
    desc: "From install to a signed EvidenceBundle in about two minutes using the built-in demo.",
    meta: "Get started",
  },
  {
    href: "/docs/concepts",
    icon: <Boxes size={18} />,
    title: "Core concepts",
    desc: "The sealed two-pass rehearsal, the EvidenceBundle format, Grade A/B fidelity, and the account closure model.",
    meta: "Concept",
  },
  {
    href: "/docs/cli",
    icon: <SquareTerminal size={18} />,
    title: "CLI reference",
    desc: "Every rustag subcommand — rehearse, forensics, record, verify, attest, and the full app management surface.",
    meta: "Reference",
  },
  {
    href: "/docs/sdk",
    icon: <Code2 size={18} />,
    title: "SDK & API",
    desc: "@rustag/sdk, POST /api/rehearse, POST /api/verify, and the Solana JSON-RPC surface.",
    meta: "Reference",
  },
  {
    href: "/docs/architecture",
    icon: <Plug size={18} />,
    title: "Architecture",
    desc: "Dual-layer design: Ingest (TouchSet, Squads, MultiRpc) + Sealed Rehearsal (LiteSVM, semantic diff, invariant policy).",
    meta: "Advanced",
  },
  {
    href: "/docs/security",
    icon: <ShieldCheck size={18} />,
    title: "Trust & security",
    desc: "Threat model, Ed25519 attestation, N-of-M provenance, and an honest list of early-access limits.",
    meta: "Trust",
  },
];

const STATES = [
  { state: "Unknown" as const, desc: "Never fetched. Pulled lazily from mainnet on first access." },
  { state: "Clean" as const, desc: "A faithful mainnet copy. Background sync keeps oracles fresh." },
  { state: "Dirty" as const, desc: "Written by a local tx. Frozen from mainnet sync forever." },
  { state: "Pinned" as const, desc: "Set via override. Locked to exactly the value you chose." },
];

const QUICKSTART = `# 1. Point at a mainnet RPC (Helius free key recommended)
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"

# 2. Rehearse a Squads proposal (built-in demo works without a key)
rustag rehearse --demo

# 3. Rehearse a real proposal from mainnet
rustag rehearse \\
  --proposal 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU \\
  --rpc $RUSTAG_MAINNET_RPC

# 4. Verify the bundle offline (zero network needed)
rustag verify groundtruth-bundle.json --closure groundtruth-closure.json`;

export default function DocsHome() {
  return (
    <div className="pb-20">
      {/* ---------------------------------------------------------------- Hero */}
      <section className="relative overflow-hidden border-b border-border px-5 pb-16 pt-14 sm:px-8 sm:pt-16">
        <RingField />
        <GridBackground className="opacity-40" />
        <ParticleField className="absolute inset-0 -z-10 size-full opacity-25 [mask-image:radial-gradient(ellipse_70%_60%_at_50%_30%,#000,transparent)]" />

        <div className="mx-auto grid max-w-6xl items-center gap-12 lg:grid-cols-[1.05fr_0.95fr]">
          <div>
            <Eyebrow className="text-brand">Documentation · Early Access</Eyebrow>
            <h1 className="mt-5 text-balance font-display text-4xl font-bold leading-[1.02] tracking-tight text-fg sm:text-5xl lg:text-6xl">
              Know what a transaction{" "}
              <em className="font-serif font-normal italic text-brand">does</em>
              {" "}before you sign it.
            </h1>
            <p className="mt-6 max-w-xl text-pretty text-lg leading-relaxed text-muted">
              RustAG rehearses any Solana transaction against faithful mainnet state in a sealed sandbox
              — then emits a{" "}
              <span className="text-fg">cryptographically signed EvidenceBundle</span>{" "}
              with a semantic diff, invariant alarms, and compute used. Verify it offline,
              before a single multisig signer approves.
            </p>

            <div className="mt-8 flex flex-wrap items-center gap-3">
              <ButtonLink href="/docs/quickstart" size="lg" className="group">
                Quickstart
                <ArrowRight size={18} className="transition-transform group-hover:translate-x-0.5" />
              </ButtonLink>
              <ButtonLink href="/docs/cli" size="lg" variant="secondary">
                CLI reference
              </ButtonLink>
            </div>

            <dl className="mt-10 grid max-w-lg grid-cols-2 divide-x divide-y divide-border border border-border sm:grid-cols-4 sm:divide-y-0">
              {HERO_STATS.map((s) => (
                <div key={s.label} className="flex flex-col gap-1 px-4 py-4">
                  <dt className="order-2 font-mono text-[10px] uppercase tracking-[0.16em] text-faint">
                    {s.label}
                  </dt>
                  <dd className="order-1 font-display text-2xl font-bold tracking-tight text-fg">
                    {s.value}
                  </dd>
                </div>
              ))}
            </dl>
          </div>

          <Reveal className="lg:pl-4">
            <MirrorPipeline />
          </Reveal>
        </div>
      </section>

      <div className="mx-auto max-w-6xl px-5 sm:px-8">
        {/* ------------------------------------------------ Early-access banner */}
        <div className="mt-10 flex flex-col gap-3 rounded-[5px] border border-brand/30 bg-brand/[0.05] px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-start gap-3">
            <Rocket size={16} className="mt-0.5 shrink-0 text-brand" />
            <p className="text-sm leading-relaxed text-muted">
              <span className="font-medium text-fg">RustAG is in early access.</span> Phase 1 — the lazy
              mirror, dirty/clean tracking, unlimited airdrops, Solana-compatible RPC, persistence, CLI,
              SDK, and dashboard — is a working local MVP. Phases 2 &amp; 3 ship in the same repo and are
              newer. You build from source; nothing is published to crates.io / npm yet.
            </p>
          </div>
        </div>

        {/* ------------------------------------------------------ Category grid */}
        <section className="mt-16">
          <Eyebrow className="text-brand">Browse the docs</Eyebrow>
          <h2 className="mt-3 font-display text-2xl font-semibold tracking-tight text-fg sm:text-3xl">
            Everything to ship against real mainnet state
          </h2>
          <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {CATEGORIES.map((c) => (
              <Link key={c.href} href={c.href} className="group">
                <GlowCard className="flex h-full flex-col gap-4 p-5">
                  <div className="flex items-center justify-between">
                    <span className="inline-grid size-10 place-items-center rounded-[3px] border border-border bg-white/[0.02] text-brand">
                      {c.icon}
                    </span>
                    <span className="label text-faint">{c.meta}</span>
                  </div>
                  <div>
                    <h3 className="flex items-center gap-1.5 font-display text-base font-semibold tracking-tight text-fg">
                      {c.title}
                      <ArrowUpRight
                        size={15}
                        className="text-faint transition-all group-hover:translate-x-0.5 group-hover:-translate-y-0.5 group-hover:text-brand"
                      />
                    </h3>
                    <p className="mt-1.5 text-sm leading-relaxed text-muted">{c.desc}</p>
                  </div>
                </GlowCard>
              </Link>
            ))}
          </div>
        </section>

        {/* ------------------------------------------------ State machine teaser */}
        <section className="mt-20">
          <div className="grid gap-10 lg:grid-cols-[0.8fr_1.2fr] lg:items-center">
            <div>
              <Eyebrow className="text-brand">The one core idea</Eyebrow>
              <h2 className="mt-3 font-display text-2xl font-semibold tracking-tight text-fg sm:text-3xl">
                Clean until you touch it.
              </h2>
              <p className="mt-4 text-pretty leading-relaxed text-muted">
                Every account carries a sync state. It is how RustAG replays mainnet on a local SVM with no
                block hash to fork from — and the single idea the whole product is built around.
              </p>
              <Link
                href="/docs/concepts"
                className="mt-5 inline-flex items-center gap-1.5 font-mono text-[12px] uppercase tracking-[0.14em] text-brand transition-opacity hover:opacity-80"
              >
                Read core concepts <ArrowRight size={13} />
              </Link>
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              {STATES.map((s) => (
                <div key={s.state} className="rounded-[4px] border border-border bg-surface p-4">
                  <StatePill state={s.state} />
                  <p className="mt-2.5 text-sm leading-relaxed text-muted">{s.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* -------------------------------------------------- Quickstart preview */}
        <section className="mt-20">
          <div className="grid gap-8 lg:grid-cols-[1fr_1.1fr] lg:items-start">
            <div>
              <Eyebrow className="text-brand">Two minutes</Eyebrow>
              <h2 className="mt-3 font-display text-2xl font-semibold tracking-tight text-fg sm:text-3xl">
                Point your <code className="font-mono text-[0.8em] text-accent-2">Connection</code> at it
                and go
              </h2>
              <p className="mt-4 text-pretty leading-relaxed text-muted">
                A stagenet exposes a Solana-compatible JSON-RPC on{" "}
                <code className="rounded-[3px] border border-border bg-bg px-1.5 py-0.5 font-mono text-[0.82em] text-fg">
                  127.0.0.1:8899
                </code>
                , WebSocket on <span className="text-fg">:8900</span>, and a REST API on{" "}
                <span className="text-fg">:9000</span>. Swap one endpoint — Anchor, the Solana CLI, and
                @solana/web3.js just work.
              </p>
              <ButtonLink href="/docs/quickstart" variant="secondary" size="md" className="mt-6 group">
                Full quickstart
                <ArrowRight size={16} className="transition-transform group-hover:translate-x-0.5" />
              </ButtonLink>
            </div>
            <CodeBlock code={QUICKSTART} lang="bash" filename="terminal" className="my-0" />
          </div>
        </section>
      </div>
    </div>
  );
}
